package com.meowmoji.sdk

class Meowmoji(
    val accelerator: Accelerator,
) : AutoCloseable {

    companion object {
        init {
            System.loadLibrary("LiteRt")
            System.loadLibrary("meowmoji")
        }
    }

    fun run(text: String, limit: Int = 3) {

    }

    @Synchronized override fun close() {
    }

    private external fun nativeOpen(path: String, accelerator: Int): Long

    private external fun nativeRun(handle: Long, tokenIds: IntArray, scores: FloatArray): Int

    private external fun nativeClose(handle: Long)

}