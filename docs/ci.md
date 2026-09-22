# CI

## One baseline for macOS and Linux

Screenshots compare across machines only when the same rasterizer draws them. Measured on the examples in this repository, a macOS host against `ubuntu-latest`:

- **Android.** Robolectric's native graphics bundle Android's own Skia and fonts, so a test renders the same pixels on macOS and Linux (0 differing pixels on the example). One committed baseline serves local runs and CI with no configuration.
- **Flutter with Ahem.** `fonts = "ahem"` under `[capture]` in `mitame.toml` makes `mitame capture` and `mitame run` set `MITAME_FONTS=ahem`, so the adapter skips font loading and every glyph renders as a box. Box edges still shade slightly differently on the two platforms (66 to 228 px per screen at the default colour tolerance), so `fonts = "ahem"` also widens the per-pixel colour tolerance from 0.1 to 0.2 of the black-to-white distance, which brings the example to 0 differing pixels while a one-word change still measures 1728 px (0.36%). `mitame init` writes the setting for a Flutter project. The trade-offs: real glyphs are never checked, Ahem boxes are wider than most fonts and can overflow tight layouts, and packages that load fonts themselves (alchemist does) ignore the switch.
- **Flutter with real fonts.** CoreText on macOS and FreeType on Linux shade glyphs differently: 207 px (0.04%) on a form with a few labels, 3% on a text-heavy widget, and no tolerance short of hiding one-word changes absorbs the second. Each rendering platform then needs its own baseline, selected with `--profile` or `MITAME_PROFILE`. To produce a Linux baseline from a Mac, capture in a Linux container with the project mounted (Rancher Desktop, Docker, or Podman all work), then promote it:

  ```sh
  nerdctl run --rm -v "$PWD:/work" -w /work -e MITAME_PROFILE=linux \
    ghcr.io/cirruslabs/flutter:3.41.6 sh -c 'flutter pub get && flutter test'
  mitame compare --update --profile linux
  ```

  Delete `.dart_tool/` before switching between the container and the host, since `flutter pub get` writes absolute SDK paths into it. Local runs keep the `default` profile and never compare against the Linux baseline.
- **iOS.** Simulator runtimes render differently across versions, so CI's simulator gets its own profile unless it matches the local one exactly.

## Workflow

A minimal GitHub Actions job installs the binary from the release archive, runs the test command through `mitame run`, and uploads the report. Pin the version so a new release does not change a passing job under you.

```yaml
permissions:
  contents: read

env:
  MITAME_VERSION: 0.1.0

steps:
  - run: |
      curl -fsSL "https://github.com/mataku/mitame/releases/download/v${MITAME_VERSION}/mitame-x86_64-unknown-linux-musl.tar.gz" | tar xz
      sudo mv mitame-x86_64-unknown-linux-musl/mitame /usr/local/bin/
  - run: mitame run
  - uses: actions/upload-artifact@v4
    if: always()
    with:
      name: mitame-report
      path: .mitame/report
```

`compare` exits 1 when anything changed, which fails the job; the uploaded `report/` opens as a standalone page. A pull request comment that summarizes `result.json` is planned as a separate composite action and will be documented here once it is published. The binary itself stops at `report/`: it does not talk to object storage or the GitHub API, and baselines live in git (plain or git-lfs).

This repository's `ci.yml` runs the Flutter and Android examples on `ubuntu-latest` against their committed baselines, so a rendering change fails the job, and the iOS example on `macos-26` under a `ci-macos` profile with no committed baseline, which validates the pipeline without guarding regressions.
