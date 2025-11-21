// File: flutter/lib/components/ui/display/key_moments_list.dart

import 'package:flame/components.dart';
import 'package:flutter/material.dart';

class KeyMomentsList extends PositionComponent {
  KeyMomentsList({super.position, super.size});

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    canvas.drawRect(rect, Paint()..color = const Color(0xFF0F172A));
    canvas.drawRect(
      rect,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = const Color(0xFF00D9FF),
    );

    final painter = TextPainter(
      text: const TextSpan(
        text: 'KEY MOMENTS',
        style: TextStyle(color: Colors.white, fontWeight: FontWeight.w700),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    painter.paint(canvas, const Offset(10, 8));
  }
}
