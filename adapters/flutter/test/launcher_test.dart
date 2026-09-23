import 'dart:ffi';
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:mitame_flutter/src/launcher.dart';

String _dartExecutable() {
  final flutterRoot = Platform.environment['FLUTTER_ROOT'];
  if (flutterRoot != null) {
    final candidate = '$flutterRoot/bin/dart';
    if (File(candidate).existsSync()) {
      return candidate;
    }
  }
  final fromResolvedExecutable = Platform.resolvedExecutable
      .replaceFirst(RegExp(r'flutter_tester$'), 'dart');
  if (File(fromResolvedExecutable).existsSync()) {
    return fromResolvedExecutable;
  }
  return 'dart';
}

void main() {
  late Directory root;

  setUp(() {
    root = Directory.systemTemp.createTempSync('mitame_launcher');
  });

  tearDown(() {
    root.deleteSync(recursive: true);
  });

  File bundle(String target) {
    final file = File(
        [root.path, 'native', target, 'mitame'].join(Platform.pathSeparator));
    file.createSync(recursive: true);
    return file;
  }

  test('maps each supported ABI to its bundled binary', () {
    final files = {
      for (final target in mitameTargets.values) target: bundle(target),
    };
    expect(mitameTargets, {
      Abi.macosArm64: 'aarch64-apple-darwin',
      Abi.linuxX64: 'x86_64-unknown-linux-musl',
      Abi.linuxArm64: 'aarch64-unknown-linux-musl',
    });
    for (final entry in mitameTargets.entries) {
      expect(
        resolveBinary(environment: const {}, abi: entry.key, packageRoot: root),
        files[entry.value]!.path,
      );
    }
  });

  test('MITAME_BINARY takes precedence over the bundle and the ABI', () {
    bundle('aarch64-apple-darwin');
    expect(
      resolveBinary(
        environment: const {'MITAME_BINARY': '/opt/mitame'},
        abi: Abi.windowsX64,
        packageRoot: root,
      ),
      '/opt/mitame',
    );
  });

  test('empty MITAME_BINARY is ignored', () {
    final file = bundle('x86_64-unknown-linux-musl');
    expect(
      resolveBinary(
        environment: const {'MITAME_BINARY': ''},
        abi: Abi.linuxX64,
        packageRoot: root,
      ),
      file.path,
    );
  });

  test('an unsupported ABI explains how to get a binary', () {
    expect(
      () => resolveBinary(
          environment: const {}, abi: Abi.windowsX64, packageRoot: root),
      throwsA(isA<LauncherUnavailable>().having(
        (e) => e.message,
        'message',
        allOf(contains('windows_x64'), contains('MITAME_BINARY'),
            contains('cargo install')),
      )),
    );
  });

  test('a missing bundled binary names the path it looked for', () {
    final expected = [root.path, 'native', 'aarch64-apple-darwin', 'mitame']
        .join(Platform.pathSeparator);
    expect(
      () => resolveBinary(
          environment: const {}, abi: Abi.macosArm64, packageRoot: root),
      throwsA(isA<LauncherUnavailable>().having(
        (e) => e.message,
        'message',
        allOf(contains(expected), contains('MITAME_BINARY')),
      )),
    );
  });

  group('through dart run', () {
    final package = Directory.current.path;

    Future<ProcessResult> launch(List<String> args, String binary) {
      return Process.run(
        _dartExecutable(),
        ['run', 'mitame_flutter:mitame', ...args],
        workingDirectory: package,
        environment: {'MITAME_BINARY': binary},
      );
    }

    test('arguments reach the binary untouched', () async {
      final script = File('${root.path}/echo.sh')
        ..writeAsStringSync('#!/bin/sh\nprintf "%s|" "\$@"\nexit 3\n');
      await Process.run('chmod', ['755', script.path]);
      final result = await launch(
          ['run', '--', 'flutter', 'test', 'test/x_test.dart'], script.path);
      expect(result.stdout, 'run|--|flutter|test|test/x_test.dart|');
      expect(result.exitCode, 3);
    }, skip: Platform.isWindows);

    test('bad MITAME_BINARY exits 2 with a message', () async {
      final result = await launch(['--version'], '${root.path}/missing');
      expect(result.exitCode, 2);
      expect(result.stderr, contains('${root.path}/missing'));
    });
  });
}
