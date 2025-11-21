// File: flutter/lib/components/ui/effects/slash_transition_effect.dart

import 'package:flame/components.dart';
import 'package:flutter/material.dart';

class SlashTransitionEffect extends PositionComponent {
  SlashTransitionEffect({super.position, super.size});

  double progress = 0.0;

  @override
  void update(double dt) {
    super.update(dt);
    progress = (progress + dt) % 1.0;
  }

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final width = size.x;
    final height = size.y;
    final slashWidth = width * 0.08;
    final offset = (width + slashWidth) * progress - slashWidth;

    final path = Path()
      ..moveTo(offset, 0)
      ..lineTo(offset + slashWidth, 0)
      ..lineTo(offset + slashWidth - 60, height)
      ..lineTo(offset - 60, height)
      ..close();

    canvas.drawPath(
      path,
      Paint()
        ..shader = const LinearGradient(
          colors: [Color(0x00FFFFFF), Color(0x55FFFFFF), Color(0x00FFFFFF)],
          stops: [0.0, 0.5, 1.0],
        ).createShader(Rect.fromLTWH(0, 0, width, height)),
    );
  }
}
