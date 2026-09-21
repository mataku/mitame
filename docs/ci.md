# CI

## Baselines per rendering platform

Glyphs are rasterized by CoreText on macOS and by FreeType on Linux, so the same widget never produces identical pixels on both. Layout, line breaks, and glyph positions do match; only the shading inside and around glyphs differs. Measured on the example with the default tolerance and anti-aliasing detection, a form with a few labels differs by 207 pixels (0.04%) and a text-heavy 360×640 widget by 3%; both count as `changed` under the strict default, and no tolerance short of hiding real changes would absorb the second. Ahem does not fix this either: box edges still differ by a few hundred pixels per screen.

The reliable setup is therefore one baseline per rendering platform, selected by `MITAME_PROFILE`. Developers on macOS keep a `default` baseline for local runs, and CI compares against a `linux` baseline that was itself rendered on Linux. To produce or refresh the Linux baseline from a Mac, run the capture in a Linux container with the repository mounted (Rancher Desktop, Docker, or Podman all work):

```sh
nerdctl run --rm -v "$PWD:/work" -w /work -e MITAME_PROFILE=linux \
  ghcr.io/cirruslabs/flutter:3.41.6 sh -c 'flutter pub get && flutter test'
mitame approve --profile linux
```

Delete `.dart_tool/` before switching between the container and the host, since `flutter pub get` writes absolute SDK paths into it. The `linux` baseline is committed like any other file; it can equally be produced by running the same commands plus `mitame approve --profile linux` in a workflow and committing the result.

## Workflow

A minimal GitHub Actions job installs the binary, captures with the `linux` profile, compares, and uploads the report. Building from source with `cargo install` works today but takes a few minutes per run; a lighter install step from the release archives is TBA once the first release exists.

```yaml
permissions:
  contents: read

steps:
  - run: cargo install --git https://github.com/mataku/mitame mitame-cli
  - run: flutter test
    env:
      MITAME_PROFILE: linux
  - run: mitame compare --profile linux
  - uses: actions/upload-artifact@v4
    if: always()
    with:
      name: mitame-report
      path: .mitame/report
```

`compare` exits 1 when anything changed, which fails the job; the uploaded `report/` opens as a standalone page. A pull request comment that summarizes `result.json` is planned as a separate composite action and will be documented here once it is published. The binary itself stops at `report/`: it does not talk to object storage or the GitHub API, and baselines live in git (plain or git-lfs). Local runs keep using the `default` profile and never compare against the Linux baseline.

This repository's `ci.yml` runs the same shape for the Android example (Robolectric on `ubuntu-latest`, profile `linux`) and the iOS example (simulator on `macos-26`, profile `ci-macos`); with no committed baseline for those profiles they report every screenshot as `added`, which validates the pipeline without guarding regressions.
