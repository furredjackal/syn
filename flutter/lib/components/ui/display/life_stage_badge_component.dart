// File: flutter/lib/components/ui/display/life_stage_badge_component.dart
import 'dart:ui';

import 'package:flame/components.dart';
import 'package:flutter/material.dart';

import '../../../syn_game.dart';

class LifeStageBadgeComponent extends PositionComponent
    with HasGameReference<SynGame> {
  LifeStageBadgeComponent({super.position, super.size});

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    size = size == Vector2.zero() ? Vector2(120, 40) : size;
  }

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final rect = RRect.fromRectAndRadius(
      Rect.fromLTWH(0, 0, size.x, size.y),
      const Radius.circular(12),
    );

    canvas.drawRRect(
      rect,
      Paint()
        ..shader = const LinearGradient(
          colors: [Color(0xFF00D9FF), Color(0xFF7C3AED)],
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
        ).createShader(rect.outerRect),
    );

    final label = TextPainter(
      text: const TextSpan(
        text: 'LIFE STAGE',
        style: TextStyle(
          color: Colors.white,
          fontWeight: FontWeight.w800,
          letterSpacing: 1.5,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: size.x - 16);

    label.paint(
      canvas,
      Offset(
        (size.x - label.width) / 2,
        (size.y - label.height) / 2,
      ),
    );
  }
}
