# Benchmark harness

`mitame-bench` generates a throwaway Flutter package under `bench/fixture/` with N golden tests, then measures wall time for five configurations:

1. no goldens: compile, VM startup, and `pumpWidget` only
2. stock `LocalFileComparator`, every golden byte-identical (the `listEquals` shortcut)
3. stock `LocalFileComparator`, every golden differs by one pixel (full decode and pixel loop, tests fail)
4. mitame comparator plus `mitame compare`, every golden identical
5. mitame comparator plus `mitame compare`, every golden differs by one pixel

```sh
cargo build --release -p mitame-cli
cargo run -p mitame-bench -- --goldens 200 --size 390x844 --jobs 4 --runs 3
```

Each configuration is run `--runs` times and the median is reported. The fixture is regenerated on every invocation unless `--keep` is passed, and its `mitame.toml` sets `threshold = 0.0` so every differing golden counts as `changed`. Results depend on the machine.

## Results (2026-09-21)

Apple Silicon laptop with 12 cores, Flutter 3.41.6, mitame built with `--release`, median of 3 runs. The fixture draws a 12×12 grid of rounded rectangles plus a text label per golden; the "differ" configurations change one pixel per golden so every one is `changed` and gets a diff image. The mitame columns are `flutter test` time plus `mitame compare` time.

| goldens, size, jobs | no goldens | stock identical | stock differ | mitame identical | mitame differ |
| --- | ---: | ---: | ---: | ---: | ---: |
| 200, 390×844, -j 4 | 4.22 s | 4.81 s | 7.16 s | 4.81 + 0.02 s | 4.82 + 0.14 s |
| 200, 390×844, -j 12 | 3.13 s | 3.37 s | 5.09 s | 3.45 + 0.02 s | 3.39 + 0.14 s |
| 200, 1170×2532, -j 4 | 4.24 s | 7.45 s | 23.01 s | 7.53 + 0.02 s | 7.51 + 0.53 s |
| 1000, 390×844, -j 4 | 17.58 s | 19.74 s | 31.48 s | 20.08 + 0.09 s | 20.13 + 0.68 s |

Identical goldens cost the same as stock: writing a PNG and a sidecar equals reading a golden and comparing bytes. When every golden differs, the comparison moves out of the test process and shrinks: at phone size stock adds 2.9 s over the no-golden baseline for 200 goldens and mitame adds 0.6 s; at 3x device size stock adds 18.8 s and mitame 3.8 s, of which 0.5 s is the Rust comparison and the rest is PNG encoding both paths share. On one Flutter app, a Flutter upgrade left 24 goldens at 8 to 9 differing pixels each under stock and at 0 under mitame's tolerance and anti-aliasing detection, while one-word changes of 0.04% to 0.09% were still reported.
