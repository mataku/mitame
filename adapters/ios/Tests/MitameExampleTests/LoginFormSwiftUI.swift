import SwiftUI

struct LoginFormSwiftUI: View {
    var dark: Bool
    var title: String = "Sign in"

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text(title)
                .font(.system(size: 24, weight: .bold))
            TextField("Email", text: .constant(""))
                .textFieldStyle(.roundedBorder)
            SecureField("Password", text: .constant(""))
                .textFieldStyle(.roundedBorder)
            Button("Continue") {}
                .frame(maxWidth: .infinity, minHeight: 48)
                .background(Color(red: 0.4, green: 0.31, blue: 0.64))
                .foregroundColor(.white)
                .cornerRadius(8)
        }
        .padding(16)
        .background(dark ? Color(red: 0.08, green: 0.07, blue: 0.09) : Color(red: 1, green: 0.97, blue: 1))
        .environment(\.colorScheme, dark ? .dark : .light)
    }
}
