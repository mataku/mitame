# AGENTS.md

Instructions for coding agents working in this repository. Humans: see `README.md` and `docs/`.

## What this is

mitame is a visual regression testing tool: a Rust binary (`crates/`) that compares PNG screenshots against a committed baseline and writes a report, plus thin capture adapters for Flutter (`adapters/flutter`), Android (`adapters/android`), and iOS (`adapters/ios`) that only write PNGs and JSON sidecars into `.mitame/current/`. The file layout and sidecar format are the contract between them; `schema/*.json` is generated from `crates/mitame-contract`.

## Boundaries that must hold

- The binary stops at `.mitame/report/`. It never talks to object storage, the GitHub API, or any hosted service. Uploading and pull request comments are CI concerns.
- Each adapter depends only on its platform SDK: `flutter_test` for Flutter, the Android SDK for `adapters/android/mitame` (Compose support lives in the separate `mitame-compose` module), UIKit/SwiftUI/Foundation for iOS. Do not add third-party packages to an adapter. The `com.vanniktech.maven.publish` Gradle plugin is a build-time exception that only drives Maven Central publishing.
- Adapters read exactly `MITAME_OUTPUT_DIR`, `MITAME_PROFILE`, `MITAME_RUN_ID`, and (Flutter only) `MITAME_FONTS`. New behavior is configured on the binary side (`mitame.toml`), not by adding adapter inputs.
- Changing the sidecar or `result.json` shape means updating `crates/mitame-contract`, regenerating `schema/` with `cargo run -p mitame-cli -- schema`, and mirroring the change in all three adapters. Additive fields do not bump `schema_version`; renames and removals do.
- The binary keeps reading every older sidecar `schema_version`; it reads only `id`, `image.scale`, and `captured_at`, so renaming or removing one of those needs a per-version reader in `crates/mitame-core/src/sidecar.rs`.
- `skills/` holds the consumer-facing agent skills (`mitame-setup`, `mitame-review`). They describe CLI flags, `mitame.toml` keys, adapter APIs, and exit codes for the versions in their frontmatter, so a change to any of those updates the skill in the same commit, and a release bumps the versions in the frontmatter. Skills must stay self-contained (relative links only within the skill directory), since installers copy the skill directory alone.
- Comparison defaults stay strict (`threshold = 0.0`, `max_diff_pixels = 0`). Do not loosen them to make an example pass; adjust the example or add a `[[rules]]` entry for the one identity that needs it.
- Baselines under `adapters/*/example/.mitame/baseline/` (and `adapters/ios/.mitame/baseline/`) are committed fixtures. The Flutter example's is rendered with Ahem (`fonts = "ahem"`) and the Android example's by Robolectric, so both match on macOS and on Linux CI; the iOS one is rendered on macOS with the iPhone 16 / iOS 18.3.1 simulator. Regenerate them with `mitame run --update` only when the example or its capture settings change, and say so in the commit body.

## Build and verify

```sh
cargo build && cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --all -- --check
cd adapters/flutter && dart analyze && dart format --set-exit-if-changed -o none lib example/lib example/test
cd adapters/flutter/example && ../../../target/debug/mitame run
cd adapters/android && ../../target/debug/mitame run
cd adapters/ios && ../../target/debug/mitame run -- xcodebuild test -scheme Mitame -destination 'platform=iOS Simulator,name=iPhone 16,OS=18.3.1'
```

- The Rust toolchain is pinned in `rust-toolchain.toml`. If `cargo` on `PATH` is a toolchain binary rather than the rustup proxy, the pin is ignored and dependencies fail to parse; run through `rustup run <version> cargo` or put the pinned toolchain's `bin` first on `PATH`.
- `mitame capture` and `mitame run` clear `.mitame/current/<profile>/` before running the test command from `[capture] command` in `mitame.toml` (the Flutter and Android examples ship one; iOS passes the command after `--`). If you call `flutter test`, Gradle, or xcodebuild directly, delete that directory first or stale captures will be compared.
- Gradle skips an up-to-date test task, so the capture command always carries `--rerun`.
- xcodebuild forwards environment variables to tests only with a `TEST_RUNNER_` prefix; `mitame run` sets both forms. A simulator name that exists for several OS versions must be qualified with `OS=`; `xcodebuild -showdestinations -scheme Mitame` lists valid ones.
- A change to the report is verified by rendering it: run the Flutter example with a visible change (edit the title in `lib/login_form.dart`), open the report with `mitame review`, and read the diff image. Do not report report changes as done from unit tests alone.
- Every example must pass its gate before docs are updated: files land at the documented identity, the sidecar `id` matches the path, `run --update` then `run` exits 0, and a visible change exits 1.

## Conventions

- No code comments in Rust, Dart, Kotlin, or Swift, including doc comments. Explain in commit messages and `docs/` instead.
- Documentation is English, one paragraph per line, never hard-wrapped. Design notes under `docs/spec/` are intentionally untracked scratch material; user-facing docs live in `README.md` and `docs/*.md`.
- Keep `README.md` short (why, install, quick start, examples, links). Details go to `docs/`.
- Identities are `<platform>/<group>/<name>[__<variant>]` restricted to `[a-z0-9_.-]`; adapters normalize names (CamelCase becomes snake_case, acronyms split as `swift_ui`). Keep the three normalizers equivalent.
- Commits follow Conventional Commits with atomic scope; a change to default behavior gets a `!` and a `BREAKING CHANGE` paragraph that tells users what to regenerate.

## Releases and publishing

- Releases, tags, and package publishing happen only through GitHub Actions (`release.yml` on `v*`, `publish-flutter.yml` on `flutter-v*`, `publish-android.yml` on `android-v*`; `snapshot-android.yml` by manual dispatch for `-SNAPSHOT` versions). `release.yml` also accepts `workflow_dispatch`, which builds every target without publishing; use it to verify the matrix before a tag. Never run `git tag`, `gh release`, `cargo publish`, `dart pub publish`, or Maven publishing locally.
- A `v*` tag must match the version in `Cargo.toml` and `adapters/flutter/pubspec.yaml`, and `CHANGELOG.md` must have a `## <version>` section, which becomes the release notes; `VERSION_NAME` in `adapters/android/gradle.properties` must be that version or its `-SNAPSHOT` (it stays a snapshot between Android releases and is set to the plain version only for the `android-v*` tag, then bumped to the next `-SNAPSHOT`). Bump the files and add the section together.
- `v0.1.0` was released on 2026-09-22 (binaries, Swift package) alongside `mitame_flutter` 0.1.0 on pub.dev; version bumps are the maintainer's call. From here on, a change to default behavior gets a `!` and a BREAKING CHANGE paragraph.
- `mitame-report` (the pull request comment action) is a separate repository and is not a dependency of anything here.
