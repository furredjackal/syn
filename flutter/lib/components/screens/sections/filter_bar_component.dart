// File: flutter/lib/components/screens/sections/filter_bar_component.dart
import 'dart:ui';

import 'package:flame/components.dart';
import 'package:flutter/material.dart';

class FilterBarComponent extends PositionComponent {
  FilterBarComponent({super.position, super.size});

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    size = size == Vector2.zero() ? Vector2(320, 52) : size;
  }

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final rect = RRect.fromRectAndRadius(
      Rect.fromLTWH(0, 0, size.x, size.y),
      const Radius.circular(10),
    );
    canvas.drawRRect(rect, Paint()..color = const Color(0xFF0F172A));
    canvas.drawRRect(
      rect,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = const Color(0xFF00D9FF),
    );

    final label = TextPainter(
      text: const TextSpan(
        text: 'FILTERS',
        style: TextStyle(color: Colors.white, letterSpacing: 1.1),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    label.paint(canvas, const Offset(12, 14));
  }
}
