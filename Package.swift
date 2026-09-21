// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "Mitame",
    platforms: [.iOS(.v15)],
    products: [
        .library(name: "Mitame", targets: ["Mitame"]),
    ],
    targets: [
        .target(name: "Mitame", path: "adapters/ios/Sources/Mitame"),
    ]
)
