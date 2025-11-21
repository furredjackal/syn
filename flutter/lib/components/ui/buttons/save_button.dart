// File: flutter/lib/components/ui/buttons/save_button.dart

import 'package:flame/components.dart';
import 'package:flame/events.dart';
import 'package:flutter/material.dart';

class SaveButton extends PositionComponent with TapCallbacks {
  SaveButton({super.position, super.size});

  @override
  void onTapDown(TapDownEvent event) {
    super.onTapDown(event);
    // TODO: hook to save flow
  }

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    _drawLabeledButton(canvas, 'SAVE', const Color(0xFF34D399));
  }

  void _drawLabeledButton(Canvas canvas, String label, Color color) {
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
