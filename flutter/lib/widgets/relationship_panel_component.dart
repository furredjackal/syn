import 'package:flame/components.dart';
import 'package:flutter/material.dart';
import 'dart:math' as math;
import '../syn_game.dart';
import '../models/game_state.dart';

/// RelationshipPanel: Compact floating right panel showing active relationships.
///
/// Design:
/// - Angled Persona-style border (right edge)
/// - Compact footprint (280x280px, mirror of StatPanel)
/// - Shows up to 3 active relationships (most important)
/// - Each relationship has: name, state indicator, affection/trust gauges
/// - Color-coded by relationship state
/// - Minimal labels, focused on visual indicators
class RelationshipPanelComponent extends PositionComponent
    with HasGameReference<SynGame> {
  late _PanelFrame _frame;
  final List<_RelationshipRow> _rows = [];

  @override
  Future<void> onLoad() async {
    // Panel frame (angled Persona border)
    _frame = _PanelFrame(size: size);
    add(_frame);

    // Title label
    final title = TextComponent(
      text: 'BONDS',
      textRenderer: TextPaint(
        style: const TextStyle(
          color: Color(0xFF00D9FF),
          fontSize: 12,
          fontWeight: FontWeight.bold,
          letterSpacing: 1,
        ),
      ),
      position: Vector2(16, 8),
      anchor: Anchor.topLeft,
    );
    add(title);

    // Get up to 3 most important relationships
    final gameState = game.gameState;
    final relationships = gameState.relationships;

    if (relationships.isEmpty) {
      // Empty state
      final emptyText = TextComponent(
        text: 'No bonds yet',
        textRenderer: TextPaint(
          style: TextStyle(
            color: Colors.white.withOpacity(0.4),
            fontSize: 11,
            height: 1.4,
          ),
        ),
        position: Vector2(16, 45),
        anchor: Anchor.topLeft,
      );
      add(emptyText);
    } else {
      // Show top 3 relationships (sorted by affection + trust)
      final sorted = List<RelationshipData>.from(relationships);
      sorted.sort((a, b) {
        final scoreA = a.affection + a.trust;
        final scoreB = b.affection + b.trust;
        return scoreB.compareTo(scoreA);
      });

      double yOffset = 40;
      for (int i = 0; i < math.min(3, sorted.length); i++) {
        final rel = sorted[i];
        final row = _RelationshipRow(
          relationship: rel,
          position: Vector2(16, yOffset),
          size: Vector2(size.x - 32, 65),
        );
        add(row);
        _rows.add(row);
        yOffset += 72;
      }
    }
  }

  @override
  void update(double dt) {
    super.update(dt);
    // Rows auto-update their relationship data
  }
}

/// Single relationship row with name, state, and gauges
class _RelationshipRow extends PositionComponent {
  final RelationshipData relationship;

  _RelationshipRow({
    required this.relationship,
    required Vector2 position,
    required Vector2 size,
  }) : super(position: position, size: size);

  @override
  void render(Canvas canvas) {
    // Background
    canvas.drawRRect(
      RRect.fromRectAndRadius(
        Rect.fromLTWH(0, 0, size.x, size.y),
        const Radius.circular(4),
      ),
      Paint()
        ..color = _getStateColor(relationship.state).withOpacity(0.15)
        ..style = PaintingStyle.fill,
    );

    // Border
    canvas.drawRRect(
      RRect.fromRectAndRadius(
        Rect.fromLTWH(0, 0, size.x, size.y),
        const Radius.circular(4),
      ),
      Paint()
        ..color = _getStateColor(relationship.state).withOpacity(0.5)
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.5,
    );

    // NPC Name (bold, top-left)
    final namePainter = TextPainter(
      text: TextSpan(
        text: relationship.npcName.toUpperCase(),
        style: TextStyle(
          color: _getStateColor(relationship.state),
          fontSize: 12,
          fontWeight: FontWeight.bold,
          letterSpacing: 0.5,
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

    // Affection gauge (left side)
    const gaugeY = 28.0;
    const gaugeHeight = 6.0;
    const gaugeWidth = 55.0;

    _drawGauge(
      canvas,
      Offset(8, gaugeY),
      'AFF',
      relationship.affection,
      gaugeWidth,
      gaugeHeight,
      const Color(0xFFFF4488), // Pink/red for affection
    );

    // Trust gauge (right side)
    _drawGauge(
      canvas,
      Offset(size.x - gaugeWidth - 8, gaugeY),
      'TRU',
      relationship.trust,
      gaugeWidth,
      gaugeHeight,
      const Color(0xFF44FF88), // Green for trust
    );

    // Additional metrics (smaller, bottom)
    final metricsY = gaugeY + 16;
    final metricsText = '${relationship.familiarity.toStringAsFixed(1)}F '
        '${relationship.resentment.toStringAsFixed(1)}R';
    final metricsPainter = TextPainter(
      text: TextSpan(
        text: metricsText,
        style: TextStyle(
          color: Colors.white.withOpacity(0.6),
          fontSize: 9,
          height: 1.0,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    metricsPainter.paint(canvas, Offset(8, metricsY));
  }

  void _drawGauge(
    Canvas canvas,
    Offset position,
    String label,
    double value,
    double width,
    double height,
    Color color,
  ) {
    // Label
    final labelPainter = TextPainter(
      text: TextSpan(
        text: label,
        style: TextStyle(
          color: color,
          fontSize: 8,
          fontWeight: FontWeight.bold,
        ),
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
        ..color = Colors.black.withOpacity(0.3)
        ..style = PaintingStyle.fill,
    );

    // Gauge fill (value -10 to +10 mapped to 0 to 100%)
    final fillPercent = ((value + 10) / 20).clamp(0.0, 1.0);
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
          ..color = color.withOpacity(0.8)
          ..style = PaintingStyle.fill,
      );
    }

    // Value text
    final valuePainter = TextPainter(
      text: TextSpan(
        text: value.toStringAsFixed(0),
        style: TextStyle(
          color: color,
          fontSize: 7,
          height: 1.0,
        ),
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
        style: TextStyle(
          color: color,
          fontSize: 8,
          fontWeight: FontWeight.bold,
          height: 1.0,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();

    final textX = position.dx + (badgeWidth / 2) - (statePainter.width / 2);
    final textY = position.dy + (badgeHeight / 2) - (statePainter.height / 2);
    statePainter.paint(canvas, Offset(textX, textY));
  }

  Color _getStateColor(String state) {
    return switch (state) {
      'Stranger' => const Color(0xFF888888),
      'Acquaintance' => const Color(0xFFAAAAFF),
      'Friend' => const Color(0xFF44FF88),
      'CloseFriend' => const Color(0xFF00FF88),
      'BestFriend' => const Color(0xFF00FFFF),
      'RomanticInterest' => const Color(0xFFFF4488),
      'Partner' => const Color(0xFFFF00AA),
      'Spouse' => const Color(0xFFFFFFFF),
      'Rival' => const Color(0xFFFF6644),
      'Estranged' => const Color(0xFF884444),
      'BrokenHeart' => const Color(0xFFCC4488),
      _ => const Color(0xFF00D9FF),
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

/// Panel frame with angled Persona border (right side)
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
        ..color = const Color(0xFF000000).withOpacity(0.65)
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
            const Color(0xFF1a1a1a).withOpacity(0.3),
            const Color(0xFF0a0a0a).withOpacity(0.2),
          ],
        ).createShader(Rect.fromLTWH(0, 0, size.x, size.y)),
    );

    // Border (cyan, angled right)
    canvas.drawPath(
      bgPath,
      Paint()
        ..color = const Color(0xFF00D9FF)
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2.5,
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
        ..color = const Color(0xFF00D9FF).withOpacity(0.3)
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1,
    );
  }
}
