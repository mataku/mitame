# CI

## Whether one baseline serves the developer machine and CI

Screenshots compare across machines only when the same rasterizer draws them.

- Android under Robolectric native graphics: identical pixels on macOS and Linux. One `default` baseline, nothing to configure.
- Flutter with `fonts = "ahem"` (or `MITAME_FONTS=ahem` exported around `mitame run` with binary 0.1.0): identical on the example between macOS and `ubuntu-latest` at the widened colour tolerance the setting brings. One `default` baseline.
- Flutter with `fonts = "real"`: CoreText and FreeType shade glyphs differently, and no tolerance short of hiding one-word changes absorbs it on text-heavy screens. Each rendering platform needs its own profile. Produce the Linux baseline from a Mac by capturing in a container and promoting the result:

  ```sh
  docker run --rm -v "$PWD:/work" -w /work -e MITAME_PROFILE=linux ghcr.io/cirruslabs/flutter:3.41.6 sh -c 'flutter pub get && flutter test'
  mitame compare --update --profile linux
  ```

  Delete `.dart_tool/` before switching between the container and the host, since `flutter pub get` writes absolute SDK paths into it. Local runs keep the `default` profile and never compare against the Linux one.
- iOS: simulator runtimes differ across versions; CI gets its own profile, see `references/ios.md`.

## GitHub Actions job

Install the binary from the release archive after the platform toolchain, run the test command through `mitame run` from the project root, and upload the report. Pin the mitame version so a new release does not change a passing job, and pin the Flutter or JDK version to the one the baseline was captured with, since a toolchain upgrade is exactly the kind of change the baseline is meant to surface. A complete workflow for a Flutter project:

```yaml
name: vrt

on:
  push:
    branches: [main]
  pull_request:

permissions:
  contents: read

env:
  MITAME_VERSION: 0.1.0
  FLUTTER_VERSION: 3.41.6

jobs:
  mitame:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v7
      - uses: subosito/flutter-action@v2
        with:
          flutter-version: ${{ env.FLUTTER_VERSION }}
          channel: stable
          cache: true
      - run: |
          curl -fsSL "https://github.com/mataku/mitame/releases/download/v${MITAME_VERSION}/mitame-x86_64-unknown-linux-musl.tar.gz" | tar xz
          sudo mv mitame-x86_64-unknown-linux-musl/mitame /usr/local/bin/
      - run: flutter pub get
      - run: mitame run
        env:
          MITAME_FONTS: ahem
      - uses: actions/upload-artifact@v7
        if: always()
        with:
          name: mitame-report
          path: .mitame/report
```

The `MITAME_FONTS` line is the workaround for binary 0.1.0, which ignores `[capture] fonts`; drop it once `MITAME_VERSION` names a release that sets the variable itself. Without it, the Linux runner renders real glyphs against an Ahem baseline and reports every entry as `changed`.

For Android, replace the Flutter step with `actions/setup-java@v5` (Temurin, the project's JDK) and run `mitame run -- --no-daemon`; Gradle wrapper and Robolectric downloads are cached by the usual Gradle caching action. For iOS, use a `macos-*` runner, the `mitame-aarch64-apple-darwin.tar.gz` archive, and `mitame run --profile <ci-profile> -- xcodebuild test …` as in `references/ios.md`. Match the action versions to what the project already pins (a project that pins actions by commit SHA keeps doing so), and add `working-directory:` to the `run` steps when the project root is not the repository root.

On an arm64 Linux runner the archive is `mitame-aarch64-unknown-linux-musl.tar.gz`.

`mitame run` exits 0 when nothing changed, 1 when any entry is `changed`, `mismatch`, or fails the `[policy]`, and 2 when the test command or a capture failed; the job fails on 1 and 2, and `if: always()` keeps the report artifact in both cases. The uploaded `report/` opens as a standalone page; `mitame review` on a downloaded artifact regenerates the HTML from `result.json` when only that file is present.

The binary stops at `report/`. Pull request comments and remote baseline storage are not its job; baselines live in git or git-lfs, and a summary comment, if wanted, is a separate action that reads `result.json`.

## Updating a CI-only profile

There is no network path from the binary to a CI baseline, and the report only holds copies of the images it shows, without the sidecars the comparison needs. When a profile is captured only on CI, upload the captures as a second artifact:

```yaml
      - uses: actions/upload-artifact@v7
        if: always()
        with:
          name: mitame-current-linux
          path: .mitame/current
```

To accept a change from such a run, download that artifact into `.mitame/current/` locally (it already contains the `<profile>/` level), run `mitame compare --update --profile <profile>`, review the git diff, and commit. Note this in the setup report so the user knows the manual step exists.
