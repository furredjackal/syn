import 'dart:math';

import 'package:flame/components.dart';
import 'package:flutter/material.dart';

import '../../../models/game_state.dart';
import '../../../syn_game.dart';
import '../cards/npc_card_component.dart';
import '../syn_theme.dart';

class RelationshipPanelComponent extends PositionComponent
    with HasGameReference<SynGame> {
  RelationshipPanelComponent({
    required this.relationships,
    required this.onOpenNetwork,
    super.position,
    super.size,
  });

  final List<RelationshipData> relationships;
  final VoidCallback onOpenNetwork;

  // Visual polish constants
  static final _backgroundGradient = LinearGradient(
    colors: [
      const Color(0xFF0A081A), // Dark blue/purple
      const Color(0xFF180F2A), // Dark magenta/indigo
    ],
    begin: Alignment.topCenter,
    end: Alignment.bottomCenter,
  );
  static const _borderColor = Color(0xFFFF4A9B); // Bright magenta/pink
  static const _slashColor = Color(0xFFFF4A9B);
  static const _borderWidth = SynLayout.borderWidthHeavy;

  // Layout constants
  static const double _topMargin = 24.0;
  static const double _bottomMargin = 24.0;
  static const double _horizontalPadding = 16.0;
  static const double _cardHeight = 78.0;
  static const double _minSpacing = 12.0;
  static const int _maxCards = 4;

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    _updateCardLayout();
  }

  @override
  void onGameResize(Vector2 newSize) {
    super.onGameResize(newSize);
    _updateCardLayout();
  }

  void _updateCardLayout() {
    // Clear previous cards to prevent duplicates on resize
    removeAll(children.whereType<NPCCardComponent>());

    final entries =
        relationships.isNotEmpty ? relationships : _placeholderRelationships();
    final cardCount = min(entries.length, _maxCards);
    if (cardCount == 0) {
      return;
    }

    final availableHeight = size.y - _topMargin - _bottomMargin;
    final cardWidth = size.x - 2 * _horizontalPadding;

    double y;
    final double spacing;

    if (cardCount > 1) {
      final totalCardHeight = cardCount * _cardHeight;
      spacing = max(
        _minSpacing,
        (availableHeight - totalCardHeight) / (cardCount - 1),
      );
      y = _topMargin;
    } else {
      spacing = 0;
      y = _topMargin + (availableHeight - _cardHeight) / 2;
    }

    for (var i = 0; i < cardCount; i++) {
      final rel = entries[i];
      final card = NPCCardComponent(
        relationship: rel,
        onTap: onOpenNetwork,
        position: Vector2(_horizontalPadding, y),
        size: Vector2(cardWidth, _cardHeight),
      );
      add(card);
      y += _cardHeight + spacing;
    }
  }

  List<RelationshipData> _placeholderRelationships() {
    return [
      RelationshipData(
        npcId: 'kaz',
        npcName: 'Kaz',
        affection: 4,
        trust: 6,
        attraction: 3,
        familiarity: 6,
        resentment: 2,
        state: 'Ally',
      ),
      RelationshipData(
        npcId: 'ila',
        npcName: 'Ila',
        affection: 2,
        trust: 5,
        attraction: 1,
        familiarity: 4,
        resentment: 3,
        state: 'Confidant',
      ),
      RelationshipData(
        npcId: 'fixer',
        npcName: 'Fixer',
        affection: 1,
        trust: 4,
        attraction: 0,
        familiarity: 5,
        resentment: 5,
        state: 'Contact',
      ),
    ];
  }

  @override
  void render(Canvas canvas) {
    // Angled silhouette, matching StatPanel's general shape but mirrored
    final path = Path()
      ..moveTo(10, 6)
      ..lineTo(size.x, 0)
      ..lineTo(size.x, size.y - 10)
      ..lineTo(size.x - 10, size.y - 6)
      ..lineTo(0, size.y)
      ..lineTo(0, size.y * 0.25)
      ..close();

    canvas.drawShadow(path, SynHudChrome.topBarShadowColor, 12, false);
    canvas.drawPath(
      path,
      Paint()..shader = _backgroundGradient.createShader(size.toRect()),
    );
    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = _borderWidth
        ..color = _borderColor,
    );

    final slash1 = Path()
      ..moveTo(size.x * 0.18, -8)
      ..lineTo(size.x * 0.34, size.y * 0.46)
      ..lineTo(size.x * 0.22, size.y * 0.46 + 14)
      ..close();
    canvas.drawPath(
      slash1,
      Paint()
        ..color = _slashColor.withOpacity(0.16)
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 4),
    );

    final slash2 = Path()
      ..moveTo(size.x * 0.70, size.y * 0.10)
      ..lineTo(size.x * 0.86, size.y * 0.58)
      ..lineTo(size.x * 0.74, size.y * 0.58 + 14)
      ..close();
    canvas.drawPath(
      slash2,
      Paint()
        ..color = _slashColor.withOpacity(0.10)
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 5),
    );

    final highlightRect = Rect.fromLTWH(
      10,
      size.y * 0.14,
      size.x - 20,
      size.y * 0.7,
    );
    canvas.drawRect(
      highlightRect,
      Paint()
        ..shader = const LinearGradient(
          colors: [Color(0x22000000), Color(0x11000000)],
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
        ).createShader(highlightRect),
    );
  }
}
