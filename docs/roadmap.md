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
- [x] `mitame review [id]` opens the report in the browser
- [x] sidecar `schema_version` check and `mitame_version` in `result.json`
- [x] prebuilt binaries on GitHub Releases (`v0.1.0`, five targets) and a Homebrew tap (`mataku/tap/mitame`)
- [ ] Windows builds (in the release matrix, never run; `capture` and `run` spawn without a shell, so `.bat` launchers must be named explicitly)
- [ ] warn when the baseline and current sidecars disagree on `env.os` while `fonts = "real"` on Flutter, the silent failure of comparing real-font captures against the wrong profile; Ahem and Robolectric baselines are shared across macOS and Linux on purpose, so the warning must not fire for them
- [x] `diff_bounds` in `result.json` records the differing region's bounding box (the diff image already outlines it) so an agent can crop instead of reading a full-size screenshot
- [ ] `compare --update --from <dir>` to apply a downloaded CI capture (for example the `linux` profile's `current/` from an artifact) to the local baseline, so the CI profile has an update path without the binary touching the network
- [ ] a setup check (`mitame doctor` or similar): `.gitignore` entries, adapter present, Flutter resolution, profile
- [x] `mitame compare | head` no longer panics on a closed stdout; stdout write errors are ignored so `--update` still completes
- [ ] decide whether `captured_at` belongs in the committed baseline sidecar; every `--update` rewrites the sidecar because of it, doubling the files in a baseline diff

## Integrations

- [ ] `mitame-report` GitHub Action that summarizes `result.json` as a pull request comment (separate repository, scaffolded locally, no remote yet; removed from docs/ci.md until it exists)
- [x] agent skills under `skills/`: `mitame-setup` for adopting mitame in a project and `mitame-review` for running it, reading `result.json`, inspecting diff images, updating only intended ids, never loosening thresholds, never `--prune` after a partial run
- [ ] automate the Homebrew formula bump from `release.yml` (needs a token for the tap repository)

## Flutter

- [x] `mitame_flutter` adapter depending on `flutter_test` only
- [x] real fonts via `FontManifest.json` and the SDK's Roboto
- [x] benchmark against stock `LocalFileComparator` (`mitame-bench`)
- [x] `mitame_flutter` 0.1.0 on pub.dev; later versions publish from `flutter-v*` tags
- [x] one baseline for macOS and Linux through `fonts = "ahem"` (0 px between macOS and Linux on the example, one-word change 1728 px)

## iOS

- [x] `Mitame` Swift package writing PNG and sidecar from a `UIView` or `UIImage`
- [x] example XCTest target verified on the iPhone 16 simulator
- [x] scale recorded from the render (`UIScreen.main.scale`, 3.0 on iPhone 16)
- [x] root `Package.swift` so the package can be referenced from this repository's URL
- [x] SwiftUI capture through `UIHostingController`
- [x] size inference when `size:` is omitted
- [ ] full-screen tier through `xcrun simctl io screenshot`
- [ ] Swift Package Index listing, for discoverability and platform badges (optional)

## Android

- [x] `mitame` Android library writing PNG and sidecar from a `View` or `Bitmap`
- [x] example module verified with Robolectric native graphics, no emulator
- [x] density recorded as `scale` (3.0 under `xxhdpi`)
- [x] Compose helper (`mitame-compose` module, `captureMitame` on a compose rule or semantics node)
- [x] Robolectric renders identically on macOS and Linux (0 px on the example), so one baseline serves local runs and CI
- [x] `io.github.mataku:mitame-android` and `mitame-android-compose` 0.1.0 on Maven Central; later versions publish from `android-v*` tags and are released by hand on the portal, snapshots through `snapshot-android.yml`
- [ ] instrumented-test tier (device or emulator, files pulled with adb)

## Later

- [ ] a `setup` GitHub Action in its own repository, if the curl step in [CI](ci.md) proves too repetitive
- [ ] Flutter: adapter for `@Preview`-based capture output, pending Flutter's own previewer gaining a capture command
