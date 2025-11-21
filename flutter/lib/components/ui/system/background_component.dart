// File: flutter/lib/components/ui/system/background_component.dart

import 'package:flame/components.dart';
import 'package:flutter/material.dart';

class BackgroundComponent extends PositionComponent {
  BackgroundComponent({super.position, super.size});

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    canvas.drawRect(
      rect,
      Paint()
        ..shader = const LinearGradient(
          colors: [Color(0xFF05070D), Color(0xFF0D1320)],
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
        ).createShader(rect),
    );

    for (var i = 0; i < 40; i++) {
      final y = (i / 40) * size.y;
      canvas.drawLine(
        Offset(0, y),
        Offset(size.x, y),
        Paint()..color = const Color(0x1100D9FF),
      );
    }
  }
}
