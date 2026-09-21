import Foundation

enum Identity {
    static func normalize(_ raw: String) -> String {
        var value = ""
        var previous: Character? = nil
        for character in raw {
            if character.isUppercase, let p = previous, p.isLowercase || p.isNumber {
                value.append("_")
            }
            value.append(character)
            previous = character
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
