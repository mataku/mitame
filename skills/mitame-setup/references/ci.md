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

The binary stops at `report/`. Pull request comments and remote baseline storage are not its job; baselines live in git or git-lfs, and a summary comment is a CI step that reads `result.json`, shown under "Pull request comment" below.

## Pull request comment

Add this only when the user asks for a pull request comment. It needs `jq` and `gh`, both preinstalled on GitHub-hosted runners, and no further dependency. Add `pull-requests: write` next to `contents: read` under `permissions`, and replace the upload step of the workflow above with these two steps (the upload gains an `id` so the comment can link to the artifact):

```yaml
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

Why each guard is there, so you can explain it in the setup report: `!cancelled()` runs the step although `mitame run` exited 1; the `head.repo.full_name` check skips fork pull requests, whose token cannot comment; `test -f` covers a run where `compare` never wrote a report (a test command that fails part-way still gets one, with the missing screenshots listed as `removed`); the marker `<!-- mitame:<job>/<profile> -->` makes the step edit its own earlier comment instead of adding one per push, and keeps one comment per job and profile; the temporary file is how a multi-line body reaches `gh api`. The comment is text only and links to the artifact for the images. The job still fails on changes; if the user wants the comment to be the only signal, add `continue-on-error: true` to the `mitame run` step. When a published `mitame-report` action exists, prefer it over this inline step; it renders the same comment.

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
