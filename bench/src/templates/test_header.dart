import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:mitame_bench/bench_widget.dart';

void main() {
  final marker = Platform.environment['BENCH_VARIANT'] == 'b' ? const Color(0xff00ff00) : const Color(0xffff0000);
  final skipGolden = Platform.environment['BENCH_SKIP_GOLDEN'] == '1';
