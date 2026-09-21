# iOS example

The example is the test target of the `Mitame` Swift package: `LoginFormTests` captures a UIKit view in two themes at a fixed size, and `LoginFormSwiftUITests` captures a SwiftUI view without passing a size and a UIKit view through size inference. In your project the test target depends on the `Mitame` product from `.package(url: "https://github.com/mataku/mitame", from: "0.1.0")` and needs nothing else.

What to copy:

- `LoginFormTests.swift`: a `@MainActor` `XCTestCase` calling `try Mitame.capture(view, name:variant:size:)`.
- `LoginFormSwiftUITests.swift`: the same call with a SwiftUI `View` and no size.

XCTest only sees environment variables that xcodebuild received with a `TEST_RUNNER_` prefix, which `mitame run` takes care of. Run from `adapters/ios`:

```sh
../../target/debug/mitame run -- xcodebuild test -scheme Mitame -destination 'platform=iOS Simulator,name=iPhone 16,OS=18.3.1'
open .mitame/report/index.html
../../target/debug/mitame compare --update
```

Use `xcodebuild -showdestinations -scheme Mitame` to pick a simulator name and OS that exist on your machine. The committed baseline was rendered on an iPhone 16 simulator running iOS 18.3.1; other simulators or OS versions render slightly differently and belong in their own profile.
