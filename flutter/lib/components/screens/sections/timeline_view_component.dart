// File: flutter/lib/components/screens/sections/timeline_view_component.dart
import 'dart:ui';

import 'package:flame/components.dart';
import 'package:flutter/material.dart';

class TimelineViewComponent extends PositionComponent {
  TimelineViewComponent({super.position, super.size});

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    canvas.drawRect(rect, Paint()..color = const Color(0xFF0B1220));
    canvas.drawRect(
      rect,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = const Color(0xFF00D9FF),
    );

    final paint = Paint()
      ..color = const Color(0xFF00D9FF)
      ..strokeWidth = 3;
    final midY = size.y / 2;
    canvas.drawLine(Offset(12, midY), Offset(size.x - 12, midY), paint);
  }
}
