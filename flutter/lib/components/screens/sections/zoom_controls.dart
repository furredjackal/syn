// File: flutter/lib/components/screens/sections/zoom_controls.dart
import 'dart:ui';

import 'package:flame/components.dart';
import 'package:flame/events.dart';
import 'package:flutter/material.dart';

class ZoomControls extends PositionComponent with TapCallbacks {
  ZoomControls({super.position, super.size});

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    const gap = 6.0;
    final buttonSize = Size((size.x - gap) / 2, size.y);

    // minus button
    final minusRect = Rect.fromLTWH(0, 0, buttonSize.width, buttonSize.height);
    _drawButton(canvas, minusRect, '-');

    // plus button
    final plusRect = Rect.fromLTWH(
      buttonSize.width + gap,
      0,
      buttonSize.width,
      buttonSize.height,
    );
    _drawButton(canvas, plusRect, '+');
  }

  void _drawButton(Canvas canvas, Rect rect, String label) {
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(8)),
      Paint()..color = const Color(0xFF0F172A),
    );
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(8)),
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = const Color(0xFF00D9FF),
    );

    final painter = TextPainter(
      text: TextSpan(
        text: label,
        style: const TextStyle(
          color: Colors.white,
          fontWeight: FontWeight.bold,
          fontSize: 20,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    painter.paint(
      canvas,
      Offset(
        rect.left + (rect.width - painter.width) / 2,
        rect.top + (rect.height - painter.height) / 2,
      ),
    );
  }
}
