import 'package:flame/components.dart';
import 'package:flutter/material.dart';
import 'package:syn/flutter/lib/models/game_state.dart';
import 'package:syn/flutter/lib/syn_game.dart';

class CharacterInfoComponent extends PositionComponent with HasGameRef<SynGame> {
  final RelationshipData relationship;

  CharacterInfoComponent({
    required this.relationship,
    Vector2? position,
    Vector2? size,
  }) : super(position: position, size: size);

  @override
  Future<void> onLoad() async {
    final background = RectangleComponent(
      paint: Paint()..color = Colors.black.withOpacity(0.3),
      size: size,
    );
    add(background);

    final name = TextComponent(
      text: relationship.npcName.toUpperCase(),
      textRenderer: TextPaint(
        style: const TextStyle(
          color: Color(0xFF00D9FF),
          fontWeight: FontWeight.bold,
        ),
      ),
      position: Vector2(12, 12),
    );
    add(name);

    final stateColor = _getStateColor();
    final stateText = TextComponent(
      text: relationship.state,
      textRenderer: TextPaint(
        style: TextStyle(
          color: stateColor,
          fontSize: 10,
        ),
      ),
      position: Vector2(size.x - 12, 12),
      anchor: Anchor.topRight,
    );
    add(stateText);

    double yOffset = 40;
    _addAxis('Affection', relationship.affection, yOffset);
    yOffset += 20;
    _addAxis('Trust', relationship.trust, yOffset);
    yOffset += 20;
    _addAxis('Attraction', relationship.attraction, yOffset);
    yOffset += 20;
    _addAxis('Familiarity', relationship.familiarity, yOffset);
    yOffset += 20;
    _addAxis('Resentment', relationship.resentment, yOffset);
  }

  void _addAxis(String label, double value, double y) {
    add(RelationshipAxisComponent(
      label: label,
      value: value,
      position: Vector2(12, y),
      size: Vector2(size.x - 24, 16),
    ));
  }

  Color _getStateColor() {
    switch (relationship.state) {
      case 'Friend':
      case 'CloseFriend':
      case 'BestFriend':
        return const Color(0xFF00FF00);
      case 'RomanticInterest':
      case 'Partner':
      case 'Spouse':
        return const Color(0xFFFF1493);
      case 'Rival':
      case 'Estranged':
      case 'BrokenHeart':
        return const Color(0xFFFF4444);
      default:
        return const Color(0xFFFFAA00);
    }
  }
}

class RelationshipAxisComponent extends PositionComponent {
  final String label;
  final double value;
  final double maxValue;

  RelationshipAxisComponent({
    required this.label,
    required this.value,
    this.maxValue = 10.0,
    Vector2? position,
    Vector2? size,
  }) : super(position: position, size: size);

  @override
  Future<void> onLoad() async {
    final axisColor = _getAxisColor();

    final labelText = TextComponent(
      text: label,
      textRenderer: TextPaint(
        style: TextStyle(
          color: Colors.white.withOpacity(0.7),
          fontSize: 11,
        ),
      ),
      position: Vector2(0, 0),
    );
    add(labelText);

    final valueText = TextComponent(
      text: value.toStringAsFixed(1),
      textRenderer: TextPaint(
        style: TextStyle(
          color: axisColor,
          fontWeight: FontWeight.bold,
          fontSize: 11,
        ),
      ),
      position: Vector2(size.x, 0),
      anchor: Anchor.topRight,
    );
    add(valueText);
  }

  Color _getAxisColor() {
    if (value < -5) return const Color(0xFFFF0000);
    if (value < 0) return const Color(0xFFFF8800);
    if (value < 5) return const Color(0xFFFFAA00);
    return const Color(0xFF00FF00);
  }
}