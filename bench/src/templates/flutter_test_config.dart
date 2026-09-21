import 'dart:async';
import 'dart:io';

import 'package:mitame_flutter/mitame_flutter.dart';

Future<void> testExecutable(FutureOr<void> Function() testMain) async {
  if (Platform.environment['BENCH_MODE'] == 'mitame') {
    await Mitame.install();
  }
  await testMain();
}
