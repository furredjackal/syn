// File: flutter/lib/components/screens/sections/wealth_display.dart

import 'package:flame/components.dart';
import 'package:flutter/material.dart';

class WealthDisplay extends PositionComponent {
  WealthDisplay({super.position, super.size});

  double credits = 0;

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(12)),
      Paint()..color = const Color(0xFF0B1323),
    );
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(12)),
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = const Color(0xFFFFC857),
    );

    final painter = TextPainter(
      text: TextSpan(
        text: '₡ ${credits.toStringAsFixed(0)}',
        style: const TextStyle(
          color: Colors.white,
          fontWeight: FontWeight.w800,
          fontSize: 18,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    painter.paint(canvas, const Offset(12, 10));
  }
}
