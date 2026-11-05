# Meowmoji 🐱

Meowmoji turns a short phrase into an animal emoji. The model runs on the device through [LiteRT](https://ai.google.dev/edge/litert). The model and LiteRT ship with Meowmoji.

The first `match` compiles the model for the selected chip, CPU or GPU. `match` returns a list of emoji and a score, highest first. A higher score is a better fit. Don't forget to `close` the model when you are done.

Meowmoji is `arm64-v8a` and needs Android 7 (API 24).

## Install

```kotlin
dependencies {
    implementation()
}
```

Compose helpers:

```kotlin
dependencies {
    implementation()
}
```

## Score a phrase

```kotlin
Meowmoji(context, Accelerator.GPU).use { meowmoji ->
    val matches = meowmoji.match("fluffy cat")
}
```

Choose `Accelerator.CPU` or `Accelerator.GPU`. Ops the GPU cannot run stay on the CPU.

## Compose

```kotlin
val meowmoji = rememberMeowmoji(Accelerator.GPU)
val matches = rememberMatchDebounced(meowmoji, text)
```

`rememberMatchLatest` scores the current phrase and cancels the previous score. `rememberMatchDebounced` waits for `debounce`, then scores. If a score is already running, it finishes.

## Screenshot
