// File: flutter/lib/components/ui/buttons/overwrite_button.dart
import 'dart:ui';

import 'package:flame/components.dart';
import 'package:flame/events.dart';
import 'package:flutter/material.dart';

class OverwriteButton extends PositionComponent with TapCallbacks {
  OverwriteButton({super.position, super.size});

  @override
  void onTapDown(TapDownEvent event) {
    super.onTapDown(event);
    // TODO: handle overwrite confirmation
  }

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    _draw(canvas, 'OVERWRITE', const Color(0xFFFBBF24));
  }

  void _draw(Canvas canvas, String label, Color color) {
    final rect = RRect.fromRectAndRadius(
      Rect.fromLTWH(0, 0, size.x, size.y),
      const Radius.circular(10),
    );
    canvas.drawRRect(rect, Paint()..color = color);
    canvas.drawRRect(
      rect,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = const Color(0xFF000000),
    );
    final painter = TextPainter(
      text: TextSpan(
        text: label,
        style: const TextStyle(
          color: Colors.black,
          fontWeight: FontWeight.w800,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    painter.paint(
      canvas,
      Offset(
        (size.x - painter.width) / 2,
        (size.y - painter.height) / 2,
      ),
    );
  }
}
