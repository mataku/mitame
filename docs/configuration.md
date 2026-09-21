# Configuration

`mitame init` writes a `mitame.toml` with the defaults into the current directory (it refuses to overwrite one unless `--force` is passed). All keys are optional:

```toml
[paths]
root = ".mitame"

[compare]
threshold = 0.0            # allowed diff ratio (differing pixels / total pixels)
max_diff_pixels = 0        # allowed differing pixels, whichever allowance is larger applies
pixel_tolerance = 0.1      # per-pixel YIQ tolerance, as in pixelmatch
anti_aliasing = true       # ignore pixels that only differ by anti-aliasing

[policy]
added = "warn"             # pass | warn | fail
removed = "warn"
mismatch = "fail"

[[rules]]
match = "flutter/**/*__*theme=dark*"
max_diff_pixels = 40
```

The binary looks for `mitame.toml` in the current directory and then in each parent, and the first directory holding `mitame.toml` or a committed `.mitame/baseline/` becomes the project root (the current directory also counts when it holds any `.mitame/`, so a stray report directory left in a parent is never picked up); `paths.root` is resolved against it, so `mitame compare` works from any subdirectory of the project. `--config <path>` skips the search and uses that file's directory, and `--root <path>` overrides the layout directory outright. Rules use `globset` semantics and the last matching rule wins. A rule may override `threshold`, `max_diff_pixels`, `pixel_tolerance`, and `anti_aliasing`.

The default is strict on purpose: `pixel_tolerance` and `anti_aliasing` already remove rendering noise (re-running an unchanged suite on the same machine yields 0 differing pixels), and a ratio threshold hides real changes on full-screen goldens, where a one-word change is 200 to 300 pixels out of 400,000. Raise `max_diff_pixels` or `threshold` only for identities that need it. A screenshot is `changed` when its differing pixels exceed the larger of `max_diff_pixels` and `threshold` × total pixels. Anti-aliasing detection follows pixelmatch: a differing pixel is ignored when it sits on an edge in one image and its darker or lighter neighbour has many identical siblings in both images. Ignored pixels are drawn in yellow in the diff image.
