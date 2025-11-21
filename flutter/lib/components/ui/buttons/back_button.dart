// File: flutter/lib/components/ui/buttons/back_button.dart
import 'dart:ui';

import 'package:flame/components.dart';
import 'package:flame/events.dart';
import 'package:flutter/material.dart';

class BackButton extends PositionComponent with TapCallbacks {
  BackButton({this.onPressed, super.position, super.size});

  final VoidCallback? onPressed;
  final String label = 'RETURN';

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final path = Path()
      ..moveTo(0, 0)
      ..lineTo(size.x - 20, 0)
      ..lineTo(size.x, size.y)
      ..lineTo(20, size.y)
      ..close();
    canvas.drawPath(path, Paint()..color = const Color(0xFF00D9FF));
    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 3
        ..color = const Color(0xFF000000),
    );
    final painter = TextPainter(
      text: TextSpan(
        text: label,
        style: const TextStyle(
          color: Color(0xFF000000),
          fontSize: 20,
          fontWeight: FontWeight.w900,
        ),
      ),
      textAlign: TextAlign.center,
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

  @override
  void onTapDown(TapDownEvent event) {
    super.onTapDown(event);
    onPressed?.call();
  }
}
