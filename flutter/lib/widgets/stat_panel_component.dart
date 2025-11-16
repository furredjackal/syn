import 'package:flame/components.dart';
import 'package:flutter/material.dart';
import '../syn_game.dart';
import 'stat_bar_component.dart';

class StatPanelComponent extends PositionComponent with HasGameRef<SynGame> {
  @override
  Future<void> onLoad() async {
    final gameState = game.gameState;

    final background = RectangleComponent(
      paint: Paint()..color = Colors.black.withOpacity(0.2),
      size: size,
    );
    add(background);

    final title = TextComponent(
      text: 'STATS',
      textRenderer: TextPaint(
        style: const TextStyle(
          color: Color(0xFF00D9FF),
          fontSize: 16,
        ),
      ),
      position: Vector2(16, 16),
    );
    add(title);

    double yOffset = 48;
    _addStatBar('Health', gameState.health, yOffset);
    yOffset += 44;
    _addStatBar('Wealth', gameState.wealth, yOffset);
    yOffset += 44;
    _addStatBar('Charisma', gameState.charisma, yOffset);
    yOffset += 44;
    _addStatBar('Intelligence', gameState.intelligence, yOffset);
    yOffset += 44;
    _addStatBar('Wisdom', gameState.wisdom, yOffset);
    yOffset += 44;
    _addStatBar('Strength', gameState.strength, yOffset);
    yOffset += 44;
    _addStatBar('Stability', gameState.stability, yOffset);
  }

  void _addStatBar(String label, int value, double y) {
    add(StatBarComponent(
      label: label,
      value: value,
      position: Vector2(16, y),
      size: Vector2(size.x - 32, 32),
    ));
  }
}