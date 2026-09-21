import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:flutter_test/flutter_test.dart';

import 'identity.dart';

class MitameComparator extends GoldenFileComparator {
  MitameComparator({
    required this.testBasedir,
    required this.testRoot,
    required this.outputDir,
    required this.profile,
    this.groupFromGoldenUri = false,
    this.runId,
    this.sdkVersion,
  });

  static const int schemaVersion = 1;
  static const String platform = 'flutter';

  final Uri testBasedir;
  final Uri testRoot;
  final Directory outputDir;
  final String profile;
  final bool groupFromGoldenUri;
  final String? runId;
  final String? sdkVersion;

  @override
  Future<bool> compare(Uint8List imageBytes, Uri golden) async {
    await _write(golden, imageBytes);
    return true;
  }

  @override
  Future<void> update(Uri golden, Uint8List imageBytes) =>
      _write(golden, imageBytes);

  Future<void> _write(Uri golden, Uint8List imageBytes) async {
    final identity = identityFor(golden);
    final root =
        Directory('${outputDir.path}${Platform.pathSeparator}$profile');
    final pngFile = File(_join(root.path, identity.relativePath));
    final sidecar = File(_join(root.path, identity.sidecarRelativePath));
    _checkCollision(identity, sidecar);
    await pngFile.parent.create(recursive: true);
    await pngFile.writeAsBytes(imageBytes, flush: true);
    await sidecar.writeAsString(
      const JsonEncoder.withIndent('  ')
          .convert(_sidecar(identity, golden, imageBytes)),
      flush: true,
    );
  }

  void _checkCollision(MitameIdentity identity, File sidecar) {
    if (runId == null || !sidecar.existsSync()) {
      return;
    }
    final existing = json.decode(sidecar.readAsStringSync());
    final previous = existing is Map
        ? (existing['ext'] as Map?)?['flutter']?['run_id']
        : null;
    if (previous == runId) {
      throw StateError(
        'Two goldens resolved to the same mitame identity "${identity.id}" in this run. '
        'Use distinct golden file names, or Mitame.install(groupFromGoldenUri: true) '
        'to keep the golden directory in the identity.',
      );
    }
  }

  MitameIdentity identityFor(Uri golden) {
    final testDir = _relativeSegments(testRoot, testBasedir);
    final goldenSegments =
        golden.pathSegments.where((s) => s.isNotEmpty).toList();
    if (goldenSegments.isEmpty) {
      throw ArgumentError('Golden URI has no file name: $golden');
    }
    final fileName = goldenSegments.removeLast();
    final stem = fileName.endsWith('.png')
        ? fileName.substring(0, fileName.length - 4)
        : fileName;
    final split = stem.indexOf('__');
    final rawName = split < 0 ? stem : stem.substring(0, split);
    final rawVariant = split < 0
        ? const <String, String>{}
        : decodeVariant(stem.substring(split + 2));
    return MitameIdentity(
      platform: platform,
      group: [
        ...testDir,
        if (groupFromGoldenUri) ...goldenSegments,
      ].map(normalizeComponent).toList(),
      name: normalizeComponent(rawName),
      variant: {
        for (final entry in rawVariant.entries)
          normalizeComponent(entry.key): normalizeComponent(entry.value),
      },
    );
  }

  Map<String, Object?> _sidecar(
      MitameIdentity identity, Uri golden, Uint8List png) {
    final data = ByteData.sublistView(png);
    return {
      'schema_version': schemaVersion,
      'id': identity.id,
      'platform': platform,
      'capture': 'widget',
      'group': identity.group.join('/'),
      'name': identity.name,
      'display_name': golden.pathSegments.last,
      'variant': identity.variant,
      'image': {
        'width': data.getUint32(16),
        'height': data.getUint32(20),
        'scale': 1.0,
      },
      'env': {
        'os': Platform.operatingSystem,
        'arch': _arch(),
        if (sdkVersion != null) 'sdk': sdkVersion,
        'ci': Platform.environment.containsKey('CI'),
      },
      'captured_at': DateTime.now().toUtc().toIso8601String(),
      'ext': {
        'flutter': {
          'golden_uri': golden.toString(),
          'test_dir': _relativeSegments(testRoot, testBasedir).join('/'),
          if (runId != null) 'run_id': runId,
        },
      },
    };
  }

  static List<String> _relativeSegments(Uri root, Uri child) {
    final rootSegments = root.pathSegments.where((s) => s.isNotEmpty).toList();
    final childSegments =
        child.pathSegments.where((s) => s.isNotEmpty).toList();
    if (childSegments.length < rootSegments.length) {
      throw StateError('Test directory $child is outside $root');
    }
    for (var i = 0; i < rootSegments.length; i++) {
      if (rootSegments[i] != childSegments[i]) {
        throw StateError('Test directory $child is outside $root');
      }
    }
    return childSegments.sublist(rootSegments.length);
  }

  static String _join(String base, String relative) =>
      '$base${Platform.pathSeparator}${relative.replaceAll('/', Platform.pathSeparator)}';

  static String _arch() {
    final version = Platform.version;
    if (version.contains('arm64')) {
      return 'aarch64';
    }
    if (version.contains('x64')) {
      return 'x86_64';
    }
    return 'unknown';
  }
}
