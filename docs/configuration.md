# Configuration

`mitame init` writes a `mitame.toml` with the defaults and the test command detected from the project (`flutter test` next to a `pubspec.yaml`, `./gradlew test --rerun` next to a Gradle settings file) into the current directory, and for a Flutter project sets `fonts = "ahem"` so one baseline serves macOS and Linux (see [CI](ci.md)); it refuses to overwrite one unless `--force` is passed. All keys are optional:

```toml
[paths]
root = ".mitame"

[capture]
command = ["flutter", "test"]   # run by `mitame capture` and `mitame run`; arguments after `--` are appended
fonts = "ahem"                  # Flutter only: "ahem" sets MITAME_FONTS=ahem for the capture, "real" (the default) loads the app's fonts

[compare]
threshold = 0.0            # allowed diff ratio (differing pixels / total pixels)
max_diff_pixels = 0        # allowed differing pixels, whichever allowance is larger applies
anti_aliasing = true       # ignore pixels that only differ by anti-aliasing

[policy]
added = "warn"             # pass | warn | fail
removed = "warn"
mismatch = "fail"

[[rules]]
match = "flutter/**/*__*theme=dark*"
max_diff_pixels = 40
```

The binary looks for `mitame.toml` in the current directory and then in each parent, and the first directory holding `mitame.toml` or a committed `.mitame/baseline/` becomes the project root (the current directory also counts when it holds any `.mitame/`, so a stray report directory left in a parent is never picked up); `paths.root` is resolved against it, so `mitame compare` works from any subdirectory of the project. `--config <path>` skips the search and uses that file's directory, and `--root <path>` overrides the layout directory outright. Rules use `globset` semantics and the last matching rule wins. A rule may override `threshold`, `max_diff_pixels`, and `anti_aliasing`.

The default is strict on purpose: a per-pixel colour tolerance and `anti_aliasing` already remove rendering noise (re-running an unchanged suite on the same machine yields 0 differing pixels), and a ratio threshold hides real changes on full-screen goldens, where a one-word change is 200 to 300 pixels out of 400,000. Raise `max_diff_pixels` or `threshold` only for identities that need it. A screenshot is `changed` when its differing pixels exceed the larger of `max_diff_pixels` and `threshold` × total pixels. Anti-aliasing detection follows pixelmatch: a differing pixel is ignored when it sits on an edge in one image and its darker or lighter neighbour has many identical siblings in both images. Ignored pixels are drawn in yellow in the diff image.

The colour tolerance is not a setting you normally touch. Two pixels count as different when their YIQ distance exceeds 0.1 of the black-to-white distance, or 0.2 when `fonts = "ahem"`, which is where Ahem box edges rendered by CoreText and FreeType still differ; `pixel_tolerance` under `[compare]` or in a rule overrides that number for the rare case that needs it.
