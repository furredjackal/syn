// File: flutter/lib/components/screens/sections/network_graph_component.dart
import 'dart:ui';

import 'package:flame/components.dart';
import 'package:flutter/material.dart';

class NetworkGraphComponent extends PositionComponent {
  NetworkGraphComponent({super.position, super.size});

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    canvas.drawRect(rect, Paint()..color = const Color(0xFF0D1320));
    canvas.drawRect(
      rect,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = const Color(0xFF00D9FF),
    );

    final paint = Paint()
      ..color = const Color(0xFF7C3AED)
      ..strokeWidth = 2;
    canvas.drawCircle(Offset(size.x * 0.3, size.y * 0.4), 10, paint);
    canvas.drawCircle(Offset(size.x * 0.7, size.y * 0.6), 10, paint);
    canvas.drawLine(
      Offset(size.x * 0.3, size.y * 0.4),
      Offset(size.x * 0.7, size.y * 0.6),
      paint,
    );
  }
}
