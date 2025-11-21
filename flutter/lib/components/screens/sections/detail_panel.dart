// File: flutter/lib/components/screens/sections/detail_panel.dart
import 'dart:ui';

import 'package:flame/components.dart';
import 'package:flutter/material.dart';

class DetailPanel extends PositionComponent {
  DetailPanel({super.position, super.size});

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(14)),
      Paint()..color = const Color(0xFF111827),
    );
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(14)),
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = const Color(0xFF00D9FF),
    );
  }
}
