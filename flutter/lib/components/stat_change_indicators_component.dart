import 'package:flame/components.dart';
import 'package:flutter/material.dart';

class StatChangeIndicatorsComponent extends PositionComponent {
  final Map<String, int> statChanges;

  StatChangeIndicatorsComponent({
    required this.statChanges,
    Vector2? position,
  }) : super(position: position ?? Vector2.zero());

  @override
  Future<void> onLoad() async {
    if (statChanges.isEmpty) {
      return;
    }

    double yOffset = 0;
    for (final entry in statChanges.entries) {
      final isPositive = entry.value >= 0;
      final color =
          isPositive ? const Color(0xFF52FF6D) : const Color(0xFFFF4F4F);
      final text = TextComponent(
        text: '${isPositive ? '+' : ''}${entry.value} ${entry.key}',
        textRenderer: TextPaint(
          style: TextStyle(
            fontFamily: 'Montserrat',
            fontWeight: FontWeight.w400,
            fontSize: 14,
            color: color,
          ),
        ),
        position: Vector2(0, yOffset),
      );
      await add(text);
      yOffset += text.size.y + 2;
    }
  }
}
