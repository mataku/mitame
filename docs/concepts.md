# How it fits together

```
.mitame/
├── baseline/<profile>/<platform>/<group>/<name>[__<variant>].{png,json}   # committed
├── current/<profile>/...                                                  # written by adapters, ignored
└── report/{result.json,index.html,baseline/,current/,diff/}               # written by mitame compare, ignored
```

The identity of a screenshot is `<platform>/<group>/<name>[__<variant>]`. Variants are `key=value` pairs, sorted by key and joined with `,`. Every component is restricted to `[a-z0-9_.-]`. The optional JSON sidecar next to each PNG carries descriptive metadata (dimensions, scale, SDK version, capture time); its schema is in `schema/sidecar.schema.json`, and the schema of `report/result.json` is in `schema/result.schema.json`. Both carry a `schema_version`; a sidecar written for a different schema version than the binary understands is reported as `error` for that screenshot with a message naming both versions, since the binary is installed per machine while adapters are pinned per project. `result.json` also records the binary's version as `mitame_version`.

Adapters read four environment variables: `MITAME_OUTPUT_DIR` (default `<cwd>/.mitame/current`), `MITAME_PROFILE` (default `default`), `MITAME_FONTS` (Flutter only; `ahem` skips font loading), and `MITAME_RUN_ID` (set by `mitame run` and `mitame test`, used to detect identity collisions within a run). Profiles separate baselines that differ systematically, such as macOS and Linux font rasterization.

`mitame run [--profile <p>] [--keep-current] -- <command...>` is the generic wrapper: it clears `.mitame/current/<profile>/`, sets the variables above (plus `TEST_RUNNER_`-prefixed copies for xcodebuild), runs the command, then compares and writes the report. `mitame test [flutter test args]` is the Flutter form of the same thing. When the command fails, the comparison still runs on the captures that succeeded and the report is written, but the exit code is 2. Pass `--keep-current` to keep earlier captures, for example when running a subset of test files, in which case the tests that did not run appear as `removed` (a warning by default). If you run the test command directly instead, delete `.mitame/current/` first so captures from an earlier run are not compared as this run's.

`mitame compare` writes `.mitame/report/index.html` next to `result.json`. The report directory is self-contained (it holds copies of the baseline and current images it shows), so uploading `.mitame/report/` as a CI artifact is enough to review a run. Every entry shows its pixel count and capture time; entries with any differing pixel show the three images with the differing region outlined on the diff. Clicking an image opens a viewer with baseline, current, diff, and an onion-skin overlay with adjustable opacity, 100% and 200% zoom, arrow-key navigation, and direct links of the form `index.html#view=<id>`. The search box filters entries by id. `mitame report` regenerates the HTML from an existing `result.json`.
