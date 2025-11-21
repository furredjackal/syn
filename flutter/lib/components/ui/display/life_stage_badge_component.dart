// File: flutter/lib/components/ui/display/life_stage_badge_component.dart

import 'package:flame/components.dart';
import 'package:flutter/material.dart';

import '../../../syn_game.dart';
import '../syn_theme.dart';

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
    final stage = game.gameState.lifeStage.isNotEmpty
        ? game.gameState.lifeStage.toUpperCase()
        : 'CHILD';
    final painter = TextPainter(
      text: TextSpan(
        text: 'Lifestage: $stage',
        style: SynTopBar.lifeStageTextStyle,
      ),
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: size.x - 8);

    final dx = (size.x - painter.width) / 2;
    final dy = (size.y - painter.height) / 2;
    painter.paint(canvas, Offset(dx, dy));
  }
}
