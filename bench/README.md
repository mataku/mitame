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

Each configuration is run `--runs` times and the median is reported. The fixture is regenerated on every invocation unless `--keep` is passed, and its `mitame.toml` sets `threshold = 0.0` so every differing golden counts as `changed`. Results depend on the machine; see `docs/spec/use-cases.md` for the numbers behind the README claims.
