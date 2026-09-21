import 'package:flutter/material.dart';

class BenchWidget extends StatelessWidget {
  const BenchWidget({super.key, required this.seed, required this.marker});

  final int seed;
  final Color marker;

  @override
  Widget build(BuildContext context) {
    return CustomPaint(painter: _Painter(seed, marker), child: const SizedBox.expand());
  }
}

class _Painter extends CustomPainter {
  _Painter(this.seed, this.marker);

  final int seed;
  final Color marker;

  @override
  void paint(Canvas canvas, Size size) {
    var state = seed * 2654435761 % 4294967296;
    int next() {
      state = (state * 1103515245 + 12345) % 2147483648;
      return state;
    }

    final paint = Paint();
    const cells = 12;
    final cw = size.width / cells;
    final ch = size.height / cells;
    for (var y = 0; y < cells; y++) {
      for (var x = 0; x < cells; x++) {
        paint.color = Color(0xff000000 | (next() & 0xffffff));
        canvas.drawRRect(
          RRect.fromRectAndRadius(Rect.fromLTWH(x * cw + 1, y * ch + 1, cw - 2, ch - 2), Radius.circular(cw / 4)),
          paint,
        );
      }
    }
    final text = TextPainter(
      text: TextSpan(text: 'bench $seed', style: const TextStyle(color: Colors.white, fontSize: 18)),
      textDirection: TextDirection.ltr,
    )..layout();
    text.paint(canvas, const Offset(8, 8));
    paint.color = marker;
    canvas.drawRect(const Rect.fromLTWH(0, 0, 1, 1), paint);
  }

  @override
  bool shouldRepaint(covariant _Painter old) => old.seed != seed || old.marker != marker;
}
