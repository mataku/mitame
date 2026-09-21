# Roadmap

## CLI

- [x] `compare` with byte-equality shortcut, pixel tolerance, dimension and scale checks
- [x] `compare --update` writes changed and added screenshots into the baseline from the comparison result; `--prune` deletes removed ones
- [x] `result.json` and diff images
- [x] HTML report with viewer (zoom, onion-skin overlay, keyboard navigation) and id filter
- [x] anti-aliasing detection
- [x] `mitame capture` and `mitame run` around the test command from `mitame.toml`
- [x] `--update` leaves screenshots that are unchanged within the tolerance alone, so the baseline diff shows only real changes
- [x] `mitame init` and project root discovery from subdirectories
- [x] sidecar `schema_version` check and `mitame_version` in `result.json`
- [ ] prebuilt binaries on GitHub Releases (workflow in place, unpublished until the first tag)
- [ ] Windows builds (in the release matrix, unverified)

## Integrations

- [ ] `mitame-report` GitHub Action that summarizes `result.json` as a pull request comment (separate repository, scaffolded but not published yet)

## Flutter

- [x] `mitame_flutter` adapter depending on `flutter_test` only
- [x] real fonts via `FontManifest.json` and the SDK's Roboto
- [x] benchmark against stock `LocalFileComparator` (`mitame-bench`)
- [ ] publish to pub.dev (`publish-flutter.yml` runs on `flutter-v*` tags once automated publishing is enabled for this repository on pub.dev)

## iOS

- [x] `Mitame` Swift package writing PNG and sidecar from a `UIView` or `UIImage`
- [x] example XCTest target verified on the iPhone 16 simulator
- [x] scale recorded from the render (`UIScreen.main.scale`, 3.0 on iPhone 16)
- [x] root `Package.swift` so the package can be referenced from this repository's URL
- [x] SwiftUI capture through `UIHostingController`
- [x] size inference when `size:` is omitted
- [ ] full-screen tier through `xcrun simctl io screenshot`

## Android

- [x] `mitame` Android library writing PNG and sidecar from a `View` or `Bitmap`
- [x] example module verified with Robolectric native graphics, no emulator
- [x] density recorded as `scale` (3.0 under `xxhdpi`)
- [x] Compose helper (`mitame-compose` module, `captureMitame` on a compose rule or semantics node)
- [ ] publish to Maven Central (`maven-publish` configured for both modules; `publishToMavenLocal` works)
- [ ] instrumented-test tier (device or emulator, files pulled with adb)

## After the first release

- [ ] a `setup` GitHub Action in its own repository, if the curl step in [CI](ci.md) proves too repetitive
- [ ] Flutter: adapter for `@Preview`-based capture output, pending Flutter's own previewer gaining a capture command
