# iOS adapter

`Mitame` is a Swift package exposed through the repository's root `Package.swift`; it depends on UIKit, SwiftUI, and Foundation only. `Mitame.capture` accepts a `UIView` or a SwiftUI `View` and renders it through `layer.render(in:)` at the screen scale; `Mitame.write` takes a ready `UIImage`.

## Install

Add the package to the test target, not to the app:

```swift
.package(url: "https://github.com/mataku/mitame", from: "0.1.0")
```

and `.product(name: "Mitame", package: "mitame")` in the test target's dependencies. In an Xcode project, add the package through File, Add Package Dependencies, and link the `Mitame` product to the UI test or unit test target only.

## Capture from a test

```swift
import Mitame
import XCTest

@MainActor
final class LoginFormTests: XCTestCase {
    func testLoginForm() throws {
        let view = LoginFormView(dark: false)
        try Mitame.capture(view, name: "login_form", variant: ["theme": "light"], size: CGSize(width: 360, height: 240))
        try Mitame.capture(LoginFormSwiftUI(dark: false), name: "login_form_swiftui", variant: ["theme": "light"])
    }
}
```

`@MainActor` is required because the view is laid out and rendered on the main thread. When `size:` is omitted a UIKit view uses its bounds, then its intrinsic content size, then `systemLayoutSizeFitting` at `Mitame.defaultWidth` (390 points); a SwiftUI view uses `sizeThatFits` at the same width, hosted in a `UIHostingController` with safe-area insets disabled. The group defaults to the test file's stem in snake case (`LoginFormTests` becomes `login_form_tests`); pass `group:` to override. The sidecar records the render scale, the iOS version, and the simulator name.

## Capture command and the environment prefix

XCTest processes only see environment variables that xcodebuild received with a `TEST_RUNNER_` prefix. `mitame run` sets both the plain and the prefixed forms, so the configured command needs nothing special:

```toml
[capture]
command = ["xcodebuild", "test", "-scheme", "MyApp", "-destination", "platform=iOS Simulator,name=iPhone 16,OS=18.3.1", "-quiet"]
```

A simulator name that exists for several OS versions is rejected without `OS=`; `xcodebuild -showdestinations -scheme MyApp` lists the valid combinations on the machine. A project with a workspace adds `-workspace MyApp.xcworkspace`. When the tests are run without `mitame run`, export `TEST_RUNNER_MITAME_OUTPUT_DIR` as an absolute path and `TEST_RUNNER_MITAME_PROFILE` yourself, and delete `.mitame/current/` first.

## Profiles

Simulator runtimes render differently across iOS versions, and CI runners rarely carry the same runtime as a developer machine. Give CI its own profile (`mitame run --profile ci-macos -- …`) and commit that profile's baseline only when the CI simulator is pinned to a specific device and OS; otherwise the CI run validates the pipeline (exit 2 on a capture failure) without guarding regressions, and the local `default` profile is where reviews happen. The mitame repository's own workflow picks the newest available iPhone runtime by UDID with `xcrun simctl list devices available -j` and passes `-destination "platform=iOS Simulator,id=<udid>"`.

## Things that produce a surprising first run

- A test that is not `@MainActor` fails to compile or crashes at render.
- A view with an image loaded asynchronously captures before the image arrives; load it synchronously or inject a placeholder in tests.
- `Mitame.capture` renders at `UIScreen.main.scale`, so a different simulator device changes the pixel dimensions and reports `mismatch`, not `changed`.
