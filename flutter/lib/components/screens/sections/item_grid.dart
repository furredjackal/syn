// File: flutter/lib/components/screens/sections/item_grid.dart
import 'dart:ui';

import 'package:flame/components.dart';
import 'package:flutter/material.dart';

class ItemGrid extends PositionComponent {
  ItemGrid({super.position, super.size});

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    canvas.drawRect(rect, Paint()..color = const Color(0xFF0F172A));
    canvas.drawRect(
      rect,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = const Color(0xFF00D9FF),
    );

    // simple grid lines as placeholder
    final cols = 4;
    final rows = 3;
    for (var c = 1; c < cols; c++) {
      final x = size.x / cols * c;
      canvas.drawLine(
        Offset(x, 0),
        Offset(x, size.y),
        Paint()..color = const Color(0x4422CCFF),
      );
    }
    for (var r = 1; r < rows; r++) {
      final y = size.y / rows * r;
      canvas.drawLine(
        Offset(0, y),
        Offset(size.x, y),
        Paint()..color = const Color(0x4422CCFF),
      );
    }
  }
}
