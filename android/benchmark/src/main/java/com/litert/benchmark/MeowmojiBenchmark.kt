package com.litert.benchmark

import android.content.Context
import android.os.Build
import android.os.Debug
import android.util.Log
import com.meowmoji.sdk.Accelerator
import com.meowmoji.sdk.EmojiScore
import com.meowmoji.sdk.Meowmoji
import com.meowmoji.sdk.MeowmojiException
import java.io.File
import java.util.Locale
import kotlin.math.ceil

/**
 * Times `"cat"` on CPU and GPU and writes one line per chip to `meowmoji-perf.txt`.
 *
 * The first match compiles the model, so its time is kept apart from the later
 * matches. Every match must return 🐈. A compile failure or a wrong emoji is
 * written as `error` on that chip's line.
 */
object MeowmojiBenchmark {

    private const val TAG = "MeowmojiPerf"
    private const val REPORT = "meowmoji-perf.txt"
    private const val WARMUP = 5
    private const val SAMPLES = 40
    private const val PHRASE = "cat"
    private const val EXPECTED = "🐈"

    fun run(context: Context) {
        val lines = mutableListOf<String>()
        for (accelerator in listOf(Accelerator.CPU, Accelerator.GPU)) {
            val line = measure(context, accelerator)
            lines += line
            Log.i(TAG, line)
            write(context, lines)
        }
    }

    private fun measure(context: Context, accelerator: Accelerator): String =
        Meowmoji(context, accelerator).use { meowmoji ->
            val before = memory()
            try {
                val run = timeMatches(meowmoji)
                meowmoji.close()
                record(accelerator, run, before, memory())
            } catch (error: MeowmojiException) {
                meowmoji.close()
                errorRecord(accelerator, before, memory(), error)
            }
        }

    private fun timeMatches(meowmoji: Meowmoji): TimedRun {
        val first = timeMatch(meowmoji)
        val afterFirst = memory()
        repeat(WARMUP) { timeMatch(meowmoji) }
        val samples = List(SAMPLES) { timeMatch(meowmoji) }
        return TimedRun(
            firstMatchMs = first.millis,
            steady = samples.map { it.millis }.sorted(),
            top = samples.last().top,
            afterFirst = afterFirst,
        )
    }

    private fun timeMatch(meowmoji: Meowmoji): TimedMatch {
        val (millis, scores) = timed { meowmoji.match(PHRASE) }
        val top = scores.first()
        if (top.emoji != EXPECTED) {
            throw MeowmojiException("expected $EXPECTED was ${top.emoji}")
        }
        return TimedMatch(millis, top)
    }

    private fun errorRecord(
        accelerator: Accelerator,
        before: Memory,
        afterClose: Memory,
        error: MeowmojiException,
    ): String {
        return listOf(
            "accelerator=${accelerator.name}",
            deviceFields(),
            "error=${token(error.message ?: "failed")}",
            "pssBeforeKb=${before.totalPssKb}",
            "nativeBeforeKb=${before.nativeHeapKb}",
            "graphicsBeforeKb=${before.graphicsKb}",
            "pssAfterCloseKb=${afterClose.totalPssKb}",
            "nativeAfterCloseKb=${afterClose.nativeHeapKb}",
            "graphicsAfterCloseKb=${afterClose.graphicsKb}",
            "vmHwmKb=${afterClose.vmHwmKb}",
        ).joinToString(" ")
    }

    private fun record(
        accelerator: Accelerator,
        run: TimedRun,
        before: Memory,
        afterClose: Memory,
    ): String {
        return listOf(
            "accelerator=${accelerator.name}",
            deviceFields(),
            "firstMatchMs=${ms(run.firstMatchMs)}",
            "steadyMedianMs=${ms(median(run.steady))}",
            "steadyP95Ms=${ms(percentile(run.steady, 0.95))}",
            "samples=${run.steady.size}",
            "warmup=$WARMUP",
            "score=${"%.4f".format(Locale.US, run.top.score)}",
            "pssBeforeKb=${before.totalPssKb}",
            "nativeBeforeKb=${before.nativeHeapKb}",
            "graphicsBeforeKb=${before.graphicsKb}",
            "pssAfterFirstKb=${run.afterFirst.totalPssKb}",
            "nativeAfterFirstKb=${run.afterFirst.nativeHeapKb}",
            "graphicsAfterFirstKb=${run.afterFirst.graphicsKb}",
            "pssAfterCloseKb=${afterClose.totalPssKb}",
            "nativeAfterCloseKb=${afterClose.nativeHeapKb}",
            "graphicsAfterCloseKb=${afterClose.graphicsKb}",
            "vmHwmKb=${afterClose.vmHwmKb}",
        ).joinToString(" ")
    }

    private fun deviceFields(): String {
        val abi = Build.SUPPORTED_ABIS.firstOrNull() ?: "unknown"
        val soc =
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) Build.SOC_MODEL else Build.HARDWARE
        return "device=${token(Build.MODEL)} soc=${token(soc)} abi=$abi sdk=${Build.VERSION.SDK_INT}"
    }

    private fun write(context: Context, lines: List<String>) {
        val directory = context.getExternalFilesDir(null) ?: context.filesDir
        File(directory, REPORT).writeText(lines.joinToString("\n") + "\n")
    }

    private fun memory(): Memory {
        val info = Debug.MemoryInfo()
        Debug.getMemoryInfo(info)
        val stats = info.memoryStats
        return Memory(
            totalPssKb = info.totalPss,
            nativeHeapKb = stats["summary.native-heap"]?.toIntOrNull() ?: 0,
            graphicsKb = stats["summary.graphics"]?.toIntOrNull() ?: 0,
            vmHwmKb = vmHwmKb(),
        )
    }

    private fun vmHwmKb(): Int {
        val line = File("/proc/self/status").readLines()
            .firstOrNull { it.startsWith("VmHWM:") }
            ?: return 0
        return line.substringAfter(":").trim().substringBefore(" ").toIntOrNull() ?: 0
    }

    private fun median(sorted: List<Double>): Double {
        if (sorted.isEmpty()) return 0.0
        val mid = sorted.size / 2
        return if (sorted.size % 2 == 0) (sorted[mid - 1] + sorted[mid]) / 2 else sorted[mid]
    }

    private fun percentile(sorted: List<Double>, fraction: Double): Double {
        if (sorted.isEmpty()) return 0.0
        val rank = ceil(sorted.size * fraction).toInt().coerceIn(1, sorted.size) - 1
        return sorted[rank]
    }

    private inline fun <T> timed(block: () -> T): Pair<Double, T> {
        val start = System.nanoTime()
        val value = block()
        return (System.nanoTime() - start) / 1_000_000.0 to value
    }

    private fun ms(value: Double): String = "%.2f".format(Locale.US, value)

    private fun token(value: String): String = value.replace(Regex("\\s+"), "_")

    private data class Memory(
        val totalPssKb: Int,
        val nativeHeapKb: Int,
        val graphicsKb: Int,
        val vmHwmKb: Int,
    )

    private data class TimedMatch(val millis: Double, val top: EmojiScore)

    private data class TimedRun(
        val firstMatchMs: Double,
        val steady: List<Double>,
        val top: EmojiScore,
        val afterFirst: Memory,
    )
}
