import 'dart:ffi';
import 'dart:io';
import 'dart:isolate';

import 'package:mitame_flutter/src/launcher.dart';

Future<void> main(List<String> args) async {
  final library = await Isolate.resolvePackageUri(
      Uri.parse('package:mitame_flutter/mitame_flutter.dart'));
  if (library == null) {
    stderr.writeln('mitame_flutter could not locate its own package root.');
    exit(2);
  }
  final String binary;
  try {
    binary = resolveBinary(
      environment: Platform.environment,
      abi: Abi.current(),
      packageRoot: File.fromUri(library).parent.parent,
    );
  } on LauncherUnavailable catch (e) {
    stderr.writeln(e.message);
    exit(2);
  }
  final Process process;
  try {
    process =
        await Process.start(binary, args, mode: ProcessStartMode.inheritStdio);
  } on ProcessException catch (e) {
    stderr.writeln('mitame_flutter could not start $binary: ${e.message}');
    exit(2);
  }
  exit(await process.exitCode);
}
