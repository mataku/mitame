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
  MITAME_VERSION: 0.2.0

steps:
  - run: |
      curl -fsSL "https://github.com/mataku/mitame/releases/download/v${MITAME_VERSION}/mitame-x86_64-unknown-linux-musl.tar.gz" | tar xz
      sudo mv mitame-x86_64-unknown-linux-musl/mitame /usr/local/bin/
  - run: mitame run
  - uses: actions/upload-artifact@v7
    if: ${{ !cancelled() }}
    with:
      name: mitame-report
      path: .mitame/report
```

`compare` exits 1 when anything changed, which fails the job; the uploaded `report/` opens as a standalone page. The binary itself stops at `report/`: it does not talk to object storage or the GitHub API, and baselines live in git (plain or git-lfs).

With `mitame_flutter` newer than 0.1.0, drop the install step for a Flutter project: after `flutter pub get`, run `dart run mitame_flutter:mitame run`, which uses the binary bundled with the adapter at the version `pubspec.lock` pins.

## Pull request comment

`result.json` is small and stable (its schema is `schema/result.schema.json`), so a pull request comment needs nothing beyond `jq` and `gh`, both preinstalled on GitHub-hosted runners. The step below renders the summary counts and every entry that is not `unchanged` into Markdown, then creates one comment per job and profile and edits that same comment on later pushes, so a pull request never accumulates stale reports. It runs after the report upload and links to the artifact, since the comment carries text only: the diff images stay in `report/`.

```yaml
permissions:
  contents: read
  pull-requests: write

env:
  MITAME_VERSION: 0.2.0

steps:
  - run: |
      curl -fsSL "https://github.com/mataku/mitame/releases/download/v${MITAME_VERSION}/mitame-x86_64-unknown-linux-musl.tar.gz" | tar xz
      sudo mv mitame-x86_64-unknown-linux-musl/mitame /usr/local/bin/
  - run: mitame run
  - id: report
    uses: actions/upload-artifact@v7
    if: ${{ !cancelled() }}
    with:
      name: mitame-report
      path: .mitame/report
  - if: ${{ !cancelled() && github.event_name == 'pull_request' && github.event.pull_request.head.repo.full_name == github.repository }}
    env:
      GH_TOKEN: ${{ github.token }}
      PR_NUMBER: ${{ github.event.pull_request.number }}
      REPORT_URL: ${{ steps.report.outputs.artifact-url }}
    run: |
      result=.mitame/report/result.json
      test -f "$result" || exit 0
      marker="<!-- mitame:${GITHUB_JOB}/$(jq -r .profile "$result") -->"
      body=$(mktemp)
      {
        echo "$marker"
        jq -r '
          def pct: if . == null then "-" else ((. * 100000 | round) / 1000 | tostring) + "%" end;
          def detail:
            if .message != null then .message
            elif .diff_pixels != null then "\(.diff_pixels) px (\(.diff_ratio | pct))"
            else "-" end;
          def rank: {changed: 0, mismatch: 1, error: 2, added: 3, removed: 4}[.status];
          .summary as $s
          | [.results[] | select(.status != "unchanged")] as $rows
          | ($rows | length) as $n
          | "## mitame report (\(.profile))",
            "",
            (if $n == 0 then "No visual changes." else "\($n) screenshot(s) need review." end),
            "",
            "| changed | added | removed | mismatch | error | unchanged |",
            "| ---: | ---: | ---: | ---: | ---: | ---: |",
            "| \($s.changed) | \($s.added) | \($s.removed) | \($s.mismatch) | \($s.error) | \($s.unchanged) |",
            (if $n > 0 then
              "",
              "| status | id | detail |",
              "| --- | --- | --- |",
              ($rows | sort_by(rank, .id) | .[:20][] | "| \(.status) | `\(.id)` | \(detail) |"),
              (if $n > 20 then "", "… and \($n - 20) more in the report." else empty end)
            else empty end)
        ' "$result"
        echo
        echo "[Download the report](${REPORT_URL:-$GITHUB_SERVER_URL/$GITHUB_REPOSITORY/actions/runs/$GITHUB_RUN_ID}) and open index.html for the diff images."
      } > "$body"
      comments="repos/${GITHUB_REPOSITORY}/issues/${PR_NUMBER}/comments"
      existing=$(gh api "$comments" --paginate --jq "first(.[] | select(.body | startswith(\"$marker\")) | .id)")
      if [ -n "$existing" ]; then
        gh api -X PATCH "repos/${GITHUB_REPOSITORY}/issues/comments/${existing}" -F body=@"$body" > /dev/null
      else
        gh api "$comments" -F body=@"$body" > /dev/null
      fi
```

The pieces that decide whether the comment actually appears:

- `pull-requests: write` is what lets `github.token` create and edit comments; the workflow's `permissions` block otherwise leaves it read-only.
- `if: ${{ !cancelled() }}` on both steps, because `mitame run` exits 1 on any visual change and a plain step would be skipped exactly when there is something to show.
- The same-repository check on `head.repo.full_name`. A pull request from a fork runs with a read-only token, so the step would fail there; skipping it keeps the fork's checks green and leaves the artifact as the review path.
- `test -f "$result" || exit 0` covers a run where `compare` never wrote a report, for example a test command that could not be started. A test command that fails part-way still produces a report for the captures it wrote, and the comment then lists the missing screenshots as `removed` while the job exits 2.
- The marker line `<!-- mitame:<job>/<profile> -->` is what the step searches for, so two jobs (or one job run for two profiles) each keep their own comment instead of overwriting each other's.
- The body goes through a temporary file (`-F body=@file`), which is how a multi-line Markdown body reaches `gh api` intact.

With the strict default threshold the comment normally says "No visual changes."; when it lists entries, the reviewer downloads the artifact, opens `index.html`, and either fixes the change or runs `mitame compare --update` locally and commits the baseline as part of the pull request. The job stays red until then, which is the assert-style default. For a review-style flow where the comment is the only signal, add `continue-on-error: true` to the `mitame run` step; the exit code is still available as `steps.<id>.outcome` if a later step wants it.

The same step works for the Android and iOS jobs unchanged, since `result.json` has one shape for every platform; the marker keeps their comments apart. A composite action that packages this rendering is on the [roadmap](roadmap.md) so a project can replace the inline script with one `uses:` line, and it will produce the same comment.

This repository's `ci.yml` runs the Flutter and Android examples on `ubuntu-latest` against their committed baselines, so a rendering change fails the job, and the iOS example on `macos-26` under a `ci-macos` profile with no committed baseline, which validates the pipeline without guarding regressions.
