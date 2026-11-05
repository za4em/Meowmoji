package com.meowmoji.sdk

import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.Assert.assertEquals
import org.junit.Assert.assertThrows
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

/**
 * Checks [Meowmoji.match] and [Meowmoji.close] on CPU.
 *
 * "cat" must return 🐈 and "dog" must return 🐕. A phrase with no known
 * word returns an empty list. The test does not record time or memory.
 */
@RunWith(AndroidJUnit4::class)
class MeowmojiTest {

    private val context = InstrumentationRegistry.getInstrumentation().targetContext

    @Test
    fun textWithNoKnownWordReturnsEmpty() {
        Meowmoji(context, Accelerator.CPU).use { meowmoji ->
            assertTrue(meowmoji.match("").isEmpty())
            assertTrue(meowmoji.match("   ").isEmpty())
            assertTrue(meowmoji.match("!!!").isEmpty())
            assertTrue(meowmoji.match("zzzzqqq").isEmpty())
        }
    }

    @Test
    fun catAndDogReturnTheirEmoji() {
        Meowmoji(context, Accelerator.CPU).use { meowmoji ->
            assertEquals("🐈", meowmoji.match("cat").first().emoji)
            assertEquals("🐕", meowmoji.match("dog").first().emoji)
        }
    }

    @Test
    fun casePunctuationAndAnUnknownWordStillMatch() {
        Meowmoji(context, Accelerator.CPU).use { meowmoji ->
            assertEquals("🐈", meowmoji.match("Cat!!!").first().emoji)
            assertEquals("🐈", meowmoji.match("cat zzzzqqq").first().emoji)
            assertEquals("🐈", meowmoji.match("cat cat cat cat cat").first().emoji)
        }
    }

    @Test
    fun limitControlsTheList() {
        Meowmoji(context, Accelerator.CPU).use { meowmoji ->
            assertThrows(IllegalArgumentException::class.java) { meowmoji.match("cat", 0) }
            assertThrows(IllegalArgumentException::class.java) { meowmoji.match("cat", -1) }

            val one = meowmoji.match("cat", 1)
            assertEquals(listOf("🐈"), one.map { it.emoji })

            val matches = meowmoji.match("cat")
            assertTrue(matches.size in 1..3)
            assertEquals("🐈", matches.first().emoji)
            assertEquals(matches.map { it.emoji }, matches.map { it.emoji }.distinct())
            assertTrue(matches.zipWithNext().all { it.first.score >= it.second.score })
        }
    }

    @Test
    fun closeRejectsALaterMatch() {
        val unopened = Meowmoji(context, Accelerator.CPU)
        unopened.close()
        unopened.close()
        val beforeOpen = assertThrows(IllegalStateException::class.java) {
            unopened.match("cat")
        }
        assertEquals("Meowmoji model is already closed", beforeOpen.message)

        val opened = Meowmoji(context, Accelerator.CPU)
        opened.match("cat")
        opened.close()
        opened.close()
        val afterOpen = assertThrows(IllegalStateException::class.java) {
            opened.match("dog")
        }
        assertEquals("Meowmoji model is already closed", afterOpen.message)
    }
}
