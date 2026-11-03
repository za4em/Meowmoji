package com.meowmoji.sdk

internal object MeowmojiNative {

    private var loaded = false

    @Synchronized
    fun load() {
        if (loaded) return

        System.loadLibrary("LiteRt")
        System.loadLibrary("meowmoji")
        loaded = true
    }

    external fun nativeOpen(path: String, accelerator: Int): Long

    external fun nativeRun(handle: Long, tokenIds: IntArray, scores: FloatArray): Int

    external fun nativeClose(handle: Long)
}