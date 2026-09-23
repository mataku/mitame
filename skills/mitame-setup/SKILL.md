---
name: mitame-setup
description: Set up mitame, the visual regression testing tool for Flutter, Android, and iOS (a Rust binary that compares screenshots against a baseline committed to git, writes a diff report, and updates the baseline through git), in a project that does not have it yet. Use this whenever the user wants to add, adopt, migrate to, or wire up mitame, wants screenshot, golden, or snapshot tests with a reviewable baseline and CI job, mentions replacing golden_toolkit, alchemist, Roborazzi, Paparazzi, or swift-snapshot-testing with mitame, or asks why a mitame baseline captured on macOS does not pass on Linux CI. It covers installing the binary, mitame init, the per-platform capture adapter, the .gitignore and baseline commit rules, the first baseline, and the GitHub Actions job.
metadata:
  mitame-version: "0.1.0"
  mitame-flutter-version: "0.1.0"
  mitame-android-version: "0.1.0"
---

# Setting up mitame

mitame splits visual regression testing into two halves. A thin capture adapter, installed in the project's test target, turns the platform's screenshot or golden mechanism into PNG files under `.mitame/current/`. The `mitame` binary, installed per machine, compares those files against `.mitame/baseline/`, which is committed to git, and writes `.mitame/report/` with `result.json`, diff images, and an HTML viewer. Nothing in the test process fails on a visual difference: `mitame compare` exits 1 and the report shows what changed, and `mitame compare --update` writes the accepted screenshots into the baseline so git shows the change as an ordinary diff.

This skill targets the versions in its frontmatter. Check `mitame --version` and the adapter version you install against them; when they differ, the flags and adapter APIs below may have moved, and the release notes at https://github.com/mataku/mitame/releases are the source of truth.

## Workflow

Work through these steps in order. Each one leaves the project in a state you can verify before the next.

### 1. Find the project root and install the binary

The project root is the directory that will hold `mitame.toml` and `.mitame/`. For Flutter it is the package directory with `pubspec.yaml`. For Android it is the Gradle root, not the module, because the example build script points capture output at the root project. For iOS it is the directory you run `xcodebuild` from. When the repository holds several apps, each gets its own root.

Check for the binary with `mitame --version`. When it is missing, install it with `brew install mataku/tap/mitame` on macOS or Linux, or download the release archive for the platform from https://github.com/mataku/mitame/releases and put `mitame` on `PATH`. Do not build from source unless the user asks; the prebuilt binary is what CI will run.

### 2. Write `mitame.toml` with `mitame init`

Run `mitame init` in the project root. It writes `mitame.toml` with the defaults and a `[capture] command` detected from the project: `flutter test` next to a `pubspec.yaml`, `./gradlew test --rerun` next to a Gradle settings file. For a Flutter project, a binary newer than 0.1.0 also sets `fonts = "ahem"` (see the font decision below). When `test/flutter_test_config.dart` does not exist yet, create it; the Flutter reference has its content. Read the file it wrote and adjust only the capture command:

- Android: narrow the task to the module that holds the screenshot tests, for example `["./gradlew", ":app:testDebugUnitTest", "--rerun"]`. Keep `--rerun`; Gradle skips an up-to-date test task, and a skipped task captures nothing, which the comparison then reports as every screenshot removed.
- iOS: set the full `xcodebuild test` invocation, including a `-destination` with `OS=`. See `references/ios.md`.
- Flutter: leave `flutter test` alone unless the project runs tests through a wrapper such as fvm; `mitame run` finds the fvm SDK on its own through `.fvmrc`.

Do not hand-write the other sections. The comparison defaults (`threshold = 0.0`, `max_diff_pixels = 0`, `anti_aliasing = true`) are strict on purpose; a per-pixel colour tolerance and anti-aliasing detection already absorb rendering noise, and a ratio threshold would hide a one-word change on a full-screen golden.

### 3. Add the capture adapter

Read the reference for the platform and follow it; each holds the exact dependency, the test-side code, and the gotchas that are not visible from the README.

- Flutter: `references/flutter.md`
- Android: `references/android.md`
- iOS: `references/ios.md`

Adapters depend on the platform SDK only. Do not add a helper package alongside them, and do not keep the previous screenshot library's comparison step active in the same test run; two comparators on one golden call is the usual cause of a test that fails locally but never writes a capture.

### 4. Set the git rules

Add `.mitame/current/` and `.mitame/report/` to `.gitignore` and leave `.mitame/baseline/` tracked. If the repository already ignores `.mitame/` wholesale, replace that line with the two entries; an ignored baseline is the most common reason CI compares against nothing. For a large baseline, git-lfs on `.mitame/baseline/**/*.png` is fine; the binary reads plain files either way.

### 5. Produce and commit the first baseline

Run `mitame run` from the project root. With no baseline, every entry is reported as `added` and the exit code is 0, because `added` is a warning under the default `[policy]`; exit code 2 means the test command or a capture failed, and the output names the cause. Then run `mitame compare --update` to write the captures into `.mitame/baseline/`, and commit the baseline together with the configuration and adapter changes.

Verify before moving on:

- `mitame run` now exits 0.
- A visible change to a captured widget makes `mitame run` exit 1 with that identity reported as `changed` and a diff image under `.mitame/report/diff/`. Under Ahem, replacing a word with one of the same length changes nothing on screen, so change padding, a colour, or the number of characters. Revert the change afterwards, and do not run `--update` while it is in place.
- `git status` shows nothing under `.mitame/current/` or `.mitame/report/`.

Once the baseline is committed, delete the previous library's golden or snapshot files (`test/**/goldens/`, `__Snapshots__/`, and the like) in a commit of their own; nothing reads them any more, and leaving them invites someone to update the wrong set.

Pin the adapter to the binary's version (`mitame_flutter: 0.1.0` rather than `^0.1.0` when the project does not commit `pubspec.lock`; an exact Maven or Swift package version otherwise) and bump both together. The sidecar carries a schema version, and a capture written for a newer schema than the binary reads is reported as `error` rather than compared; older schemas are read.

### 6. Add the CI job

Read `references/ci.md`. It has the minimal GitHub Actions job, the version pin, an optional pull request comment step, and the decision that matters most: whether one baseline can serve both the developer machines and the CI runner, which depends on the platform and, for Flutter, on the `fonts` setting.

### 7. Report what was done

Tell the user which files changed, where the baseline lives, how to review a difference (`mitame review`, or open `.mitame/report/index.html`), how to accept one (`mitame compare --update`, then review the git diff and revert the files they do not want), and which decision from the next section they should confirm. Point them at the mitame-review skill, when it is installed, for day-to-day runs.

## Decisions that shape the setup

**Ahem or real fonts (Flutter only).** `fonts = "ahem"` renders every glyph as a box and gives one baseline that passes on macOS and Linux CI, at the cost of never checking real glyphs; Ahem boxes are wider than most fonts and can overflow tight layouts. `fonts = "real"` checks real text but CoreText and FreeType shade glyphs differently, so each rendering platform needs its own baseline, selected with `--profile` or `MITAME_PROFILE`. Default to Ahem, which is what `mitame init` writes, and state the trade-off to the user rather than deciding silently. The `[capture] fonts` key is newer than binary 0.1.0: that binary ignores the key, does not set `MITAME_FONTS`, and keeps the colour tolerance at 0.1. `mitame_flutter` 0.1.0 already honours the variable, so with binary 0.1.0 export `MITAME_FONTS=ahem` in the environment of every `mitame run` (locally and in CI) and set `pixel_tolerance = 0.2` under `[compare]`, which is the value a newer binary derives from `fonts = "ahem"`.

**Profiles.** A profile is a baseline directory per rendering platform (`.mitame/baseline/<profile>/`). Android under Robolectric and Flutter under Ahem render the same pixels on macOS and Linux and need only `default`. iOS simulators render differently across OS versions, so CI usually gets its own profile, and real-font Flutter needs a `linux` profile captured in a container. Introduce a profile only when the platform needs one.

**Identity.** Every screenshot is `<platform>/<group>/<name>[__<variant>]` with components restricted to `[a-z0-9_.-]`. On Flutter the group is the test file's directory under `test/`, so two goldens with the same stem in one test directory collide; on Android and iOS it is the test class or file name in snake case. Variants such as theme and locale belong in the variant map, not in the name, so the report groups them.

## Guardrails

These match the boundaries the mitame repository holds for itself; keep them in the consumer project too.

- Do not raise `threshold` or `max_diff_pixels` to make the first run pass. A screenshot that differs between two runs on the same machine has a real cause (an animation, a clock, a random seed). Fix that, or add a `[[rules]]` entry for the one identity that needs it, with the reason in the commit message.
- Do not run `--prune`. Removed entries are a warning by default, and pruning after a partial run deletes baseline files that the tests which did not run still need.
- Do not commit `.mitame/current/` or `.mitame/report/`, and do not add `.mitame/baseline/` to `.gitignore`.
- Do not wire the binary to object storage or the GitHub API. The report is uploaded with `actions/upload-artifact`; the binary stops at `.mitame/report/`.
- Do not edit files under `.mitame/baseline/` by hand; `mitame compare --update` and git are the only writers.
