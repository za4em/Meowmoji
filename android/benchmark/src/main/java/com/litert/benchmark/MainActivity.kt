package com.litert.benchmark

import android.app.Activity
import android.os.Bundle

class MainActivity : Activity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        MeowmojiBenchmark.run(applicationContext)
        finish()
    }
}
