import 'dart:convert';
import 'dart:io';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

import 'comparator.dart';
import 'identity.dart';

class Mitame {
  Mitame._();

  static const String outputDirEnv = 'MITAME_OUTPUT_DIR';
  static const String profileEnv = 'MITAME_PROFILE';
  static const String fontsEnv = 'MITAME_FONTS';
  static const String defaultProfile = 'default';

  static Future<void> install({bool loadFonts = false}) async {
    final current = goldenFileComparator;
    if (current is! LocalFileComparator) {
      throw StateError(
        'Mitame.install expects the default LocalFileComparator but found '
        '${current.runtimeType}. Call Mitame.install before any other '
        'comparator replacement.',
      );
    }
    final env = Platform.environment;
    final outputDir = Directory(
      env[outputDirEnv] ??
          '${Directory.current.path}${Platform.pathSeparator}.mitame${Platform.pathSeparator}current',
    );
    final profile = env[profileEnv] ?? defaultProfile;
    final testRoot =
        Directory('${Directory.current.path}${Platform.pathSeparator}test');
    goldenFileComparator = MitameComparator(
      testBasedir: current.basedir,
      testRoot: testRoot.uri,
      outputDir: outputDir,
      profile: profile,
    );
    if (loadFonts && env[fontsEnv] != 'ahem') {
      await loadAppFonts();
    }
  }

  static String golden(String name, [Map<String, String> variant = const {}]) {
    final identity = MitameIdentity(
      platform: 'flutter',
      group: const [],
      name: normalizeComponent(name),
      variant: {
        for (final entry in variant.entries)
          normalizeComponent(entry.key): normalizeComponent(entry.value),
      },
    );
    return '${identity.stem}.png';
  }

  static Iterable<Map<String, String>> matrix(Map<String, List<String>> axes) {
    final keys = axes.keys.toList()..sort();
    Iterable<Map<String, String>> expand(int index) sync* {
      if (index == keys.length) {
        yield const {};
        return;
      }
      final key = keys[index];
      for (final value in axes[key]!) {
        for (final rest in expand(index + 1)) {
          yield {key: value, ...rest};
        }
      }
    }

    return expand(0);
  }

  static Future<void> loadAppFonts() async {
    TestWidgetsFlutterBinding.ensureInitialized();
    await _loadManifestFonts();
    await _loadSdkFonts();
  }

  static Future<void> _loadManifestFonts() async {
    final manifest = await rootBundle.loadString('FontManifest.json');
    final entries = json.decode(manifest) as List<dynamic>;
    for (final entry in entries) {
      final family = _stripPackagePrefix(entry['family'] as String);
      final fonts = entry['fonts'] as List<dynamic>;
      final loader = FontLoader(family);
      for (final font in fonts) {
        loader.addFont(rootBundle.load(font['asset'] as String));
      }
      await loader.load();
    }
  }

  static Future<void> _loadSdkFonts() async {
    final dir = _materialFontsDir();
    if (dir == null || !dir.existsSync()) {
      return;
    }
    final files = dir
        .listSync()
        .whereType<File>()
        .where((f) =>
            f.path.split(Platform.pathSeparator).last.startsWith('Roboto-'))
        .toList();
    if (files.isEmpty) {
      return;
    }
    for (final family in const [
      'Roboto',
      'CupertinoSystemText',
      'CupertinoSystemDisplay'
    ]) {
      final loader = FontLoader(family);
      for (final file in files) {
        loader.addFont(file.readAsBytes().then((b) => ByteData.sublistView(b)));
      }
      await loader.load();
    }
  }

  static Directory? _materialFontsDir() {
    final root = Platform.environment['FLUTTER_ROOT'] ?? _rootFromExecutable();
    if (root == null) {
      return null;
    }
    return Directory([root, 'bin', 'cache', 'artifacts', 'material_fonts']
        .join(Platform.pathSeparator));
  }

  static String? _rootFromExecutable() {
    final marker =
        '${Platform.pathSeparator}bin${Platform.pathSeparator}cache${Platform.pathSeparator}';
    final exe = Platform.resolvedExecutable;
    final index = exe.indexOf(marker);
    return index < 0 ? null : exe.substring(0, index);
  }

  static String _stripPackagePrefix(String family) {
    const prefix = 'packages/';
    if (!family.startsWith(prefix)) {
      return family;
    }
    final rest = family.substring(prefix.length);
    final slash = rest.indexOf('/');
    return slash < 0 ? family : rest.substring(slash + 1);
  }
}
