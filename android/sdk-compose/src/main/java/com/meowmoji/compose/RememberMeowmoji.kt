package com.meowmoji.compose

import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.platform.LocalContext
import com.meowmoji.sdk.Accelerator
import com.meowmoji.sdk.EmojiScore
import com.meowmoji.sdk.Meowmoji
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.withContext
import kotlin.time.Duration
import kotlin.time.Duration.Companion.milliseconds

/**
 * Returns a [Meowmoji] for [accelerator] and closes it when this composable leaves or [accelerator] changes.
 *
 * @param accelerator chip the model compiles for.
 */
@Composable
fun rememberMeowmoji(accelerator: Accelerator = Accelerator.CPU): Meowmoji {
    val context = LocalContext.current
    val meowmoji = remember(context, accelerator) { Meowmoji(context, accelerator) }
    DisposableEffect(meowmoji) {
        onDispose { meowmoji.close() }
    }
    return meowmoji
}

/**
 * Scores [text] once.
 *
 * Scoring runs on [Dispatchers.Default].
 * A new [text] or [limit] cancels the previous scoring. Its result is dropped, and
 * the new phrase is scored.
 *
 * @param meowmoji model that scores the phrase.
 * @param text phrase to match.
 * @param limit how many emoji to return.
 */
@Composable
fun rememberMatchLatest(
    meowmoji: Meowmoji,
    text: String,
    limit: Int = 3,
): List<EmojiScore> {
    var matches by remember { mutableStateOf<List<EmojiScore>>(emptyList()) }

    LaunchedEffect(meowmoji, text, limit) {
        matches = withContext(Dispatchers.Default) {
            meowmoji.match(text, limit)
        }
    }

    return matches
}

/**
 * Scores [text] after it stays unchanged for [debounce].
 *
 * Scoring runs on [Dispatchers.Default].
 * While [debounce] is still waiting, a new [text] or [limit] replaces the phrase
 * that has not been scored yet.
 * Scoring that has already started runs to completion, and its result is kept.
 *
 * @param meowmoji model that scores the phrase.
 * @param text phrase to match.
 * @param limit how many emoji to return.
 * @param debounce how long [text] must stay unchanged before scoring.
 */
@OptIn(FlowPreview::class)
@Composable
fun rememberMatchDebounced(
    meowmoji: Meowmoji,
    text: String,
    limit: Int = 3,
    debounce: Duration = 300.milliseconds,
): List<EmojiScore> {
    var matches by remember { mutableStateOf<List<EmojiScore>>(emptyList()) }
    val currentText by rememberUpdatedState(text)
    val currentLimit by rememberUpdatedState(limit)
    val currentDebounce by rememberUpdatedState(debounce)

    LaunchedEffect(meowmoji) {
        snapshotFlow { currentText to currentLimit }
            .debounce { currentDebounce }
            .collect { (value, matchLimit) ->
                matches = withContext(Dispatchers.Default) {
                    meowmoji.match(value, matchLimit)
                }
            }
    }

    return matches
}
