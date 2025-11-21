// File: flutter/lib/components/screens/sections/relationship_detail_panel.dart
import 'dart:ui';

import 'package:flame/components.dart';
import 'package:flutter/material.dart';

class RelationshipDetailPanel extends PositionComponent {
  RelationshipDetailPanel({super.position, super.size});

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(14)),
      Paint()..color = const Color(0xFF0F172A),
    );
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(14)),
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = const Color(0xFFFF7A93),
    );
  }
}
