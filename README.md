# mitame

Visual regression testing for Flutter. A single Rust binary compares screenshots against a committed baseline, produces diff images and a machine-readable result, and manages baseline approval. A thin capture adapter writes PNG files into a fixed layout and nothing else.

The Flutter adapter depends on `flutter_test` from the SDK and on no third-party package. It replaces the golden file comparator so that existing `matchesGoldenFile` calls become the capture step. The test process never fails on a visual difference; `mitame compare` does, once per suite, in parallel.

## Why mitame

Flutter already has golden tests, and there are Dart packages that build on them. mitame exists because of what those leave unsolved.

- **Real text on CI.** Stock goldens require an exact pixel match, and macOS and Linux rasterize fonts differently, so goldens rendered locally fail on CI. Existing packages work around this by rendering text as Ahem placeholder boxes on CI, which means CI never checks real text. mitame compares with a per-pixel tolerance and keeps a separate baseline per profile, so CI verifies the same glyphs a user sees.
- **Review instead of assert.** A visual difference does not fail `flutter test`. `mitame compare` reports every changed screenshot with a diff image, and `mitame approve` promotes the ones you accept. This is the workflow of hosted tools like Percy or reg-suit, without a hosted service and without a Node toolchain in a Flutter repository.
- **A dependency that does not rot.** Dart golden packages pull in their own dependency trees and break, or are discontinued, when the SDK moves. The mitame adapter is a few hundred lines that depend on `flutter_test` alone, so the only thing it tracks is the Flutter SDK. The comparison logic lives in a single static binary that has no relationship to your `pubspec.lock`.
- **Comparison outside the test process.** The test isolate only encodes and writes a PNG. Decoding, diffing, and reporting happen once per suite in Rust, in parallel across all screenshots, and never block a test file.
- **Baseline as data, not test fixtures.** Screenshots live under `.mitame/baseline/` instead of scattered `goldens/` directories next to tests, which makes them easy to put on git-lfs or an object store and to review as a set.

## Status

Early development. The end-to-end slice works for Flutter: capture from `flutter test`, `mitame compare` with an HTML report, `mitame approve`, and the `mitame test` wrapper. See [Roadmap](#roadmap) for what is not built yet.

## Layout

```
.mitame/
├── baseline/<profile>/<platform>/<group>/<name>[__<variant>].{png,json}   # committed
├── current/<profile>/...                                                  # written by adapters, ignored
└── report/{result.json,diff/...}                                          # written by mitame compare, ignored
```

The identity of a screenshot is `<platform>/<group>/<name>[__<variant>]`. Variants are `key=value` pairs, sorted by key and joined with `,`. Every component is restricted to `[a-z0-9_.-]`. The optional JSON sidecar next to each PNG carries descriptive metadata; its schema is in `schema/sidecar.schema.json`, and the schema of `report/result.json` is in `schema/result.schema.json`.

Adapters read two environment variables: `MITAME_OUTPUT_DIR` (default `<cwd>/.mitame/current`) and `MITAME_PROFILE` (default `default`). Profiles separate baselines that differ systematically, such as macOS and Linux font rasterization.

## Flutter

`test/flutter_test_config.dart`:

```dart
import 'dart:async';
import 'package:mitame_flutter/mitame_flutter.dart';

Future<void> testExecutable(FutureOr<void> Function() testMain) async {
  await Mitame.install(loadFonts: true);
  await testMain();
}
```

`loadFonts: true` loads the fonts declared in the package and the SDK's Roboto, so goldens render real glyphs instead of the Ahem placeholder font.

```dart
for (final variant in Mitame.matrix({'theme': ['light', 'dark'], 'locale': ['ja', 'en']})) {
  await tester.pumpWidget(buildApp(variant));
  await tester.pumpAndSettle();
  await expectLater(find.byType(LoginForm), matchesGoldenFile(Mitame.golden('login_form', variant)));
}
```

Then:

```sh
flutter test
mitame compare      # exit 0: no differences, 1: differences or policy failure, 2: error
mitame approve      # promote current into baseline
```

`mitame test [flutter test args]` runs both steps in one command for local feedback: it sets the output directory and profile, runs `flutter test`, then compares. mitame's own flags such as `--profile` go before the `flutter test` arguments. `compare` writes `.mitame/report/index.html` alongside `result.json`. The report directory is self-contained (it holds copies of the baseline and current images it shows), so uploading `.mitame/report/` as a CI artifact is enough to review a run. `mitame report` regenerates the HTML from an existing `result.json`.

## Configuration

`mitame.toml` in the working directory, all keys optional:

```toml
[paths]
root = ".mitame"

[compare]
threshold = 0.001          # max diff ratio that still counts as unchanged
pixel_tolerance = 0.1      # per-pixel YIQ tolerance, as in pixelmatch
anti_aliasing = true       # accepted, not yet applied

[policy]
added = "warn"             # pass | warn | fail
removed = "warn"
mismatch = "fail"

[[rules]]
match = "flutter/**/*__*theme=dark*"
threshold = 0.01
```

Rules use `globset` semantics and the last matching rule wins.

## Roadmap

Only Flutter is supported today. The contract is capture-agnostic, so iOS and Android are planned as additional capture adapters that write PNGs into the same layout and reuse `mitame compare` and `mitame approve` unchanged.

### CLI

- [x] `compare` with byte-equality shortcut, pixel tolerance, dimension and scale checks
- [x] `approve` for all or selected identities
- [x] `result.json` and diff images
- [x] HTML report
- [ ] anti-aliasing detection (config key accepted, not applied)
- [x] `mitame test`: run `flutter test` then `compare` in one command for local feedback
- [ ] skip sidecar copy in `approve` when the PNG is unchanged
- [ ] remote baseline storage (S3 / GCS) and PR comments
- [ ] prebuilt binaries on GitHub Releases (workflow in place, unpublished until the first tag)
- [ ] GitHub Action to install the binary
- [ ] Windows builds

### Flutter

- [x] `mitame_flutter` adapter depending on `flutter_test` only
- [x] real fonts via `FontManifest.json` and the SDK's Roboto
- [ ] publish to pub.dev (needs automated publishing set up on pub.dev)
- [ ] benchmark against stock `LocalFileComparator`
- [ ] adapter for `@Preview`-based capture output

### iOS

Not started. Planned as an XCTest shim that renders a view to a PNG and writes it with `platform: ios` under `MITAME_OUTPUT_DIR`, in the spirit of swift-snapshot-testing. Full-screen capture through `xcrun simctl io screenshot` is a possible second tier.

- [ ] XCTest shim writing PNG and sidecar
- [ ] example project
- [ ] scale handling for 2x / 3x devices

### Android

Not started. Planned as a Robolectric or instrumented-test shim that renders a View or Composable to a PNG and writes it with `platform: android`, in the spirit of Roborazzi. Full-screen capture through `adb exec-out screencap` is a possible second tier.

- [ ] Robolectric shim writing PNG and sidecar
- [ ] example project
- [ ] density handling (`mdpi` … `xxxhdpi`)

## Install

Prebuilt binaries are published on GitHub Releases for macOS (arm64, x64) and Linux (x64, arm64, statically linked with musl). Pick the archive for your platform:

```sh
curl -fsSL https://github.com/mataku/mitame/releases/latest/download/mitame-aarch64-apple-darwin.tar.gz | tar xz
sudo mv mitame-aarch64-apple-darwin/mitame /usr/local/bin/
```

Archive names are `mitame-<target>.tar.gz` where `<target>` is one of `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`. Each archive has a matching `.sha256` file.

To build from source instead:

```sh
cargo install --git https://github.com/mataku/mitame mitame-cli
```

The Flutter adapter is not on pub.dev yet. Until then, reference it as a git dependency:

```yaml
dev_dependencies:
  mitame_flutter:
    git:
      url: https://github.com/mataku/mitame
      path: adapters/flutter
```

## Development

```sh
cargo build
cargo test
cd adapters/flutter/example && flutter test && ../../../target/debug/mitame compare
```

The workspace pins its Rust toolchain in `rust-toolchain.toml`.
