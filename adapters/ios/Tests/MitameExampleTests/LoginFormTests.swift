import Mitame
import UIKit
import XCTest

@MainActor
final class LoginFormTests: XCTestCase {
    func testLoginForm() throws {
        for theme in ["light", "dark"] {
            let view = LoginFormView(dark: theme == "dark")
            try Mitame.capture(view, name: "login_form", variant: ["theme": theme], size: CGSize(width: 360, height: 240))
        }
    }
}
