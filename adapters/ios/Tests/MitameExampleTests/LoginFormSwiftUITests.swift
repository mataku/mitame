import Mitame
import SwiftUI
import XCTest

@MainActor
final class LoginFormSwiftUITests: XCTestCase {
    func testLoginFormSwiftUI() throws {
        for theme in ["light", "dark"] {
            try Mitame.capture(LoginFormSwiftUI(dark: theme == "dark"), name: "login_form_swiftui", variant: ["theme": theme])
        }
    }

    func testInferredSizeForUIKitView() throws {
        let png = try Mitame.capture(LoginFormView(dark: false), name: "login_form_fitted")
        XCTAssertTrue(FileManager.default.fileExists(atPath: png.path))
    }
}
