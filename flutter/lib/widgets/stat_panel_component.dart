import 'package:flame/components.dart';
import 'package:flutter/material.dart';
import '../syn_game.dart';
import 'stat_bar_component.dart';

class StatPanelComponent extends PositionComponent with HasGameReference<SynGame> {
  final Map<String, StatBarComponent> _statBars = {};
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
    _addStatBar('health', 'Health', gameState.health, yOffset);
    yOffset += 44;
    _addStatBar('wealth', 'Wealth', gameState.wealth, yOffset);
    yOffset += 44;
    _addStatBar('charisma', 'Charisma', gameState.charisma, yOffset);
    yOffset += 44;
    _addStatBar('intelligence', 'Intelligence', gameState.intelligence, yOffset);
    yOffset += 44;
    _addStatBar('wisdom', 'Wisdom', gameState.wisdom, yOffset);
    yOffset += 44;
    _addStatBar('strength', 'Strength', gameState.strength, yOffset);
    yOffset += 44;
    _addStatBar('stability', 'Stability', gameState.stability, yOffset);
  }

  void _addStatBar(String key, String label, int value, double y) {
    final bar = StatBarComponent(
      label: label,
      value: value,
      position: Vector2(16, y),
      size: Vector2(size.x - 32, 32),
    );
    _statBars[key] = bar;
    add(bar);
  }

  @override
  void update(double dt) {
    super.update(dt);
    final gameState = game.gameState;
    _statBars['health']?.updateValue(gameState.health);
    _statBars['wealth']?.updateValue(gameState.wealth);
    _statBars['charisma']?.updateValue(gameState.charisma);
    _statBars['intelligence']?.updateValue(gameState.intelligence);
    _statBars['wisdom']?.updateValue(gameState.wisdom);
    _statBars['strength']?.updateValue(gameState.strength);
    _statBars['stability']?.updateValue(gameState.stability);
  }
}
