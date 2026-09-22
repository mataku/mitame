# Flutter

The adapter is [`mitame_flutter`](https://pub.dev/packages/mitame_flutter) on pub.dev; add it with `flutter pub add --dev mitame_flutter`.

`loadFonts: true` loads the fonts declared in the package and the SDK's Roboto, so goldens render real glyphs instead of the Ahem placeholder font. Setting `MITAME_FONTS=ahem` in the environment skips font loading for that run, which renders text as Ahem boxes. Ahem glyphs are wider than real ones and can overflow tight layouts, and packages that load fonts themselves (alchemist does) are unaffected by the switch.

By default the identity is the test file's directory plus the golden file's stem, so `matchesGoldenFile('goldens/login.png')` in `test/ui/auth/` becomes `flutter/ui/auth/login`. Golden helper packages use fixed directory names such as `goldens/` or `goldens/ci/` that carry no information. If two golden files in one test directory share a stem, the adapter throws a `StateError` naming the identity when both are written in the same `mitame run`; pass `groupFromGoldenUri: true` to keep the golden directory in the identity instead.

When the capture command starts with `flutter`, `mitame capture` and `mitame run` pick the Flutter binary from `--flutter <path>`, then `MITAME_FLUTTER`, then the version named in `.fvmrc` if it exists under the fvm cache (`FVM_CACHE_PATH` or `~/fvm`), then `.fvm/flutter_sdk/bin/flutter`, then `flutter` on `PATH`, and prints the chosen path as `using …`. mitame's own flags such as `--profile` go before the `--` that introduces `flutter test` arguments.

See also the [Flutter example](../adapters/flutter/example/README.md).
