// File: flutter/lib/components/ui/display/final_stats_display.dart

import 'package:flame/components.dart';
import 'package:flutter/material.dart';

class FinalStatsDisplay extends PositionComponent {
  FinalStatsDisplay({super.position, super.size});

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(14)),
      Paint()..color = const Color(0xFF0F172A),
    );
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(14)),
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = const Color(0xFF00D9FF),
    );

    final painter = TextPainter(
      text: const TextSpan(
        text: 'FINAL STATS',
        style: TextStyle(color: Colors.white, fontWeight: FontWeight.w800),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    painter.paint(canvas, const Offset(12, 10));
  }
}
