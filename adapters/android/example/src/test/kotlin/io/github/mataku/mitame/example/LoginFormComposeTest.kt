package io.github.mataku.mitame.example

import androidx.compose.ui.test.junit4.createComposeRule
import io.github.mataku.mitame.compose.captureMitame
import org.junit.Rule
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [35], qualifiers = "w360dp-h640dp-xxhdpi")
class LoginFormComposeTest {
    @get:Rule
    val composeTestRule = createComposeRule()

    @Test
    fun loginFormCompose() {
        composeTestRule.setContent { LoginFormComposable(dark = false) }
        composeTestRule.captureMitame("login_form_compose", mapOf("theme" to "light"))
    }
}
