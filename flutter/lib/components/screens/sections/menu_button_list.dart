// File: flutter/lib/components/screens/sections/menu_button_list.dart

import 'package:flame/components.dart';
import 'package:flutter/material.dart';

class MenuButtonList extends PositionComponent {
  MenuButtonList({super.position, super.size});

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    canvas.drawRect(rect, Paint()..color = const Color(0xFF111827));
    canvas.drawRect(
      rect,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = const Color(0xFF00D9FF),
    );

    final painter = TextPainter(
      text: const TextSpan(
        text: 'MENU BUTTON LIST',
        style: TextStyle(color: Colors.white),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    painter.paint(canvas, const Offset(10, 8));
  }
}
