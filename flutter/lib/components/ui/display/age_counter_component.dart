// File: flutter/lib/components/ui/display/age_counter_component.dart

import 'package:flame/components.dart';
import 'package:flutter/material.dart';

import '../../../syn_game.dart';
import '../syn_theme.dart';

class AgeCounterComponent extends PositionComponent
    with HasGameReference<SynGame> {
  AgeCounterComponent({super.position, super.size});

  int displayedAge = 0;

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    size = size == Vector2.zero() ? Vector2(120, 40) : size;
  }

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final painter = TextPainter(
      text: TextSpan(
        children: [
          TextSpan(
            text: 'Age: ',
            style: SynTopBar.ageInlineStyle.copyWith(
              color: SynTopBar.ageLabelColor,
            ),
          ),
          TextSpan(
            text: '$displayedAge',
            style: SynTopBar.ageInlineStyle.copyWith(
              color: SynTopBar.ageValueColor,
              fontWeight: FontWeight.w800,
            ),
          ),
        ],
      ),
      textAlign: TextAlign.center,
      textDirection: TextDirection.ltr,
    )..layout();

    final dx = (size.x - painter.width) / 2;
    final dy = (size.y - painter.height) / 2;
    painter.paint(canvas, Offset(dx, dy));
  }
}
