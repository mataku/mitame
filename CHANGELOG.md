# Changelog

Sections are keyed by version; `release.yml` publishes the section matching the pushed `v<version>` tag as the release notes.

## Unreleased

- Agent skills under `skills/`: `mitame-setup` guides a coding agent through installing the binary, `mitame init`, the platform adapter, the git rules, the first baseline, and the CI job; `mitame-review` through running `mitame run`, reading `result.json`, and updating the baseline for intended changes only. Installable with `npx skills add mataku/mitame`.
- `[capture] fonts = "ahem"` in `mitame.toml` makes `capture` and `run` set `MITAME_FONTS=ahem` and widens the per-pixel colour tolerance to 0.2; `mitame init` writes it for Flutter projects. Measured on the example, that gives 0 differing pixels between macOS and Linux while a one-word change still registers at 1728 px. `pixel_tolerance` is now optional and only needed to override that derived value.

## 0.1.0

First release.

- `mitame compare` compares PNG screenshots in `.mitame/current/` against `.mitame/baseline/` with a byte-equality shortcut, per-pixel YIQ tolerance, anti-aliasing detection, and dimension and scale checks, and writes `result.json`, diff images, and a self-contained HTML report with a viewer (zoom, onion-skin overlay, keyboard navigation, id filter).
- `mitame compare --update` writes changed, mismatched, and added screenshots into the baseline from the comparison result and leaves screenshots that are unchanged within the tolerance alone; `--prune` also deletes removed ones.
- `mitame capture` runs the test command from `[capture] command` in `mitame.toml` with the capture environment set and `.mitame/current/<profile>/` cleared; `mitame run` is capture followed by compare and writes the report even when the command fails.
- `mitame init` writes `mitame.toml` with the defaults and the test command detected from the project; `mitame review [id]` opens the report in the browser, optionally at one screenshot.
- Commands find the project root from any subdirectory, `result.json` records the binary version, and a sidecar written for another schema version is reported as an error.
- Profiles (`--profile`, `MITAME_PROFILE`) keep one baseline per rendering platform; per-identity `[[rules]]` override thresholds; `[policy]` decides whether added, removed, and mismatched screenshots fail the run.
- Adapters: `mitame_flutter` (depends on `flutter_test` only, replaces the golden comparator, loads real fonts, `Mitame.matrix` for variants), `mitame-android` and `mitame-android-compose` (Android SDK and Compose test only), and the `Mitame` Swift package (UIKit and SwiftUI capture). `mitame_flutter` 0.1.0 is on pub.dev; the Android modules reach Maven Central through their own `android-v0.1.0` publish; the Swift package resolves from the `v0.1.0` tag.
- Prebuilt binaries for macOS (arm64, x64), Linux (x64 and arm64, musl), and Windows (x64, experimental and not exercised by CI).
