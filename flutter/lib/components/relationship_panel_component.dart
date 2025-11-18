import 'dart:math' as math;

import 'package:flame/components.dart';
import 'package:flutter/material.dart';
import '../models/game_state.dart';
import '../syn_game.dart';
import '../ui/syn_theme.dart';

class RelationshipPanelComponent extends PositionComponent
    with HasGameReference<SynGame> {
  final List<_RelationshipCardComponent> _cards = [];

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
      final card = _RelationshipCardComponent(
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

/// Single relationship row with name, state, and gauges
class _RelationshipCardComponent extends PositionComponent {
  final RelationshipData relationship;

  _RelationshipCardComponent({
    required this.relationship,
    required Vector2 position,
    required Vector2 size,
  }) : super(position: position, size: size);

  @override
  void render(Canvas canvas) {
    final bgRect = RRect.fromRectAndRadius(
      Rect.fromLTWH(0, 0, size.x, size.y),
      const Radius.circular(10),
    );
    canvas.drawRRect(
      bgRect,
      Paint()
        ..color = SynColors.bgPanel.withOpacity(0.9)
        ..style = PaintingStyle.fill,
    );
    canvas.drawRRect(
      bgRect,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = SynLayout.borderWidthLight
        ..color = _getStateColor(relationship.state).withOpacity(0.8),
    );

    // NPC Name (bold, top-left)
    final namePainter = TextPainter(
      text: TextSpan(
        text: relationship.npcName.toUpperCase(),
        style: SynTextStyles.body.copyWith(
          fontWeight: FontWeight.w700,
          letterSpacing: 1.2,
          color: SynColors.textPrimary,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    namePainter.paint(canvas, const Offset(8, 6));

    // Relationship State Badge (right side, top)
    final stateBadgeX = size.x - 68.0;
    final stateBadgeY = 6.0;
    _drawStateBadge(
      canvas,
      Offset(stateBadgeX, stateBadgeY),
      relationship.state,
    );

    const gaugeHeight = 6.0;
    final gaugeWidth = size.x - 32;
    const startY = 30.0;
    const rowSpacing = 18.0;
    final gauges = [
      ('AFF', relationship.affection, SynColors.accentRed, -10.0, 10.0),
      ('TRU', relationship.trust, SynColors.accentGreen, -10.0, 10.0),
      ('ATR', relationship.attraction, SynColors.accentViolet, -10.0, 10.0),
      ('FAM', relationship.familiarity, SynColors.primaryCyan, 0.0, 10.0),
      ('RES', relationship.resentment, SynColors.accentOrange, -10.0, 10.0),
    ];

    for (var i = 0; i < gauges.length; i++) {
      final spec = gauges[i];
      _drawGauge(
        canvas,
        Offset(8, startY + i * rowSpacing),
        spec.$1,
        spec.$2,
        gaugeWidth,
        gaugeHeight,
        spec.$3,
        minValue: spec.$4,
        maxValue: spec.$5,
      );
    }
  }

  void _drawGauge(
    Canvas canvas,
    Offset position,
    String label,
    double value,
    double width,
    double height,
    Color color, {
    double minValue = -10,
    double maxValue = 10,
  }) {
    // Label
    final labelPainter = TextPainter(
      text: TextSpan(
        text: label,
        style: SynTextStyles.chip.copyWith(color: color, fontSize: 9),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    labelPainter.paint(canvas, position);

    // Gauge background
    const gaugeY = 12.0;
    canvas.drawRRect(
      RRect.fromRectAndRadius(
        Rect.fromLTWH(position.dx, position.dy + gaugeY, width, height),
        const Radius.circular(2),
      ),
      Paint()
        ..color = SynColors.bgDark.withOpacity(0.4)
        ..style = PaintingStyle.fill,
    );

    // Gauge fill (value -10 to +10 mapped to 0 to 100%)
    final fillPercent = ((value - minValue) / (maxValue - minValue)).clamp(0.0, 1.0);
    final fillWidth = width * fillPercent;

    if (fillWidth > 0) {
      canvas.drawRRect(
        RRect.fromRectAndRadius(
          Rect.fromLTWH(
            position.dx,
            position.dy + gaugeY,
            fillWidth,
            height,
          ),
          const Radius.circular(2),
        ),
      Paint()
        ..color = color.withOpacity(0.85)
        ..style = PaintingStyle.fill,
    );
    }

    // Value text
    final valuePainter = TextPainter(
      text: TextSpan(
        text: value.toStringAsFixed(0),
        style: SynTextStyles.chip.copyWith(color: color, fontSize: 9),
      ),
      textDirection: TextDirection.ltr,
    )..layout();

    final valueX = position.dx + (width / 2) - (valuePainter.width / 2);
    final valueY = position.dy + gaugeY + 0.5;
    valuePainter.paint(canvas, Offset(valueX, valueY));
  }

  void _drawStateBadge(Canvas canvas, Offset position, String state) {
    const badgeWidth = 60.0;
    const badgeHeight = 16.0;

    final color = _getStateColor(state);

    // Badge background
    canvas.drawRRect(
      RRect.fromRectAndRadius(
        Rect.fromLTWH(position.dx, position.dy, badgeWidth, badgeHeight),
        const Radius.circular(2),
      ),
      Paint()
        ..color = color.withOpacity(0.2)
        ..style = PaintingStyle.fill,
    );

    // Badge border
    canvas.drawRRect(
      RRect.fromRectAndRadius(
        Rect.fromLTWH(position.dx, position.dy, badgeWidth, badgeHeight),
        const Radius.circular(2),
      ),
      Paint()
        ..color = color.withOpacity(0.6)
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1,
    );

    // State text
    final statePainter = TextPainter(
        text: TextSpan(
          text: _getStateLabel(state),
          style: SynTextStyles.chip.copyWith(color: color, fontSize: 9),
        ),
      textDirection: TextDirection.ltr,
    )..layout();

    final textX = position.dx + (badgeWidth / 2) - (statePainter.width / 2);
    final textY = position.dy + (badgeHeight / 2) - (statePainter.height / 2);
    statePainter.paint(canvas, Offset(textX, textY));
  }

  Color _getStateColor(String state) {
    return switch (state) {
      'Stranger' => SynColors.textMuted,
      'Acquaintance' => SynColors.accentIndigo,
      'Friend' => SynColors.accentGreen,
      'CloseFriend' => SynColors.primaryCyan,
      'BestFriend' => SynColors.accentCyan,
      'RomanticInterest' => SynColors.accentMagenta,
      'Partner' => SynColors.accentOrange,
      'Spouse' => SynColors.textPrimary,
      'Rival' => SynColors.accentRed,
      'Estranged' => SynColors.accentOrange.withOpacity(0.7),
      'BrokenHeart' => SynColors.accentMagenta.withOpacity(0.7),
      _ => SynColors.primaryCyan,
    };
  }

  String _getStateLabel(String state) {
    return switch (state) {
      'Stranger' => 'STR',
      'Acquaintance' => 'ACQ',
      'Friend' => 'FRI',
      'CloseFriend' => 'CF+',
      'BestFriend' => 'BF+',
      'RomanticInterest' => 'ROM',
      'Partner' => 'PRT',
      'Spouse' => 'SPO',
      'Rival' => 'RIV',
      'Estranged' => 'EST',
      'BrokenHeart' => 'BH',
      _ => '?',
    };
  }
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
