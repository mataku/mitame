# mitame_flutter

Capture adapter for [mitame](https://github.com/mataku/mitame) (見た目, pronounced /mitame/, "mee-tah-meh"), a visual regression testing tool with a single Rust binary for comparison, reporting, and baseline updates.

This package depends on `flutter_test` only. It replaces `goldenFileComparator` so that every existing `matchesGoldenFile` call writes its PNG (and a JSON sidecar) into `.mitame/current/` instead of comparing in the test process. `mitame compare` then does the comparison once per suite with per-pixel tolerance and anti-aliasing detection, writes an HTML report, and `mitame compare --update` writes accepted screenshots into `.mitame/baseline/`.

## Setup

Add the package as a dev dependency with `flutter pub add --dev mitame_flutter`, then replace the comparator in `test/flutter_test_config.dart`:

```dart
import 'dart:async';
import 'package:mitame_flutter/mitame_flutter.dart';

Future<void> testExecutable(FutureOr<void> Function() testMain) async {
  await Mitame.install(loadFonts: true);
  await testMain();
}
```

Then run `dart run mitame_flutter:mitame run` (or `flutter test` followed by `dart run mitame_flutter:mitame compare`); with `mitame_flutter` 0.1.0, run `mitame run` (or `mitame compare`) instead. See the repository README for the binary, configuration, profiles, and CI usage.

## Running mitame

Versions of this package newer than 0.1.0 bundle the mitame binary for macOS arm64 and Linux (x64, arm64). Run it from the project root with `dart run mitame_flutter:mitame`, for example `dart run mitame_flutter:mitame init` and `dart run mitame_flutter:mitame run`; the binary version is pinned by `pubspec.lock` together with this package. On other platforms, install the binary from source (`cargo install --git https://github.com/mataku/mitame mitame-cli`) and set `MITAME_BINARY` to its path; the launcher then runs that binary instead. With 0.1.0, install the binary the same way and run `mitame` directly, with no `MITAME_BINARY`.

## API

- `Mitame.install({loadFonts, groupFromGoldenUri})` replaces the comparator; `loadFonts: true` loads the fonts declared in `pubspec.yaml` and the SDK's Roboto so goldens render real glyphs.
- `Mitame.golden(name, variant)` builds a golden file name that encodes a variant map such as `{'theme': 'dark'}`.
- `Mitame.matrix({...})` expands a map of axes into variant combinations.

Environment variables read by the adapter: `MITAME_OUTPUT_DIR`, `MITAME_PROFILE`, `MITAME_FONTS`, `MITAME_RUN_ID`.
