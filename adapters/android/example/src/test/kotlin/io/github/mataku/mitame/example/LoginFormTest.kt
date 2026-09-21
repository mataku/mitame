package io.github.mataku.mitame.example

import androidx.test.core.app.ApplicationProvider
import io.github.mataku.mitame.Mitame
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.RobolectricTestRunner
import org.robolectric.annotation.Config
import org.robolectric.annotation.GraphicsMode

@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [35], qualifiers = "w360dp-h640dp-xxhdpi")
class LoginFormTest {
    @Test
    fun loginForm() {
        val context = ApplicationProvider.getApplicationContext<android.content.Context>()
        val density = context.resources.displayMetrics.density
        val width = (360 * density).toInt()
        val height = (240 * density).toInt()
        for (theme in listOf("light", "dark")) {
            val view = LoginFormView(context, dark = theme == "dark")
            Mitame.capture(view, "login_form", mapOf("theme" to theme), widthPx = width, heightPx = height)
        }
    }
}
