# Android

Both modules are on Maven Central; add them to the test configuration of the module that holds the screenshot tests:

```kotlin
dependencies {
    testImplementation("io.github.mataku:mitame-android:0.1.0")
    testImplementation("io.github.mataku:mitame-android-compose:0.1.0")
}
```

`adapters/android/mitame` is an Android library that depends on the Android SDK only. It takes a `View` or a `Bitmap` and writes the PNG and sidecar; how the bitmap is produced is the project's choice. The example module renders with Robolectric's native graphics, which needs no emulator and draws the same pixels on macOS and Linux (0 differing pixels between a Mac and `ubuntu-latest` on the example), so one committed baseline serves local runs and CI; Paparazzi or an instrumented test can hand `Mitame.write` a bitmap just the same.

```kotlin
@RunWith(RobolectricTestRunner::class)
@GraphicsMode(GraphicsMode.Mode.NATIVE)
@Config(sdk = [35], qualifiers = "w360dp-h640dp-xxhdpi")
class LoginFormTest {
    @Test
    fun loginForm() {
        val context = ApplicationProvider.getApplicationContext<Context>()
        val view = LoginFormView(context)
        Mitame.capture(view, "login_form", mapOf("theme" to "light"), widthPx = 1080, heightPx = 720)
    }
}
```

Compose views go through the `mitame-compose` module, which depends on `mitame` and `androidx.compose.ui:ui-test-junit4`, so the base library stays free of Compose. With a `createComposeRule()` in the test, `composeTestRule.captureMitame(name, variant)` captures the root node, and `onNodeWithTag(...).captureMitame(...)` captures a subtree; both use `captureToImage()` and record the node's density as `scale`. The Compose BOM in use requires `compileSdk = 37` for `mitame-compose` and for modules that use it; the base `mitame` library compiles against 36.

```kotlin
@get:Rule
val composeTestRule = createComposeRule()

@Test
fun loginFormCompose() {
    composeTestRule.setContent { LoginFormComposable(dark = false) }
    composeTestRule.captureMitame("login_form_compose", mapOf("theme" to "light"))
}
```

The group defaults to the test class name in snake case (`login_form_test`, with acronyms split as in `login_form_ui_test`); pass `group =` to override. The sidecar records the density as `scale`, the API level as `env.sdk`, and `robolectric` as the renderer when it detects it. Gradle test workers inherit the environment, so `MITAME_OUTPUT_DIR` and `MITAME_PROFILE` work as they do for Flutter; the example's build script defaults the output directory to the repository root (keeping an inherited `MITAME_OUTPUT_DIR` when one is set) because a Gradle module's working directory is the module itself. Gradle skips a test task whose inputs did not change, so the capture command carries `--rerun`; the example's `mitame.toml` sets it, so `mitame run` alone is enough:

```sh
mitame run    # [capture] command = ["./gradlew", ":example:testDebugUnitTest", "--rerun"]
```

Snapshot builds are published to the Central Portal snapshot repository; to try one, add `maven("https://central.sonatype.com/repository/maven-snapshots/")` to the repositories and depend on `io.github.mataku:mitame-android:<version>-SNAPSHOT` (and `mitame-android-compose` for Compose), where the version is the `VERSION_NAME` in `adapters/android/gradle.properties`.

See also the [Android example](../adapters/android/example/README.md).
