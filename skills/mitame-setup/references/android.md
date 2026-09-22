# Android adapter

Two modules on Maven Central: `io.github.mataku:mitame-android` depends on the Android SDK only and captures a `View` or a `Bitmap`; `io.github.mataku:mitame-android-compose` adds `captureMitame` on a compose rule or semantics node and depends on `androidx.compose.ui:ui-test-junit4`. Both are test dependencies of the module that holds the screenshot tests.

## Install

```kotlin
dependencies {
    testImplementation("io.github.mataku:mitame-android:0.1.0")
    testImplementation("io.github.mataku:mitame-android-compose:0.1.0")
}
```

Leave the Compose module out when the project does not use Compose. The Compose module's BOM requires `compileSdk = 37` for the modules that use it; the base module compiles against 36.

## Module build script

Robolectric with native graphics renders real pixels without an emulator, and it renders the same pixels on macOS and Linux, so one committed baseline serves local runs and CI. The module needs Android resources in unit tests and, because Gradle test workers start in the module directory rather than the repository root, an explicit output directory:

```kotlin
android {
    testOptions {
        unitTests.isIncludeAndroidResources = true
    }
}

tasks.withType<Test>().configureEach {
    environment("MITAME_OUTPUT_DIR", System.getenv("MITAME_OUTPUT_DIR") ?: rootProject.file(".mitame/current").absolutePath)
}
```

The `System.getenv` fallback keeps an inherited value, which is how `mitame run` passes its own output directory through. Without this block, captures land under the module directory and `mitame compare` at the root finds none, reporting every baseline entry as removed.

Add Robolectric and the test core to the test dependencies when they are not there yet (`org.robolectric:robolectric`, `androidx.test:core`), and `debugImplementation("androidx.compose.ui:ui-test-manifest")` for Compose rules.

## Capture from a test

A `View`:

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

`@GraphicsMode(NATIVE)` is what makes Robolectric draw; without it the bitmap is blank. `@Config(qualifiers = "…xxhdpi")` fixes the density, which the sidecar records as `scale`, so a change of density shows up as `mismatch` rather than as a silent size change. Pass `widthPx` and `heightPx` when the view has not been laid out; the adapter measures and lays it out at that size.

A Compose screen:

```kotlin
@get:Rule
val composeTestRule = createComposeRule()

@Test
fun loginFormCompose() {
    composeTestRule.setContent { LoginForm(dark = false) }
    composeTestRule.captureMitame("login_form_compose", mapOf("theme" to "light"))
}
```

`composeTestRule.captureMitame` captures the root node; `onNodeWithTag("...").captureMitame(...)` captures a subtree. The group defaults to the test class name in snake case (`LoginFormTest` becomes `login_form_test`); pass `group =` to override. `Mitame.write(bitmap, name, variant, scale = …)` takes a bitmap from any other renderer, so a project on Paparazzi or an instrumented test can hand its bitmap over without changing how it is produced.

## Capture command

`mitame init` detects `["./gradlew", "test", "--rerun"]`, which runs every test task in every module. Narrow it to the module and variant that hold the screenshot tests:

```toml
[capture]
command = ["./gradlew", ":app:testDebugUnitTest", "--rerun"]
```

`--rerun` stays. Gradle skips a test task whose inputs did not change, and a skipped task writes no captures, which the comparison then reports as every screenshot removed. On CI, `mitame run -- --no-daemon` appends the flag to the configured command.

## Things that produce a surprising first run

- A test class without `@GraphicsMode(GraphicsMode.Mode.NATIVE)` captures a blank or black image.
- A mismatch of `sdk` between local runs and CI changes rendering; pin it in `@Config`.
- `MITAME_OUTPUT_DIR` set to a relative path is resolved against the worker's directory; the build script above uses an absolute path.
- Robolectric downloads the Android runtime jar on first use; a sandboxed or offline CI runner needs it cached or pre-fetched.
