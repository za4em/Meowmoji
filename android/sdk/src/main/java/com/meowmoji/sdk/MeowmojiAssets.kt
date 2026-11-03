package com.meowmoji.sdk

import android.content.Context
import java.io.File

internal object MeowmojiAssets {

    private const val MODEL_VERSION = 1
    private const val MODEL_ASSET = "meowmoji-$MODEL_VERSION.tflite"
    private const val VOCAB_ASSET = "vocab.txt"
    private const val LABELS_ASSET = "labels.txt"

    private var wordIds: Map<String, Int>? = null
    private var labels: List<String>? = null

    @Synchronized
    fun wordIds(context: Context): Map<String, Int> {
        wordIds?.let { return it }

        val loaded = context.assets.open(VOCAB_ASSET).bufferedReader().use { reader ->
            reader.readLines().mapIndexedNotNull { index, line ->
                val word = line.trim()
                if (word.isEmpty()) null else word to index
            }.toMap()
        }
        wordIds = loaded
        return loaded
    }

    @Synchronized
    fun labels(context: Context): List<String> {
        labels?.let { return it }

        val loaded = context.assets.open(LABELS_ASSET).bufferedReader().use { reader ->
            reader.readLines().map { it.trim() }.filter { it.isNotEmpty() }
        }
        labels = loaded
        return loaded
    }

    @Synchronized
    fun modelPath(context: Context): String {
        val modelPath = File(context.cacheDir, MODEL_ASSET)
        if (modelPath.isFile) return modelPath.absolutePath

        val temporary = File(context.cacheDir, "$MODEL_ASSET.tmp")
        context.assets.open(MODEL_ASSET).use { input ->
            temporary.outputStream().use { input.copyTo(it) }
        }
        if (!temporary.renameTo(modelPath)) {
            temporary.delete()
            throw MeowmojiException("Meowmoji model cannot be prepared: model file was not stored")
        }
        return modelPath.absolutePath
    }
}
