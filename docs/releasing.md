# Releasing

Releases are cut by GitHub Actions only. A `v<version>` tag builds the binaries for every target, checks that the tag matches the workspace `Cargo.toml`, `adapters/flutter/pubspec.yaml`, and `adapters/android/gradle.properties`, and publishes the archives with checksums using the matching `## <version>` section of `CHANGELOG.md` as the release notes; a missing section fails the version check before anything is built. A `flutter-v<version>` tag publishes `mitame_flutter` to pub.dev through pub.dev's automated publishing (configure the repository and the `flutter-v{{version}}` tag pattern on the package's admin page first). Swift Package Manager consumers use the `v<version>` tag directly. The Android modules are published to Maven Central by running `publish-android.yml` from the Actions tab: it checks that `VERSION_NAME` in `adapters/android/gradle.properties` is not a snapshot and equals the workspace version, publishes both modules to Maven Local as a pre-flight, uploads them with the `com.vanniktech.maven.publish` plugin using the Central Portal token and in-memory GPG key held in the `ORG_GRADLE_PROJECT_MAVENCENTRALUSERNAME`, `ORG_GRADLE_PROJECT_MAVENCENTRALPASSWORD`, `ORG_GRADLE_PROJECT_SIGNINGINMEMORYKEY`, `ORG_GRADLE_PROJECT_SIGNINGINMEMORYKEYID`, and `ORG_GRADLE_PROJECT_SIGNINGINMEMORYKEYPASSWORD` secrets, then tags the commit `android-v<version>`. The upload lands as a deployment on the Central Portal that has to be released from its Deployments page; the `dry-run` input stops before the upload and the tag, and `tag: false` uploads without tagging, which is how the secrets are verified against a deployment that is then dropped on the portal. Running `release.yml` manually from the Actions tab (`workflow_dispatch`) builds every target without publishing, which is how the matrix is verified before a tag is pushed.

## Development

```sh
cargo build
cargo test
cd adapters/flutter/example && ../../../target/debug/mitame run
```

The workspace pins its Rust toolchain in `rust-toolchain.toml`. `bench/README.md` describes the benchmark harness that produced the numbers in the README.
