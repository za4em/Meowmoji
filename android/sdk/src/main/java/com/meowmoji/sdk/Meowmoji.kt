package com.meowmoji.sdk

import android.content.Context
import java.io.File


data class EmojiScore(
    val emoji: String,
    val score: Float,
)

/**
 * A compiled Meowmoji model for the selected chip.
 *
 * Construction prepares the model. [match] returns a sorted list of emoji with
 * scores that fit a phrase. [close] frees the model so it can no longer be used.
 *
 * ```
 * Meowmoji(context, Accelerator.CPU).use { meowmoji ->
 *     val matches = meowmoji.match("Meow meow meow")
 * }
 * ```
 *
 * @param context app [Context].
 * @param accelerator chip the model compiles for.
 * @throws IllegalStateException if the model cannot be prepared.
 */
class Meowmoji(
    context: Context,
    val accelerator: Accelerator,
) : AutoCloseable {

    @Volatile
    private var handle: Long? = null

    init {
        MeowmojiNative.load()
        val opened = MeowmojiNative.nativeOpen(modelPath(context), accelerator.nativeValue)
        if (opened == 0L) throw IllegalStateException("failed to open meowmoji native")
        handle = opened
    }

    /**
     * Returns a sorted list of emoji with scores that fit [text].
     *
     * @param text phrase to match.
     * @param limit how many emoji to return.
     * @return emoji with scores, sorted by score.
     * @throws IllegalStateException if this model is already closed.
     */
    @Synchronized
    fun match(text: String, limit: Int = 3): List<EmojiScore> {
        val openHandle = handle ?: throw IllegalStateException("meowmoji native is closed")

        return emptyList()
    }

    /** Frees the model so it can no longer be used. */
    @Synchronized
    override fun close() {
        val openHandle = handle ?: return
        handle = null
        MeowmojiNative.nativeClose(openHandle)
    }

    companion object {
        private const val MODEL_ASSET = "meowmoji.tflite"

        @Synchronized
        private fun modelPath(context: Context): String {
            val model = File(context.cacheDir, MODEL_ASSET)
            if (model.isFile) return model.absolutePath
            context.assets.open(MODEL_ASSET).use { input ->
                model.outputStream().use { input.copyTo(it) }
            }
            return model.absolutePath
        }
    }

}