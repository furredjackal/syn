// File: flutter/lib/components/ui/buttons/load_button.dart

import 'package:flame/components.dart';
import 'package:flame/events.dart';
import 'package:flutter/material.dart';

class LoadButton extends PositionComponent with TapCallbacks {
  LoadButton({super.position, super.size});

  @override
  void onTapDown(TapDownEvent event) {
    super.onTapDown(event);
    // TODO: hook to load flow
  }

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final rect = RRect.fromRectAndRadius(
      Rect.fromLTWH(0, 0, size.x, size.y),
      const Radius.circular(10),
    );
    canvas.drawRRect(rect, Paint()..color = const Color(0xFF60A5FA));
    canvas.drawRRect(
      rect,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = const Color(0xFF000000),
    );

    final painter = TextPainter(
      text: const TextSpan(
        text: 'LOAD',
        style: TextStyle(
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
