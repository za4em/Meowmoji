package com.meowmoji

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.runtime.snapshotFlow
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.meowmoji.sdk.Accelerator
import com.meowmoji.sdk.Meowmoji
import com.meowmoji.ui.theme.MeowmojiTheme
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.FlowPreview
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.withContext
import kotlin.time.Duration.Companion.milliseconds

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent {
            MeowmojiTheme {
                Scaffold(modifier = Modifier.fillMaxSize()) { innerPadding ->
                    Content(
                        modifier = Modifier.padding(innerPadding)
                    )
                }
            }
        }
    }
}

@OptIn(FlowPreview::class)
@Composable
fun Content(modifier: Modifier = Modifier) {
    val context = LocalContext.current
    var meowmoji by remember { mutableStateOf<Meowmoji?>(null) }
    var input by rememberSaveable { mutableStateOf("") }
    var result by rememberSaveable { mutableStateOf("") }

    LaunchedEffect(Unit) {
        meowmoji = withContext(Dispatchers.Default) {
            Meowmoji(context, Accelerator.CPU)
        }
    }

    LaunchedEffect(meowmoji) {
        val model = meowmoji ?: return@LaunchedEffect
        snapshotFlow { input }
            .debounce(300.milliseconds)
            .collect { text ->
                result = withContext(Dispatchers.Default) {
                    model.match(text, limit = 2).joinToString { "${it.emoji} ${"%.4f".format(it.score)}" }
                }
            }
    }

    DisposableEffect(Unit) {
        onDispose { meowmoji?.close() }
    }

    Column(
        modifier = modifier.padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {

        Text("Meowmoji", fontSize = 30.sp)

        TextField(input, onValueChange = { input = it })

        Text("Match: $result")
    }
}