import 'dart:math' as math;

import 'package:flame/components.dart';
import 'package:flutter/material.dart';
import '../models/game_state.dart';
import '../syn_game.dart';
import '../ui/syn_theme.dart';
import 'character_info_component.dart';

class RelationshipPanelComponent extends PositionComponent
    with HasGameReference<SynGame> {
  final List<CharacterInfoComponent> _cards = [];

  @override
  Future<void> onLoad() async {
    add(_PanelFrame(size: size));
    _buildCards();
  }

  void _buildCards() {
    final relationships = game.gameState.relationships;
    final items = relationships.isEmpty
        ? _placeholderRelationships()
        : relationships.take(_maxCards).toList();

    final availableHeight = size.y - _panelPadding * 2;
    final cardHeight = availableHeight / _maxCards - _cardSpacing;

    for (var i = 0; i < math.min(_maxCards, items.length); i++) {
      final relationship = items[i];
      final card = CharacterInfoComponent(
        relationship: relationship,
        position: Vector2(
          _panelPadding,
          _panelPadding + i * (cardHeight + _cardSpacing),
        ),
        size: Vector2(size.x - _panelPadding * 2, cardHeight),
      );
      add(card);
      _cards.add(card);
    }

    if (relationships.isEmpty) {
      add(
        TextComponent(
          text: 'NO BONDS YET',
          anchor: Anchor.center,
          position: Vector2(size.x / 2, size.y / 2),
          textRenderer: TextPaint(
            style: const TextStyle(
              fontFamily: 'Montserrat',
              fontWeight: FontWeight.w700,
              letterSpacing: 2,
              color: Color(0x66FFFFFF),
              fontSize: 14,
            ),
          ),
        ),
      );
    }
  }

  List<RelationshipData> _placeholderRelationships() {
    return [
      RelationshipData(
        npcId: 'placeholder_1',
        npcName: 'MOM',
        affection: 8.0,
        trust: 6.0,
        attraction: -2.0,
        familiarity: 9.0,
        resentment: 1.0,
        state: 'Family',
      ),
      RelationshipData(
        npcId: 'placeholder_2',
        npcName: 'BEST FRIEND',
        affection: 7.0,
        trust: 8.0,
        attraction: 0.0,
        familiarity: 8.0,
        resentment: -1.0,
        state: 'Friend',
      ),
      RelationshipData(
        npcId: 'placeholder_3',
        npcName: 'RIVAL',
        affection: -3.0,
        trust: -2.0,
        attraction: 4.0,
        familiarity: 3.0,
        resentment: 5.0,
        state: 'Rival',
      ),
    ];
  }

  static const double _panelPadding = 18;
  static const double _cardSpacing = 12;
  static const int _maxCards = 3;
}

/// Panel frame with angled border (right side)
class _PanelFrame extends PositionComponent {
  _PanelFrame({required Vector2 size}) : super(size: size);

  @override
  void render(Canvas canvas) {
    const angleOffset = 12.0;

    // Background fill (dark semi-transparent, mirrored from StatPanel)
    final bgPath = Path()
      ..moveTo(0, angleOffset)
      ..lineTo(size.x, 0)
      ..lineTo(size.x, size.y - angleOffset)
      ..lineTo(0, size.y)
      ..close();

    canvas.drawPath(
      bgPath,
      Paint()
        ..color = SynColors.bgPanel.withOpacity(0.85)
        ..style = PaintingStyle.fill,
    );

    // Gradient overlay (subtle)
    canvas.drawPath(
      bgPath,
      Paint()
        ..shader = LinearGradient(
          begin: Alignment.topRight,
          end: Alignment.bottomLeft,
          colors: [
            SynColors.bgDark.withOpacity(0.4),
            SynColors.bgPanel.withOpacity(0.25),
          ],
        ).createShader(Rect.fromLTWH(0, 0, size.x, size.y)),
    );

    // Border (cyan, angled right)
    canvas.drawPath(
      bgPath,
      Paint()
        ..color = SynColors.accentViolet
        ..style = PaintingStyle.stroke
        ..strokeWidth = SynLayout.borderWidthHeavy,
    );

    // Inner accent line (depth)
    const innerOffset = 1.5;
    final innerPath = Path()
      ..moveTo(innerOffset, angleOffset + innerOffset)
      ..lineTo(size.x - innerOffset, innerOffset)
      ..lineTo(size.x - innerOffset, size.y - angleOffset - innerOffset)
      ..lineTo(innerOffset, size.y - innerOffset)
      ..close();

    canvas.drawPath(
      innerPath,
      Paint()
        ..color = SynColors.accentViolet.withOpacity(0.4)
        ..style = PaintingStyle.stroke
        ..strokeWidth = SynLayout.borderWidthLight,
    );
  }
}
