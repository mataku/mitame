import UIKit

final class LoginFormView: UIView {
    init(dark: Bool, title: String = "Sign in") {
        super.init(frame: .zero)
        backgroundColor = dark ? UIColor(red: 0.08, green: 0.07, blue: 0.09, alpha: 1) : UIColor(red: 1, green: 0.97, blue: 1, alpha: 1)
        let foreground = dark ? UIColor(white: 0.9, alpha: 1) : UIColor(white: 0.11, alpha: 1)

        let titleLabel = UILabel()
        titleLabel.text = title
        titleLabel.font = .systemFont(ofSize: 24, weight: .bold)
        titleLabel.textColor = foreground

        let email = UITextField()
        email.placeholder = "Email"
        email.borderStyle = .roundedRect
        let password = UITextField()
        password.placeholder = "Password"
        password.borderStyle = .roundedRect
        password.isSecureTextEntry = true

        let button = UIButton(type: .system)
        button.setTitle("Continue", for: .normal)
        button.setTitleColor(.white, for: .normal)
        button.backgroundColor = UIColor(red: 0.4, green: 0.31, blue: 0.64, alpha: 1)
        button.layer.cornerRadius = 8
        button.heightAnchor.constraint(equalToConstant: 48).isActive = true

        let stack = UIStackView(arrangedSubviews: [titleLabel, email, password, button])
        stack.axis = .vertical
        stack.spacing = 12
        stack.translatesAutoresizingMaskIntoConstraints = false
        addSubview(stack)
        NSLayoutConstraint.activate([
            stack.leadingAnchor.constraint(equalTo: leadingAnchor, constant: 16),
            stack.trailingAnchor.constraint(equalTo: trailingAnchor, constant: -16),
            stack.topAnchor.constraint(equalTo: topAnchor, constant: 16),
        ])
    }

    required init?(coder: NSCoder) { fatalError() }
}
