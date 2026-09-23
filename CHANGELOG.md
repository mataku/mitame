# Changelog

Sections are keyed by version; `release.yml` publishes the section matching the pushed `v<version>` tag as the release notes.

## 0.2.0

- Release binaries are about a third smaller (1.45 MB instead of 2.33 MB on macOS arm64): the release profile optimizes for size with `opt-level = "z"` and `panic = "abort"`, while the PNG decoding and diff crates keep `opt-level = 3`, so a 120-screenshot comparison takes 0.29 s instead of 0.25 s with byte-identical results. A panic now aborts the process (exit status 134) instead of unwinding (exit code 101).
- `mitame_flutter` bundles the binary and runs it through `dart run mitame_flutter:mitame`, so Flutter projects need no separate install; see `adapters/flutter/CHANGELOG.md`.
- Release archives are built for `aarch64-apple-darwin`, `x86_64-unknown-linux-musl`, and `aarch64-unknown-linux-musl` only. The Intel macOS and Windows archives are no longer published, since neither can be verified during development; `cargo install` still builds from source on those platforms.
- Sidecars written for an older `schema_version` are read instead of reported as `error`, so a future schema bump does not break committed baselines; only the fields the binary uses are checked. A sidecar for a newer `schema_version` is still an error, and it is now reported as such even when its fields were renamed, instead of as a JSON parse error.
- `mitame compare --update` drops `captured_at` and `ext.<platform>.run_id` from the sidecars it writes into the baseline, keeping the adapter's key order, so an updated screenshot no longer rewrites its sidecar on every run. Existing baseline sidecars lose those two fields the next time their screenshot is updated; nothing needs to be regenerated.
- A sidecar that cannot be parsed, or that names another identity or schema version, is now reported as `error` in every case. Before, an unparseable current sidecar was ignored for `added` screenshots (and copied into the baseline by `--update`), and a baseline sidecar was only checked after the byte-equality and dimension checks, so a bad baseline sidecar surfaced as `unchanged` or `mismatch`.
- `result.json` entries with differing pixels carry `diff_bounds` (`x`, `y`, `width`, `height` in image pixels), the bounding box the diff image already outlines, so an agent can crop to the region instead of reading a full-size screenshot. Additive; `schema_version` stays 1.
- `mitame compare | head` no longer panics with `Broken pipe` when the reader closes stdout early. Write errors on stdout are ignored, so a `--update` run still writes the baseline and the exit code still reflects the comparison.
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
