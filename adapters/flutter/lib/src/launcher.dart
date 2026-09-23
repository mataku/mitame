import 'dart:ffi';
import 'dart:io';

const String binaryEnv = 'MITAME_BINARY';

const Map<Abi, String> mitameTargets = {
  Abi.macosArm64: 'aarch64-apple-darwin',
  Abi.linuxX64: 'x86_64-unknown-linux-musl',
  Abi.linuxArm64: 'aarch64-unknown-linux-musl',
};

const String _install =
    'Install mitame with Homebrew (brew install mataku/tap/mitame) or from source '
    '(cargo install --git https://github.com/mataku/mitame mitame-cli), then set '
    '$binaryEnv to its path.';

class LauncherUnavailable implements Exception {
  LauncherUnavailable(this.message);

  final String message;

  @override
  String toString() => message;
}

String resolveBinary({
  required Map<String, String> environment,
  required Abi abi,
  required Directory packageRoot,
}) {
  final override = environment[binaryEnv];
  if (override != null && override.isNotEmpty) {
    return override;
  }
  final target = mitameTargets[abi];
  if (target == null) {
    throw LauncherUnavailable(
        'mitame_flutter bundles no mitame binary for $abi. $_install');
  }
  final file = File([packageRoot.path, 'native', target, 'mitame']
      .join(Platform.pathSeparator));
  if (!file.existsSync()) {
    throw LauncherUnavailable(
        'mitame_flutter has no bundled binary at ${file.path}; bundled binaries '
        'are only present in the package published to pub.dev. $_install');
  }
  return file.path;
}
