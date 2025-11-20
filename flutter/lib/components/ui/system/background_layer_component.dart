import 'dart:math';

import 'package:flame/components.dart';
import 'package:flutter/material.dart';

/// Flame background layer with the existing gradient + geometric accents.
class BackgroundLayerComponent extends PositionComponent {
  @override
  void onGameResize(Vector2 size) {
    super.onGameResize(size);
    this.size = size;
  }

  @override
  void render(Canvas canvas) {
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    final gradient = Paint()
      ..shader = const LinearGradient(
        colors: [
          Color(0xFFB80024),
          Color(0xFF5A000F),
        ],
        begin: Alignment.topLeft,
        end: Alignment.bottomRight,
      ).createShader(rect);
    canvas.drawRect(rect, gradient);

    final darkPaint = Paint()..color = const Color(0x22000000);
    for (int i = 0; i < 12; i++) {
      final y = size.y / 12 * i;
      canvas.drawRect(Rect.fromLTWH(0, y, size.x, size.y / 24), darkPaint);
    }

    final overlayPath = Path()
      ..moveTo(size.x * 0.62, 0)
      ..lineTo(size.x, size.y * 0.05)
      ..lineTo(size.x * 0.9, size.y)
      ..lineTo(size.x * 0.5, size.y)
      ..close();
    canvas.drawPath(
      overlayPath,
      Paint()..color = const Color(0xFF111111),
    );
    canvas.drawPath(
      overlayPath,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 6
        ..color = const Color(0xFFFFFFFF),
    );

    final triangles = Paint()..color = const Color(0xFFFFFFFF);
    final random = Random(7);
    for (int i = 0; i < 10; i++) {
      final x = size.x * 0.65 + random.nextDouble() * size.x * 0.3;
      final y = random.nextDouble() * size.y;
      final triangle = Path()
        ..moveTo(x, y)
        ..lineTo(x + 20, y + 8)
        ..lineTo(x, y + 16)
        ..close();
      canvas.drawPath(triangle, triangles);
    }
  }
}
