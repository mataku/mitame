# Android example

A library module with Robolectric tests that capture a `View` (two themes) and a Compose screen. It lives inside the adapter's Gradle project, so it depends on `project(":mitame")` and `project(":mitame-compose")`; a standalone project would use the Maven coordinates `io.github.mataku:mitame-android` and `io.github.mataku:mitame-android-compose` instead (after `publishToMavenLocal`, until they are on Maven Central).

What to copy:

- `build.gradle.kts`: `testOptions.unitTests.isIncludeAndroidResources = true` for Robolectric, the Compose plugin and `compileSdk = 37` only because this module uses Compose, and the `tasks.withType<Test>` block that points `MITAME_OUTPUT_DIR` at the repository root (Gradle test workers start in the module directory) while keeping an inherited value.
- `src/test/.../LoginFormTest.kt`: `@GraphicsMode(GraphicsMode.Mode.NATIVE)` and a `@Config(qualifiers = "…xxhdpi")` so Robolectric renders real pixels at a known density, then `Mitame.capture(view, name, variant, widthPx, heightPx)`.
- `src/test/.../LoginFormComposeTest.kt`: `createComposeRule()`, `setContent {}`, then `composeTestRule.captureMitame(name, variant)`.

Run from `adapters/android`:

```sh
../../target/debug/mitame run -- ./gradlew :example:testDebugUnitTest --rerun
open .mitame/report/index.html
../../target/debug/mitame approve
```

`--rerun` matters: Gradle skips the test task when nothing changed, and a skipped task captures nothing.
