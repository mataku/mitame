# Releasing

Releases are cut by GitHub Actions only. A `v<version>` tag builds the binaries for every target, checks that the tag matches the workspace `Cargo.toml`, `adapters/flutter/pubspec.yaml`, and `adapters/android/gradle.properties`, and publishes the archives with checksums. A `flutter-v<version>` tag publishes `mitame_flutter` to pub.dev through pub.dev's automated publishing (configure the repository and the `flutter-v{{version}}` tag pattern on the package's admin page first). Swift Package Manager consumers use the `v<version>` tag directly. Running `release.yml` manually from the Actions tab (`workflow_dispatch`) builds every target without publishing, which is how the matrix is verified before a tag is pushed.

## Development

```sh
cargo build
cargo test
cd adapters/flutter/example && ../../../target/debug/mitame run
```

The workspace pins its Rust toolchain in `rust-toolchain.toml`. `bench/README.md` describes the benchmark harness that produced the numbers in the README.
