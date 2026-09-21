# mitame

Visual regression testing for Flutter, Android, and iOS with one binary and one report. A single Rust binary compares screenshots against a committed baseline, produces diff images and a machine-readable result, and manages baseline approval. Thin per-platform capture adapters, each a few hundred lines that depend on the platform SDK alone, write PNG files into a fixed layout and nothing else.

The Flutter adapter replaces the golden file comparator so that existing `matchesGoldenFile` calls become the capture step; the Android and iOS adapters take a `View` or `UIView` (or a ready bitmap) from any test framework. The test process never fails on a visual difference; `mitame compare` does, once per suite, in parallel.

## Why mitame

Flutter already has golden tests, and Android and iOS have mature snapshot libraries. mitame exists because of what those leave unsolved.

- **Upgrades stop invalidating every golden.** `flutter_test` compares pixels exactly, so an engine or SDK update that shifts a few pixels of anti-aliasing fails every golden at once, and the only remedies are to regenerate them all without knowing whether a real change slipped in, or to render text as Ahem boxes on CI and never check real text. mitame's per-pixel tolerance and anti-aliasing detection separate that noise from change (a real project's 24 goldens went from 8 to 9 differing pixels each under stock to 0 under mitame), while every remaining pixel counts, so a one-word change is never hidden. Systematic differences between rendering platforms, such as macOS against Linux, are not absorbed; they get a baseline per profile instead.
- **Review instead of assert.** A visual difference does not fail the test run. `mitame compare` reports every changed screenshot with a diff image in a self-contained HTML page, and `mitame approve` promotes the ones you accept. Baselines live under `.mitame/baseline/` as one reviewable set rather than scattered next to tests. This is the workflow of hosted tools like Percy or reg-suit, without a hosted service and without a Node toolchain in a mobile repository.
- **One pipeline for Flutter, Android, and iOS.** The adapters share a file contract, so the same tolerance, anti-aliasing detection, profiles, approval flow, and report page apply to all three. The Android and iOS adapters were added without changing the binary, and a single `compare` run reports every platform on one page.
- **Dependencies that do not rot.** Golden helper packages pull in their own dependency trees and break, or are discontinued, when the SDK moves. Each mitame adapter depends only on its platform's SDK (`flutter_test`, the Android SDK, UIKit), so the only thing it tracks is that SDK, and the comparison logic lives in a static binary with no relationship to your lockfiles.
- **Comparison outside the test process.** The test isolate only encodes and writes a PNG; decoding, diffing, and reporting happen once per suite in Rust, in parallel, and never block a test file. Unchanged goldens cost the same as stock; when every golden differs, the comparison overhead for 200 phone-size goldens drops from 2.9 s to 0.7 s, and from 18.8 s to 3.8 s at 3x device size (`mitame-bench`, 4 jobs, Apple Silicon). Rendering and PNG encoding inside the test runner are unchanged and remain the larger share of total time.

### When mitame is not the right tool

If you work on a single native platform and already rely on Roborazzi's Compose and Preview integration or swift-snapshot-testing's SwiftUI and non-image strategies, mitame's adapters give you less: they render a view to a bitmap and stop. The gain is for teams that mix Flutter with native modules, or that want one visual regression pipeline and one report across repositories.

## Status

Early development, no release cut yet. The end-to-end flow works for Flutter (capture from `flutter test`, `mitame compare` with an HTML report, `mitame approve`, the `mitame test` wrapper) and for the Android and iOS example projects through `mitame run`. The adapters are not yet published as packages. See [Roadmap](#roadmap).

## Install

Prebuilt binaries will be published on GitHub Releases for macOS (arm64, x64), Linux (x64, arm64, statically linked with musl), and Windows (x64); until the first release, build from source:

```sh
cargo install --git https://github.com/mataku/mitame mitame-cli
```

Once released, archives are named `mitame-<target>.tar.gz` with `<target>` one of `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`, `x86_64-pc-windows-msvc`, each with a `.sha256` file:

```sh
curl -fsSL https://github.com/mataku/mitame/releases/latest/download/mitame-aarch64-apple-darwin.tar.gz | tar xz
sudo mv mitame-aarch64-apple-darwin/mitame /usr/local/bin/
```

On GitHub Actions, the `setup` action in this repository downloads the right archive, verifies its checksum, and adds `mitame` to `PATH`:

```yaml
- uses: mataku/mitame/setup@v0.1.0
  with:
    version: v0.1.0
```

Adapters:

- Flutter: not on pub.dev yet; reference it as a git dependency.

  ```yaml
  dev_dependencies:
    mitame_flutter:
      git:
        url: https://github.com/mataku/mitame
        path: adapters/flutter
  ```

- iOS: a Swift package exposed through the repository's root `Package.swift`.

  ```swift
  .package(url: "https://github.com/mataku/mitame", from: "0.1.0")
  ```

- Android: set up for Maven publishing as `io.github.mataku:mitame-android` but not on Maven Central yet; `./gradlew :mitame:publishToMavenLocal` in `adapters/android` installs it into `~/.m2` for `mavenLocal()` consumers.

## Quick start (Flutter)

Add the adapter, then replace the comparator in `test/flutter_test_config.dart`:

```dart
import 'dart:async';
import 'package:mitame_flutter/mitame_flutter.dart';

Future<void> testExecutable(FutureOr<void> Function() testMain) async {
  await Mitame.install(loadFonts: true);
  await testMain();
}
```

Existing `matchesGoldenFile` calls now capture instead of compare. To capture variants, encode them in the golden name:

```dart
for (final variant in Mitame.matrix({'theme': ['light', 'dark'], 'locale': ['ja', 'en']})) {
  await tester.pumpWidget(buildApp(variant));
  await tester.pumpAndSettle();
  await expectLater(find.byType(LoginForm), matchesGoldenFile(Mitame.golden('login_form', variant)));
}
```

Run, review, approve:

```sh
mitame test           # runs flutter test, then compares; exit 0 clean, 1 differences, 2 error
open .mitame/report/index.html
mitame approve        # promote accepted captures into .mitame/baseline/
```

Add `.mitame/current/` and `.mitame/report/` to `.gitignore` and commit `.mitame/baseline/`. Existing goldens can be moved into the baseline by copying: the adapter writes exactly the bytes stock `flutter_test` would have written, and `flutter test --update-goldens` also writes into `.mitame/current/` once the adapter is installed.

## How it fits together

```
.mitame/
├── baseline/<profile>/<platform>/<group>/<name>[__<variant>].{png,json}   # committed
├── current/<profile>/...                                                  # written by adapters, ignored
└── report/{result.json,index.html,baseline/,current/,diff/}               # written by mitame compare, ignored
```

The identity of a screenshot is `<platform>/<group>/<name>[__<variant>]`. Variants are `key=value` pairs, sorted by key and joined with `,`. Every component is restricted to `[a-z0-9_.-]`. The optional JSON sidecar next to each PNG carries descriptive metadata (dimensions, scale, SDK version, capture time); its schema is in `schema/sidecar.schema.json`, and the schema of `report/result.json` is in `schema/result.schema.json`.

Adapters read four environment variables: `MITAME_OUTPUT_DIR` (default `<cwd>/.mitame/current`), `MITAME_PROFILE` (default `default`), `MITAME_FONTS` (Flutter only; `ahem` skips font loading), and `MITAME_RUN_ID` (set by `mitame run` and `mitame test`, used to detect identity collisions within a run). Profiles separate baselines that differ systematically, such as macOS and Linux font rasterization.

`mitame run [--profile <p>] [--keep-current] -- <command...>` is the generic wrapper: it clears `.mitame/current/<profile>/`, sets the variables above (plus `TEST_RUNNER_`-prefixed copies for xcodebuild), runs the command, then compares and writes the report. `mitame test [flutter test args]` is the Flutter form of the same thing. When the command fails, the comparison still runs on the captures that succeeded and the report is written, but the exit code is 2. Pass `--keep-current` to keep earlier captures, for example when running a subset of test files, in which case the tests that did not run appear as `removed` (a warning by default). If you run the test command directly instead, delete `.mitame/current/` first so captures from an earlier run are not compared as this run's.

`mitame compare` writes `.mitame/report/index.html` next to `result.json`. The report directory is self-contained (it holds copies of the baseline and current images it shows), so uploading `.mitame/report/` as a CI artifact is enough to review a run. Every entry shows its pixel count and capture time; entries with any differing pixel show the three images with the differing region outlined on the diff. Clicking an image opens a viewer with baseline, current, diff, and an onion-skin overlay with adjustable opacity, 100% and 200% zoom, arrow-key navigation, and direct links of the form `index.html#view=<id>`. The search box filters entries by id. `mitame report` regenerates the HTML from an existing `result.json`.

## Flutter details

`loadFonts: true` loads the fonts declared in the package and the SDK's Roboto, so goldens render real glyphs instead of the Ahem placeholder font. Setting `MITAME_FONTS=ahem` in the environment skips font loading for that run, which renders text as Ahem boxes. Ahem glyphs are wider than real ones and can overflow tight layouts, and packages that load fonts themselves (alchemist does) are unaffected by the switch.

By default the identity is the test file's directory plus the golden file's stem, so `matchesGoldenFile('goldens/login.png')` in `test/ui/auth/` becomes `flutter/ui/auth/login`. Golden helper packages use fixed directory names such as `goldens/` or `goldens/ci/` that carry no information. If two golden files in one test directory share a stem, the adapter throws a `StateError` naming the identity when both are written in the same `mitame test` run; pass `groupFromGoldenUri: true` to keep the golden directory in the identity instead.

`mitame test` picks the Flutter binary from `--flutter <path>`, then `MITAME_FLUTTER`, then the version named in `.fvmrc` if it exists under the fvm cache (`FVM_CACHE_PATH` or `~/fvm`), then `.fvm/flutter_sdk/bin/flutter`, then `flutter` on `PATH`, and prints the chosen path as `using …`. mitame's own flags such as `--profile` go before the `flutter test` arguments.

## Android

`adapters/android/mitame` is an Android library that depends on the Android SDK only. It takes a `View` or a `Bitmap` and writes the PNG and sidecar; how the bitmap is produced is the project's choice. The example module renders with Robolectric's native graphics, which needs no emulator, but Paparazzi or an instrumented test can hand `Mitame.write` a bitmap just the same.

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

The group defaults to the test class name in snake case (`login_form_test`); pass `group =` to override. The sidecar records the density as `scale`, the API level as `env.sdk`, and `robolectric` as the renderer when it detects it. Gradle test workers inherit the environment, so `MITAME_OUTPUT_DIR` and `MITAME_PROFILE` work as they do for Flutter; the example's build script defaults the output directory to the repository root (keeping an inherited `MITAME_OUTPUT_DIR` when one is set) because a Gradle module's working directory is the module itself. Gradle skips a test task whose inputs did not change, so capture with `--rerun`:

```sh
mitame run -- ./gradlew :example:testDebugUnitTest --rerun
```

## iOS

`adapters/ios` is a Swift package whose `Mitame` library depends on UIKit and Foundation only. `Mitame.capture` lays a `UIView` out at the requested size, renders it through `layer.render(in:)` at the screen scale, and writes the PNG and sidecar; `Mitame.write` takes a ready `UIImage`. The example test target runs on the iOS simulator through `xcodebuild test`.

```swift
@MainActor
final class LoginFormTests: XCTestCase {
    func testLoginForm() throws {
        let view = LoginFormView(dark: false)
        try Mitame.capture(view, name: "login_form", variant: ["theme": "light"], size: CGSize(width: 360, height: 240))
    }
}
```

The group defaults to the test file's stem in snake case (`login_form_tests`); pass `group:` to override. The sidecar records the render scale, the iOS version as `env.sdk`, and the simulator name as `env.device`. XCTest processes only see environment variables that xcodebuild receives with a `TEST_RUNNER_` prefix; `mitame run` sets both forms, so:

```sh
mitame run -- xcodebuild test -scheme Mitame -destination 'platform=iOS Simulator,name=iPhone 16,OS=18.3.1'
```

Without `mitame run`, export `TEST_RUNNER_MITAME_OUTPUT_DIR` (an absolute path) and `TEST_RUNNER_MITAME_PROFILE` yourself. `xcodebuild -showdestinations -scheme Mitame` lists the simulator names and OS versions valid on a machine. A name alone is rejected when several runtimes have that device, so include `OS=<version>` (or `OS=latest` on CI).

## Configuration

`mitame.toml` in the working directory, all keys optional:

```toml
[paths]
root = ".mitame"

[compare]
threshold = 0.0            # allowed diff ratio (differing pixels / total pixels)
max_diff_pixels = 0        # allowed differing pixels, whichever allowance is larger applies
pixel_tolerance = 0.1      # per-pixel YIQ tolerance, as in pixelmatch
anti_aliasing = true       # ignore pixels that only differ by anti-aliasing

[policy]
added = "warn"             # pass | warn | fail
removed = "warn"
mismatch = "fail"

[[rules]]
match = "flutter/**/*__*theme=dark*"
max_diff_pixels = 40
```

Rules use `globset` semantics and the last matching rule wins. A rule may override `threshold`, `max_diff_pixels`, `pixel_tolerance`, and `anti_aliasing`.

The default is strict on purpose: `pixel_tolerance` and `anti_aliasing` already remove rendering noise (re-running an unchanged suite on the same machine yields 0 differing pixels), and a ratio threshold hides real changes on full-screen goldens, where a one-word change is 200 to 300 pixels out of 400,000. Raise `max_diff_pixels` or `threshold` only for identities that need it. A screenshot is `changed` when its differing pixels exceed the larger of `max_diff_pixels` and `threshold` × total pixels. Anti-aliasing detection follows pixelmatch: a differing pixel is ignored when it sits on an edge in one image and its darker or lighter neighbour has many identical siblings in both images. Ignored pixels are drawn in yellow in the diff image.

## CI

### Baselines per rendering platform

Glyphs are rasterized by CoreText on macOS and by FreeType on Linux, so the same widget never produces identical pixels on both. Layout, line breaks, and glyph positions do match; only the shading inside and around glyphs differs. Measured on the example with the default tolerance and anti-aliasing detection, a form with a few labels differs by 207 pixels (0.04%) and a text-heavy 360×640 widget by 3%; both count as `changed` under the strict default, and no tolerance short of hiding real changes would absorb the second. Ahem does not fix this either: box edges still differ by a few hundred pixels per screen.

The reliable setup is therefore one baseline per rendering platform, selected by `MITAME_PROFILE`. Developers on macOS keep a `default` baseline for local runs, and CI compares against a `linux` baseline that was itself rendered on Linux. To produce or refresh the Linux baseline from a Mac, run the capture in a Linux container with the repository mounted (Rancher Desktop, Docker, or Podman all work):

```sh
nerdctl run --rm -v "$PWD:/work" -w /work -e MITAME_PROFILE=linux \
  ghcr.io/cirruslabs/flutter:3.41.6 sh -c 'flutter pub get && flutter test'
mitame approve --profile linux
```

Delete `.dart_tool/` before switching between the container and the host, since `flutter pub get` writes absolute SDK paths into it. The `linux` baseline is committed like any other file; it can equally be produced by running the same commands plus `mitame approve --profile linux` in a workflow and committing the result.

### Workflow

A minimal GitHub Actions job captures with the `linux` profile, compares, uploads the report, and posts the summary to the pull request:

```yaml
permissions:
  contents: read
  pull-requests: write

steps:
  - uses: mataku/mitame/setup@v0.1.0
  - run: flutter test
    env:
      MITAME_PROFILE: linux
  - run: mitame compare --profile linux
  - uses: actions/upload-artifact@v4
    if: always()
    with:
      name: mitame-report
      path: .mitame/report
  - uses: mataku/mitame-report@v1
    if: always() && github.event_name == 'pull_request'
```

`compare` exits 1 when anything changed, which fails the job; the uploaded `report/` opens as a standalone page. [mitame-report](https://github.com/mataku/mitame-report) is a separate composite action, not yet published, that reads `result.json` and keeps one sticky comment per job up to date, linking to the workflow run for the images. The binary itself stops at `report/`: it does not talk to object storage or the GitHub API, and baselines live in git (plain or git-lfs). Local runs keep using the `default` profile and never compare against the Linux baseline.

This repository's `ci.yml` runs the same shape for the Android example (Robolectric on `ubuntu-latest`, profile `linux`) and the iOS example (simulator on `macos-latest`, profile `ci-macos`); with no committed baseline for those profiles they report every screenshot as `added`, which validates the pipeline without guarding regressions.

## Roadmap

### CLI

- [x] `compare` with byte-equality shortcut, pixel tolerance, dimension and scale checks
- [x] `approve` for all or selected identities
- [x] `result.json` and diff images
- [x] HTML report with viewer (zoom, onion-skin overlay, keyboard navigation) and id filter
- [x] anti-aliasing detection
- [x] `mitame test` and `mitame run -- <command>`
- [x] skip sidecar copy in `approve` when the PNG is unchanged
- [ ] prebuilt binaries on GitHub Releases (workflow in place, unpublished until the first tag)
- [ ] GitHub Action to install the binary (`setup/action.yml` in place, unverified until the first release)
- [ ] Windows builds (in the release matrix, unverified)

### Integrations

- [ ] `mitame-report` GitHub Action that summarizes `result.json` as a pull request comment (separate repository, scaffolded but not published yet)

### Flutter

- [x] `mitame_flutter` adapter depending on `flutter_test` only
- [x] real fonts via `FontManifest.json` and the SDK's Roboto
- [x] benchmark against stock `LocalFileComparator` (`mitame-bench`)
- [ ] publish to pub.dev (`publish-flutter.yml` runs on `flutter-v*` tags once automated publishing is enabled for this repository on pub.dev)
- [ ] adapter for `@Preview`-based capture output

### iOS

- [x] `Mitame` Swift package writing PNG and sidecar from a `UIView` or `UIImage`
- [x] example XCTest target verified on the iPhone 16 simulator
- [x] scale recorded from the render (`UIScreen.main.scale`, 3.0 on iPhone 16)
- [x] root `Package.swift` so the package can be referenced from this repository's URL
- [ ] SwiftUI helper (`UIHostingController` wrapper)
- [ ] size inference when `size:` is omitted (containers report `noIntrinsicMetric` today)
- [ ] full-screen tier through `xcrun simctl io screenshot`

### Android

- [x] `mitame` Android library writing PNG and sidecar from a `View` or `Bitmap`
- [x] example module verified with Robolectric native graphics, no emulator
- [x] density recorded as `scale` (3.0 under `xxhdpi`)
- [ ] Compose helper
- [ ] publish to Maven Central (`maven-publish` configured; `publishToMavenLocal` works)
- [ ] instrumented-test tier (device or emulator, files pulled with adb)

## Releasing

Releases are cut by GitHub Actions only. A `v<version>` tag builds the binaries for every target, checks that the tag matches the workspace `Cargo.toml`, `adapters/flutter/pubspec.yaml`, and `adapters/android/mitame/build.gradle.kts`, and publishes the archives with checksums. A `flutter-v<version>` tag publishes `mitame_flutter` to pub.dev through pub.dev's automated publishing (configure the repository and the `flutter-v{{version}}` tag pattern on the package's admin page first). Swift Package Manager consumers use the `v<version>` tag directly.

## Development

```sh
cargo build
cargo test
cd adapters/flutter/example && ../../../target/debug/mitame test
```

The workspace pins its Rust toolchain in `rust-toolchain.toml`. `bench/README.md` describes the benchmark harness that produced the numbers above.
