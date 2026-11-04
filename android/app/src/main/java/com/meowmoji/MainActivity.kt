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
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import com.meowmoji.compose.rememberMatches
import com.meowmoji.compose.rememberMeowmoji
import com.meowmoji.sdk.Accelerator
import com.meowmoji.ui.theme.MeowmojiTheme
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

@Composable
fun Content(modifier: Modifier = Modifier) {
    val meowmoji = rememberMeowmoji(Accelerator.CPU)
    var input by rememberSaveable { mutableStateOf("") }
    val result = rememberMatches(meowmoji, input, limit = 2, debounce = 500.milliseconds)

    Column(
        modifier = modifier.padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp)
    ) {

        Text("Meowmoji", fontSize = 30.sp)

        TextField(input, onValueChange = { input = it })

        Text("Match: $result")

        result.forEach { emoji ->
            Text("${emoji.emoji} ${"%.4f".format(emoji.score)}")
        }
    }
}