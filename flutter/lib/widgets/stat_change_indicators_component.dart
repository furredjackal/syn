import 'package:flame/components.dart';
import 'package:flutter/material.dart';
import 'package:syn/flutter/lib/syn_game.dart';

class StatChangeIndicatorsComponent extends PositionComponent
    with HasGameRef<SynGame> {
  final Map<String, int> statChanges;

  StatChangeIndicatorsComponent({
    required this.statChanges,
    Vector2? position,
    Vector2? size,
  }) : super(position: position, size: size);

  @override
  Future<void> onLoad() async {
    double xOffset = 0;
    for (var entry in statChanges.entries) {
      final isPositive = entry.value > 0;
      final color = isPositive ? const Color(0xFF00FF00) : const Color(0xFFFF0000);

      final text = TextComponent(
        text: '${isPositive ? '+' : ''}${entry.value} ${entry.key}',
        textRenderer: TextPaint(
          style: TextStyle(
            color: color,
            fontSize: 11,
          ),
        ),
        position: Vector2(xOffset, 0),
      );

      add(text);
      xOffset += text.width + 8;
    }
  }
}