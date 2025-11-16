import 'package:flame/components.dart';
import 'package:flutter/material.dart';
import 'package:syn/flutter/lib/syn_game.dart';
import 'package:syn/flutter/lib/widgets/character_info_component.dart';

class RelationshipPanelComponent extends PositionComponent
    with HasGameRef<SynGame> {
  @override
  Future<void> onLoad() async {
    final gameState = game.gameState;

    final background = RectangleComponent(
      paint: Paint()..color = Colors.black.withOpacity(0.2),
      size: size,
    );
    add(background);

    final title = TextComponent(
      text: 'RELATIONSHIPS',
      textRenderer: TextPaint(
        style: const TextStyle(
          color: Color(0xFF00D9FF),
          fontSize: 16,
        ),
      ),
      position: Vector2(16, 16),
    );
    add(title);

    if (gameState.relationships.isEmpty) {
      final emptyText = TextComponent(
        text: 'No relationships yet.',
        textRenderer: TextPaint(
          style: TextStyle(
            color: Colors.white.withOpacity(0.5),
            fontSize: 12,
          ),
        ),
        position: Vector2(16, 48),
      );
      add(emptyText);
    } else {
      double yOffset = 48;
      for (var rel in gameState.relationships) {
        add(CharacterInfoComponent(
          relationship: rel,
          position: Vector2(16, yOffset),
          size: Vector2(size.x - 32, 140),
        ));
        yOffset += 156;
      }
    }
  }
}