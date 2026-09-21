# iOS

`adapters/ios` is a Swift package whose `Mitame` library depends on UIKit, SwiftUI, and Foundation only. `Mitame.capture` accepts a `UIView` or a SwiftUI `View` (hosted in a `UIHostingController` with safe-area insets disabled), lays it out at the requested size, renders it through `layer.render(in:)` at the screen scale, and writes the PNG and sidecar; `Mitame.write` takes a ready `UIImage`. When `size:` is omitted, a UIKit view uses its bounds, then its intrinsic content size, then `systemLayoutSizeFitting` at `Mitame.defaultWidth` (390 points); a SwiftUI view uses `sizeThatFits` at the same width. The example test target runs on the iOS simulator through `xcodebuild test`.

```swift
@MainActor
final class LoginFormTests: XCTestCase {
    func testLoginForm() throws {
        let view = LoginFormView(dark: false)
        try Mitame.capture(view, name: "login_form", variant: ["theme": "light"], size: CGSize(width: 360, height: 240))
        try Mitame.capture(LoginFormSwiftUI(dark: false), name: "login_form_swiftui", variant: ["theme": "light"])
    }
}
```

The group defaults to the test file's stem in snake case (`login_form_tests`, with acronyms split as in `login_form_swift_ui_tests`); pass `group:` to override. The sidecar records the render scale, the iOS version as `env.sdk`, and the simulator name as `env.device`. XCTest processes only see environment variables that xcodebuild receives with a `TEST_RUNNER_` prefix; `mitame run` sets both forms, so:

```sh
mitame run -- xcodebuild test -scheme Mitame -destination 'platform=iOS Simulator,name=iPhone 16,OS=18.3.1'
```

Without `mitame run`, export `TEST_RUNNER_MITAME_OUTPUT_DIR` (an absolute path) and `TEST_RUNNER_MITAME_PROFILE` yourself. `xcodebuild -showdestinations -scheme Mitame` lists the simulator names and OS versions valid on a machine. A name alone is rejected when several runtimes have that device, so include `OS=<version>` (or `OS=latest` on CI).

See also the [iOS example](../adapters/ios/Tests/MitameExampleTests/README.md).
