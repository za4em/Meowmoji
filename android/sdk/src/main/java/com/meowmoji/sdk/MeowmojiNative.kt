package com.meowmoji.sdk

internal object MeowmojiNative {

    private var loaded = false

    @Synchronized
    fun load() {
        if (loaded) return

        try {
            System.loadLibrary("LiteRt")
            System.loadLibrary("meowmoji")
        } catch (error: UnsatisfiedLinkError) {
            throw MeowmojiException(
                "Meowmoji model cannot be prepared: libraries failed to load",
                error,
            )
        }
        loaded = true
    }

    external fun nativeOpen(path: String, accelerator: Int): Long

    external fun nativeRun(handle: Long, tokenIds: IntArray, scores: FloatArray): Int

    external fun nativeClose(handle: Long)
}