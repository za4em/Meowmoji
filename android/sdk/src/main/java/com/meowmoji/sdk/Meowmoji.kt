package com.meowmoji.sdk

import android.content.Context
import java.io.IOException

/**
 * An emoji and how well it fits a phrase.
 *
 * @param emoji emoji that fits the phrase.
 * @param score how well [emoji] fits. Higher is a better fit.
 */
data class EmojiScore(
    val emoji: String,
    val score: Float,
)

/** The model could not be prepared, or a match could not be scored. */
class MeowmojiException(
    message: String,
    cause: Throwable? = null,
) : Exception(message, cause)

/**
 * A Meowmoji model for the selected chip.
 *
 * [match] returns a sorted list of emoji with scores that fit a phrase.
 * [close] frees the model so it can no longer be used.
 *
 * ```
 * Meowmoji(context, Accelerator.CPU).use { meowmoji ->
 *     val matches = meowmoji.match("Meow meow meow")
 * }
 * ```
 *
 * @param context app [Context].
 * @param accelerator chip the model compiles for.
 * @throws IOException if the word list, the labels, or the model file cannot be read.
 * @throws MeowmojiException if the model file cannot be stored.
 */
class Meowmoji(
    context: Context,
    private val accelerator: Accelerator,
) : AutoCloseable {

    private var handle: Long? = null
    private var closed = false

    private val modelPath = MeowmojiAssets.modelPath(context)
    private val wordIds = MeowmojiAssets.wordIds(context)
    private val labels = MeowmojiAssets.labels(context)

    init {
        MeowmojiNative.load()
    }

    /**
     * Returns a sorted list of emoji with scores that fit [text].
     *
     * @param text phrase to match.
     * @param limit how many emoji to return.
     * @return emoji with scores, sorted by score.
     * @throws IllegalStateException if this model is already closed.
     * @throws IllegalArgumentException if [limit] is not positive.
     * @throws MeowmojiException if the model cannot be prepared or scoring fails.
     */
    @Synchronized
    fun match(text: String, limit: Int = 3): List<EmojiScore> {
        if (closed) throw IllegalStateException("Meowmoji model is already closed")
        if (limit <= 0) throw IllegalArgumentException("limit must be positive")

        val chunks = text.toWords().toIds().toChunks()
        if (chunks.isEmpty()) return emptyList()

        val openHandle = handle ?: open()
        return chunks
            .flatMap { ids -> run(openHandle, ids).take(limit) }
            .sortedByDescending { it.score }
            .distinctBy { it.emoji }
            .take(limit)
    }

    private fun open(): Long {
        val opened = MeowmojiNative.nativeOpen(modelPath, accelerator.nativeValue)
        if (opened == 0L) {
            throw MeowmojiException("Meowmoji model cannot be prepared: compilation failed")
        }
        handle = opened
        return opened
    }

    private fun run(handle: Long, ids: IntArray): List<EmojiScore> {
        val scores = FloatArray(labels.size)
        val result = MeowmojiNative.nativeRun(handle = handle, tokenIds = ids, scores = scores)
        if (result != 0) throw MeowmojiException("Meowmoji scoring failed: $result")

        return scores
            .mapIndexed { index, score -> EmojiScore(emoji = labels[index], score) }
            .sortedByDescending { it.score }
    }

    private fun String.toWords(): List<String> {
        return WORD_SEPARATOR.split(this.lowercase()).filter { it.isNotEmpty() }
    }

    private fun List<String>.toIds(): List<Int> {
        return mapNotNull { wordIds[it] }
    }

    private fun List<Int>.toChunks(): List<IntArray> {
        if (isEmpty()) return emptyList()
        if (size <= TOKEN_SLOT_COUNT) return listOf(paddedChunk())

        val chunks = mutableListOf<IntArray>()
        var start = 0
        while (start + TOKEN_SLOT_COUNT <= size) {
            chunks.add(subList(start, start + TOKEN_SLOT_COUNT).toIntArray())
            start += TOKEN_SLOT_COUNT
        }
        if (start < size) chunks.add(takeLast(TOKEN_SLOT_COUNT).toIntArray())
        return chunks
    }

    private fun List<Int>.paddedChunk(): IntArray {
        val row = IntArray(TOKEN_SLOT_COUNT)
        for (index in indices) row[index] = this[index]
        return row
    }

    /** Frees the model so it can no longer be used. */
    @Synchronized
    override fun close() {
        if (closed) return
        closed = true
        handle?.let(MeowmojiNative::nativeClose)
    }

    companion object {
        private const val TOKEN_SLOT_COUNT = 4
        private val WORD_SEPARATOR = Regex("""[^\p{L}]+""")
    }
}