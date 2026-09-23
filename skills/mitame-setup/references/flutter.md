# Flutter adapter

The adapter is `mitame_flutter` on pub.dev and depends on `flutter_test` only.

## Install

```sh
flutter pub add --dev mitame_flutter
```

`mitame_flutter` newer than 0.1.0 bundles the mitame binary for macOS arm64 and Linux (x64, arm64). Run every mitame command through it from the project root (`dart run mitame_flutter:mitame init`, `dart run mitame_flutter:mitame run`, and so on); no separate binary install is needed. On another platform, or with `mitame_flutter` 0.1.0, install the binary from source and set `MITAME_BINARY` to its path.

Then install the comparator once per test run in `test/flutter_test_config.dart`, creating the file when the project has none (`flutter_test` loads it automatically from the `test/` directory):

```dart
import 'dart:async';

import 'package:mitame_flutter/mitame_flutter.dart';

Future<void> testExecutable(FutureOr<void> Function() testMain) async {
  await Mitame.install(loadFonts: true);
  await testMain();
}
```

When the file already exists, keep the project's other setup and add the `Mitame.install` call before `testMain()`. Remove a previous library's comparator or font loading from the same file (`loadAppFonts()` from golden_toolkit, `goldenFileComparator = ...` assignments); `Mitame.install` replaces `goldenFileComparator` itself and loads the fonts declared in the package plus the SDK's Roboto when `loadFonts` is true. Under `fonts = "ahem"` the binary sets `MITAME_FONTS=ahem` for the capture and the adapter skips font loading regardless of `loadFonts`. Binary 0.1.0 does not know the `fonts` key and does not set the variable; with that binary, export `MITAME_FONTS=ahem` yourself around `mitame run`, as the setup decision in `SKILL.md` describes.

## What changes for existing tests

Nothing in the test files. Every existing `matchesGoldenFile` call now writes a capture instead of comparing, and `expectLater(..., matchesGoldenFile(...))` never fails on pixels. Keep the existing golden files in the repository until the first baseline is committed, then delete them; they are no longer read.

To seed the baseline from the existing goldens instead of from a fresh capture, copy each golden to its identity path under `.mitame/baseline/default/flutter/<group>/<stem>.png`, where `<group>` is the test file's directory relative to `test/` (a test in `test/ui/auth/` has group `ui/auth`, a test directly in `test/` has no group segment). A copied golden that was rendered by another library or with other fonts will show as `changed` on the first run, so a fresh capture through `mitame run` followed by `mitame compare --update` is usually the better seed; the old goldens remain in git history for comparison.

## Identity and variants

The identity of `matchesGoldenFile('goldens/login.png')` in `test/ui/auth/` is `flutter/ui/auth/login`; the golden directory (`goldens/`, `goldens/ci/`) is dropped because helper packages use fixed names that carry no information. Two goldens with the same stem in one test directory throw a `StateError` naming the identity; rename one, or pass `groupFromGoldenUri: true` to `Mitame.install` to keep the directory in the identity.

Encode variants through the helpers so the report groups them under one name:

```dart
for (final variant in Mitame.matrix({'theme': ['light', 'dark'], 'locale': ['ja', 'en']})) {
  await tester.pumpWidget(buildApp(variant));
  await tester.pumpAndSettle();
  await expectLater(find.byType(LoginForm), matchesGoldenFile(Mitame.golden('login_form', variant)));
}
```

`Mitame.golden(name, variant)` returns a golden path whose stem is `name__key=value,key=value`, which the adapter decodes into the variant map.

## Flutter SDK selection

When `[capture] command` starts with `flutter`, `mitame capture` and `mitame run` pick the binary in this order: `--flutter <path>`, `MITAME_FLUTTER`, the version named in `.fvmrc` under the fvm cache, `.fvm/flutter_sdk/bin/flutter`, then `flutter` on `PATH`, and print the chosen path as `using …`. A project on fvm therefore needs no wrapper in the capture command. mitame's own flags such as `--profile` go before the `--` that introduces extra `flutter test` arguments.

## Things that produce a surprising first run

- A widget with an animation or a `Timer` that is still running at capture time differs between runs; pump to a settled frame before the golden call.
- `pumpAndSettle` with an infinite animation times out; use `pump(Duration)` to a known frame instead.
- Packages that load fonts themselves (alchemist does) ignore `MITAME_FONTS=ahem`, so their goldens render real glyphs and need a per-platform profile.
- Running `flutter test` directly instead of `mitame run` leaves earlier captures in `.mitame/current/`; delete that directory first, or use `mitame run`, which clears it.
