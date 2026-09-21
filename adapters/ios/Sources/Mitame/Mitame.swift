import Foundation
import SwiftUI
import UIKit

public enum MitameError: Error, CustomStringConvertible {
    case noOutputDirectory
    case renderFailed
    case noSize
    case collision(String)

    public var description: String {
        switch self {
        case .noOutputDirectory:
            return "MITAME_OUTPUT_DIR is not set; pass it with TEST_RUNNER_MITAME_OUTPUT_DIR to xcodebuild"
        case .renderFailed:
            return "failed to render the view to an image"
        case .noSize:
            return "could not infer a size for the view; pass size: explicitly"
        case .collision(let id):
            return "Two captures resolved to the same mitame identity \"\(id)\" in this run. Use distinct names or pass group explicitly."
        }
    }
}

public enum Mitame {
    public static let outputDirEnv = "MITAME_OUTPUT_DIR"
    public static let profileEnv = "MITAME_PROFILE"
    public static let runIdEnv = "MITAME_RUN_ID"
    public static let platform = "ios"
    public static let schemaVersion = 1

    @MainActor
    @discardableResult
    public static func capture(
        _ view: UIView,
        name: String,
        variant: [String: String] = [:],
        group: String? = nil,
        size: CGSize? = nil,
        scale: CGFloat? = nil,
        file: StaticString = #filePath
    ) throws -> URL {
        let targetSize = try size ?? inferredSize(of: view)
        let window = UIWindow(frame: CGRect(origin: .zero, size: targetSize))
        view.frame = window.bounds
        window.addSubview(view)
        window.isHidden = false
        window.layoutIfNeeded()
        let format = UIGraphicsImageRendererFormat()
        format.scale = scale ?? UIScreen.main.scale
        format.opaque = false
        let renderer = UIGraphicsImageRenderer(size: targetSize, format: format)
        let image = renderer.image { context in
            view.layer.render(in: context.cgContext)
        }
        window.isHidden = true
        view.removeFromSuperview()
        return try write(image, name: name, variant: variant, group: group ?? fileGroup(file), scale: format.scale, displayName: name)
    }

    @MainActor
    @discardableResult
    public static func capture<Content: View>(
        _ content: Content,
        name: String,
        variant: [String: String] = [:],
        group: String? = nil,
        size: CGSize? = nil,
        scale: CGFloat? = nil,
        file: StaticString = #filePath
    ) throws -> URL {
        let host = UIHostingController(rootView: content)
        host.view.backgroundColor = .clear
        if #available(iOS 16.4, *) {
            host.safeAreaRegions = []
        }
        let targetSize: CGSize
        if let size {
            targetSize = size
        } else {
            let fitted = host.sizeThatFits(in: CGSize(width: defaultWidth, height: .greatestFiniteMagnitude))
            guard fitted.width > 0, fitted.height > 0, fitted.height.isFinite else { throw MitameError.noSize }
            targetSize = CGSize(width: fitted.width.rounded(.up), height: fitted.height.rounded(.up))
        }
        return try capture(host.view, name: name, variant: variant, group: group, size: targetSize, scale: scale, file: file)
    }

    @MainActor public static var defaultWidth: CGFloat = 390

    @MainActor
    static func inferredSize(of view: UIView) throws -> CGSize {
        if view.bounds.size != .zero {
            return view.bounds.size
        }
        let intrinsic = view.intrinsicContentSize
        if intrinsic.width != UIView.noIntrinsicMetric, intrinsic.height != UIView.noIntrinsicMetric,
           intrinsic.width > 0, intrinsic.height > 0 {
            return intrinsic
        }
        let fitted = view.systemLayoutSizeFitting(
            CGSize(width: defaultWidth, height: UIView.layoutFittingCompressedSize.height),
            withHorizontalFittingPriority: .required,
            verticalFittingPriority: .fittingSizeLevel
        )
        guard fitted.height > 0 else { throw MitameError.noSize }
        return CGSize(width: defaultWidth, height: fitted.height.rounded(.up))
    }

    @discardableResult
    public static func write(
        _ image: UIImage,
        name: String,
        variant: [String: String] = [:],
        group: String,
        scale: CGFloat,
        displayName: String? = nil
    ) throws -> URL {
        let env = ProcessInfo.processInfo.environment
        guard let outputDir = env[outputDirEnv] else { throw MitameError.noOutputDirectory }
        let profile = env[profileEnv] ?? "default"
        let runId = env[runIdEnv]
        let groupSegments = group.split(separator: "/").map { Identity.normalize(String($0)) }
        let normalizedName = Identity.normalize(name)
        var normalizedVariant: [String: String] = [:]
        for (key, value) in variant {
            normalizedVariant[Identity.normalize(key)] = Identity.normalize(value)
        }
        let stem = Identity.stem(name: normalizedName, variant: normalizedVariant)
        let id = Identity.id(platform: platform, group: groupSegments, stem: stem)
        let root = URL(fileURLWithPath: outputDir).appendingPathComponent(profile)
        let png = root.appendingPathComponent("\(id).png")
        let sidecar = root.appendingPathComponent("\(id).json")
        try checkCollision(id: id, sidecar: sidecar, runId: runId)
        guard let data = image.pngData(), let cgImage = image.cgImage else { throw MitameError.renderFailed }
        try FileManager.default.createDirectory(at: png.deletingLastPathComponent(), withIntermediateDirectories: true)
        try data.write(to: png, options: .atomic)
        var envInfo: [String: Any] = [
            "os": "ios",
            "sdk": UIDevice.current.systemVersion,
            "ci": env["CI"] != nil,
        ]
        if let device = env["SIMULATOR_DEVICE_NAME"] ?? env["SIMULATOR_MODEL_IDENTIFIER"] {
            envInfo["device"] = device
        }
        var ext: [String: Any] = [:]
        if let runId { ext["run_id"] = runId }
        let document: [String: Any] = [
            "schema_version": schemaVersion,
            "id": id,
            "platform": platform,
            "capture": "widget",
            "group": groupSegments.joined(separator: "/"),
            "name": normalizedName,
            "display_name": displayName ?? name,
            "variant": normalizedVariant,
            "image": ["width": cgImage.width, "height": cgImage.height, "scale": Double(scale)],
            "env": envInfo,
            "captured_at": ISO8601DateFormatter().string(from: Date()),
            "ext": ["ios": ext],
        ]
        let json = try JSONSerialization.data(withJSONObject: document, options: [.prettyPrinted, .sortedKeys, .withoutEscapingSlashes])
        try json.write(to: sidecar, options: .atomic)
        return png
    }

    static func fileGroup(_ file: StaticString) -> String {
        let path = "\(file)"
        let stem = URL(fileURLWithPath: path).deletingPathExtension().lastPathComponent
        return Identity.normalize(stem)
    }

    static func checkCollision(id: String, sidecar: URL, runId: String?) throws {
        guard let runId, let data = try? Data(contentsOf: sidecar),
              let object = try? JSONSerialization.jsonObject(with: data) as? [String: Any],
              let ext = object["ext"] as? [String: Any],
              let ios = ext["ios"] as? [String: Any],
              let previous = ios["run_id"] as? String else { return }
        if previous == runId { throw MitameError.collision(id) }
    }
}
