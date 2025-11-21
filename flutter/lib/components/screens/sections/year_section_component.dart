// File: flutter/lib/components/screens/sections/year_section_component.dart
import 'dart:ui';

import 'package:flame/components.dart';
import 'package:flutter/material.dart';

class YearSectionComponent extends PositionComponent {
  YearSectionComponent({super.position, super.size});

  int year = 0;

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    canvas.drawRect(rect, Paint()..color = const Color(0xFF0D0F14));
    canvas.drawRect(
      rect,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = const Color(0xFF00D9FF),
    );

    final painter = TextPainter(
      text: TextSpan(
        text: 'YEAR $year',
        style: const TextStyle(
          color: Colors.white,
          fontWeight: FontWeight.bold,
          letterSpacing: 1.2,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    painter.paint(canvas, const Offset(12, 10));
  }
}
