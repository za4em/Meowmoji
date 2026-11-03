package com.meowmoji.sdk

/**
 * Chip that compiles and runs a [Meowmoji] model.
 */
enum class Accelerator(internal val nativeValue: Int) {
    /** On-device CPU. */
    CPU(0),

    /** On-device GPU. Ops the GPU cannot run stay on the CPU. */
    GPU(1),
}