package com.meowmoji.sdk

/**
 * Chip that compiles and runs a [Meowmoji] model.
 *
 * @param nativeValue value passed to the native runtime.
 */
enum class Accelerator(val nativeValue: Int) {
    /** On-device CPU. */
    CPU(0),

    /** On-device GPU. Ops the GPU cannot run stay on the CPU. */
    GPU(1),
}