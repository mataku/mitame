# Flutter example

A minimal Flutter package that captures one widget in four variants plus one plain golden. Everything mitame needs is in three places:

- `pubspec.yaml`: `mitame_flutter` under `dev_dependencies` (a path dependency here; a git or pub.dev dependency in your project) and `flutter_test`.
- `test/flutter_test_config.dart`: `await Mitame.install(loadFonts: true)` before `testMain()`. This is the only change to test infrastructure.
- `test/login/login_form_test.dart`: an ordinary `testWidgets` using `matchesGoldenFile`. `Mitame.matrix` and `Mitame.golden` encode the theme and locale variants into the golden name; the plain `matchesGoldenFile('goldens/login_form.png')` call shows that unmodified tests are captured too.

`.mitame/baseline/` is committed; `.mitame/current/` and `.mitame/report/` are ignored by the repository's `.gitignore`.

Run from this directory:

```sh
MITAME_BINARY=../../../target/debug/mitame dart run mitame_flutter:mitame run
open .mitame/report/index.html
MITAME_BINARY=../../../target/debug/mitame dart run mitame_flutter:mitame compare --update    # after reviewing a change
```

Change the title in `lib/login_form.dart` and run `MITAME_BINARY=../../../target/debug/mitame dart run mitame_flutter:mitame run` again to see five `changed` entries with diff images. The committed baseline is rendered with Ahem (`fonts = "ahem"` in `mitame.toml`), which is why the same baseline passes on macOS and on the Linux CI runner.
