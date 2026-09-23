---
name: mitame-review
description: Run a mitame visual regression check, read its result.json and diff images, and update the committed baseline only for intended changes. Use this whenever a project has a mitame.toml or a .mitame/baseline/ directory and the user asks to run the screenshot, golden, or snapshot tests, check for visual regressions, see what changed visually, review or accept a diff, update or approve the baseline, or explain why mitame run or mitame compare exited 1 or 2 locally or in CI. Also use it before committing a UI change in such a project, since the baseline must move with the change.
metadata:
  mitame-version: "0.1.0"
---

# Running and reviewing mitame

mitame compares PNG captures in `.mitame/current/` against the committed baseline in `.mitame/baseline/` and writes `.mitame/report/` with `result.json`, diff images, and an HTML viewer. The test process never fails on a visual difference; the comparison does, with exit code 1, and the baseline is updated by a separate command that git then shows as an ordinary diff. Your job is to run it, read the result, decide per screenshot whether the change was intended, and update only those.

## Run

From anywhere inside the project (the binary walks up to the directory holding `mitame.toml` or `.mitame/baseline/`):

```sh
mitame run
```

This clears `.mitame/current/<profile>/`, runs the test command from `[capture] command` with the capture environment set, then compares. Exit codes:

- 0: nothing changed within the tolerance.
- 1: at least one entry is `changed` or `mismatch`, or an `added` or `removed` entry fails the `[policy]`.
- 2: the test command failed, or an entry has status `error`. The report is still written for the captures that succeeded, but do not update the baseline from it.

To run a subset of tests, append the test command's arguments after `--` and add `--keep-current` so the captures from tests that did not run are not cleared; without it, or with a filter and no `--keep-current`, every other screenshot is reported as `removed`. Do not call `flutter test`, Gradle, or `xcodebuild` directly for a capture; stale files from an earlier run would be compared as this run's.

## Read the result

Read `.mitame/report/result.json` before opening any image. Its `summary` counts entries per status, and each entry in `results` has `id`, `status`, `diff_pixels`, `diff_ratio`, `diff_bounds`, `baseline`, `current`, `diff`, and `message`. `diff_bounds` is the bounding box of the differing pixels as `x`, `y`, `width`, `height` in image pixels (the same box the diff image outlines), present whenever `diff_pixels` is above 0; crop the baseline and current PNGs to it instead of reading them whole, and read its size against the image size: a small box with dense pixels is a local text or colour change, a box spanning the image with few pixels is a layout shift. The three image paths are relative to the `.mitame/` directory, so `baseline/default/flutter/login/login_form.png` is `.mitame/baseline/default/flutter/login/login_form.png` and a diff is `.mitame/report/diff/<id>.png`; `diff` is null for entries without differing pixels.

- `unchanged`: within the tolerance; nothing to do.
- `changed`: pixels differ beyond the tolerance. The diff image outlines the differing region; open only that image, not the full baseline and current pair, unless the region is unclear.
- `mismatch`: the dimensions or the scale differ. This is a device, density, or profile problem, not a pixel change; check which simulator, density qualifier, or profile produced the capture before treating it as a UI change.
- `added`: no baseline entry. Expected for a new screenshot; suspicious for a renamed one, which appears as one `added` and one `removed`.
- `removed`: no current capture. After a full run this means the test was deleted or renamed, or the capture command skipped it (a Gradle task without `--rerun`, a filtered test run without `--keep-current`).
- `error`: the sidecar could not be read, names another identity, or was written for a newer schema version than the binary; the `message` names the cause, which is usually a binary and adapter version mismatch.

`mitame review` opens the report in the browser, and `mitame review <id>` opens the viewer at one screenshot with baseline, current, diff, and an onion-skin overlay; offer it to the user when a decision needs their eyes.

## Decide per screenshot

For each `changed` or `mismatch` entry, answer one question: is this the change the current work intended? Match the differing region against the diff of the source files (`git diff` on the UI code). A change in a screenshot whose source did not change points at something else: a dependency bump, a font, a clock or animation frame, a different device or profile. Report those to the user rather than accepting them.

Then update the baseline for the intended ones only:

```sh
mitame compare --update
git diff --stat .mitame/baseline
```

`--update` writes the `changed`, `mismatch`, and `added` entries into `.mitame/baseline/<profile>/` from the last comparison, without rerunning the tests, and leaves `unchanged` entries alone even when their bytes differ within the tolerance. It writes all of them, so revert the files for the entries you did not accept with `git checkout -- .mitame/baseline/<profile>/<platform>/<group>/<name>.png` (and its `.json` sidecar). Then run `mitame run` once more; it must exit 0. Commit the baseline files in the same commit as the UI change so the history shows them together.

## Guardrails

- Never raise `threshold` or `max_diff_pixels` in `[compare]` to make a run pass. The defaults are strict because a per-pixel colour tolerance and anti-aliasing detection already remove rendering noise, and a ratio threshold hides a one-word change on a full-screen golden. A genuinely noisy identity gets a `[[rules]]` entry with a `match` glob for that identity alone and the reason in the commit message.
- Never run `--prune` after a partial or filtered run, or after exit code 2. It deletes the baseline files for every `removed` entry, including the tests that simply did not run. Use it only after a full, successful run in which the `removed` entries are tests that were deliberately deleted.
- Do not update the baseline from a run that exited 2; fix the failure first.
- Do not edit or copy files under `.mitame/baseline/` by hand; `--update` and git are the only writers.
- Do not delete `.mitame/baseline/` and regenerate it to "start fresh". That loses the review; the git diff of an `--update` is the review.

## Report to the user

Lead with the outcome: the exit code and the summary counts. Then list the ids by status, with what changed for each `changed` entry in one line (the region and the likely source change), which ones you accepted into the baseline and which you left for them, and the baseline files that will be in the commit. When something changed without a matching source change, say so first.
