// File: flutter/lib/components/ui/display/ending_display.dart

import 'package:flame/components.dart';
import 'package:flutter/material.dart';

class EndingDisplay extends PositionComponent {
  EndingDisplay({super.position, super.size});

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    canvas.drawRect(rect, Paint()..color = const Color(0xFF0B1120));
    canvas.drawRect(
      rect,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = const Color(0xFFFACC15),
    );

    final painter = TextPainter(
      text: const TextSpan(
        text: 'ENDING',
        style: TextStyle(
          color: Colors.white,
          fontWeight: FontWeight.w800,
          letterSpacing: 1.4,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    painter.paint(canvas, const Offset(12, 12));
  }
}
