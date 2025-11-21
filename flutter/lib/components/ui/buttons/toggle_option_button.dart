// File: flutter/lib/components/ui/buttons/toggle_option_button.dart

import 'package:flame/components.dart';
import 'package:flame/events.dart';
import 'package:flutter/material.dart';

class ToggleOptionButton extends PositionComponent with TapCallbacks {
  ToggleOptionButton({super.position, super.size});

  bool isOn = false;

  @override
  void onTapDown(TapDownEvent event) {
    super.onTapDown(event);
    isOn = !isOn;
  }

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final rect = RRect.fromRectAndRadius(
      Rect.fromLTWH(0, 0, size.x, size.y),
      const Radius.circular(10),
    );
    canvas.drawRRect(
      rect,
      Paint()..color = isOn ? const Color(0xFF00D9FF) : const Color(0xFF111827),
    );
    canvas.drawRRect(
      rect,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = const Color(0xFF00D9FF),
    );

    final painter = TextPainter(
      text: TextSpan(
        text: isOn ? 'ON' : 'OFF',
        style: TextStyle(
          color: isOn ? Colors.black : Colors.white,
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
