// File: flutter/lib/components/screens/sections/stat_item_component.dart

import 'package:flame/components.dart';
import 'package:flutter/material.dart';

class StatItemComponent extends PositionComponent {
  StatItemComponent({super.position, super.size});

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final rect = RRect.fromRectAndRadius(
      Rect.fromLTWH(0, 0, size.x, size.y),
      const Radius.circular(8),
    );
    canvas.drawRRect(rect, Paint()..color = const Color(0xFF0E1624));
    canvas.drawRRect(
      rect,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.5
        ..color = const Color(0xFF7C3AED),
    );

    final label = TextPainter(
      text: const TextSpan(
        text: 'STAT',
        style: TextStyle(color: Colors.white),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    label.paint(canvas, const Offset(10, 8));
  }
}
