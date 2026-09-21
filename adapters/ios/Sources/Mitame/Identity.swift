import Foundation

enum Identity {
    static func normalize(_ raw: String) -> String {
        var value = ""
        let characters = Array(raw)
        for (index, character) in characters.enumerated() {
            if character.isUppercase, index > 0 {
                let previous = characters[index - 1]
                let next: Character? = index + 1 < characters.count ? characters[index + 1] : nil
                if previous.isLowercase || previous.isNumber || (previous.isUppercase && next?.isLowercase == true) {
                    value.append("_")
                }
            }
            value.append(character)
        }
        value = value.lowercased()
        let safe = value.map { c -> Character in
            (c.isASCII && (c.isLetter || c.isNumber || c == "_" || c == "." || c == "-")) ? c : "_"
        }
        var result = String(safe)
        while result.contains("__") {
            result = result.replacingOccurrences(of: "__", with: "_")
        }
        while let first = result.first, first == "." || first == "_" {
            result.removeFirst()
        }
        return result.isEmpty ? "_" : result
    }

    static func encodeVariant(_ variant: [String: String]) -> String {
        variant.keys.sorted().map { "\($0)=\(variant[$0]!)" }.joined(separator: ",")
    }

    static func stem(name: String, variant: [String: String]) -> String {
        variant.isEmpty ? name : "\(name)__\(encodeVariant(variant))"
    }

    static func id(platform: String, group: [String], stem: String) -> String {
        ([platform] + group + [stem]).joined(separator: "/")
    }
}
