import 'dart:math' as math;

import 'package:flame/components.dart';
import 'package:flutter/material.dart';
import '../syn_game.dart';
import '../ui/syn_theme.dart';

/// StatPanel: Compact floating left panel with circular stat rings.
///
/// Design:
/// - Angled Persona-style border (left edge)
/// - Compact footprint (280x280px)
/// - Circular stat rings (3x2 grid: Health, Wealth, Charisma, Intelligence, Wisdom, Strength)
/// - Each ring shows stat value as filled percentage
/// - Minimal labels (icon-like abbreviations)
/// - No full-height stretching, no rigid frames
class StatPanelComponent extends PositionComponent
    with HasGameReference<SynGame> {
  final List<_StatRing> _rings = [];
  final Map<String, _StatRing> _ringMap = {};

  @override
  Future<void> onLoad() async {
    _buildPanelBackground();
    _buildRings();
  }

  void _buildPanelBackground() {
    add(_PanelFrame(size: size));
  }

  void _buildRings() {
    final state = game.gameState;
    final stats = [
      _StatDescriptor(
        key: 'health',
        stat: 'Health',
        label: 'HP',
        value: state.health,
        color: SynColors.accentRed,
      ),
      _StatDescriptor(
        key: 'mood',
        stat: 'Mood',
        label: 'MOOD',
        value: state.mood,
        color: moodToColor(state.mood),
      ),
      _StatDescriptor(
        key: 'wealth',
        stat: 'Wealth',
        label: '\$',
        value: state.wealth,
        color: SynColors.accentGreen,
      ),
      _StatDescriptor(
        key: 'charisma',
        stat: 'Charisma',
        label: 'CHA',
        value: state.charisma,
        color: SynColors.accentCyan,
      ),
      _StatDescriptor(
        key: 'intelligence',
        stat: 'Intelligence',
        label: 'INT',
        value: state.intelligence,
        color: SynColors.accentGold,
      ),
      _StatDescriptor(
        key: 'wisdom',
        stat: 'Wisdom',
        label: 'WIS',
        value: state.wisdom,
        color: SynColors.accentMagenta,
      ),
      _StatDescriptor(
        key: 'strength',
        stat: 'Strength',
        label: 'STR',
        value: state.strength,
        color: SynColors.accentOrange,
      ),
      _StatDescriptor(
        key: 'stability',
        stat: 'Stability',
        label: 'STB',
        value: state.stability,
        color: SynColors.accentIndigo,
      ),
    ];

    final columns = 2;
    final rows = 4;
    final horizontalPadding = size.x * 0.1;
    final verticalPadding = size.y * 0.12;
    final cellWidth = (size.x - horizontalPadding * 2) / columns;
    final cellHeight = (size.y - verticalPadding * 2) / rows;

    for (var i = 0; i < stats.length; i++) {
      final descriptor = stats[i];
      final row = i ~/ columns;
      final col = i % columns;
      final center = Vector2(
        horizontalPadding + cellWidth * col + cellWidth / 2,
        verticalPadding + cellHeight * row + cellHeight / 2,
      );

      final ring = _StatRing(
        stat: descriptor.stat,
        label: descriptor.label,
        value: descriptor.value,
        maxValue: 100,
        color: descriptor.color,
        position: center,
      );
      add(ring);
      _rings.add(ring);
      _ringMap[descriptor.key] = ring;
    }
  }

  @override
  void update(double dt) {
    super.update(dt);
    final gameState = game.gameState;
    _ringMap['health']?.updateValue(gameState.health);
    _ringMap['mood']?.updateValue(gameState.mood);
    _ringMap['wealth']?.updateValue(gameState.wealth);
    _ringMap['charisma']?.updateValue(gameState.charisma);
    _ringMap['intelligence']?.updateValue(gameState.intelligence);
    _ringMap['wisdom']?.updateValue(gameState.wisdom);
    _ringMap['strength']?.updateValue(gameState.strength);
    _ringMap['stability']?.updateValue(gameState.stability);
  }
}

class _StatDescriptor {
  final String key;
  final String stat;
  final String label;
  final int value;
  final Color color;

  const _StatDescriptor({
    required this.key,
    required this.stat,
    required this.label,
    required this.value,
    required this.color,
  });
}

/// Individual circular stat ring with percentage fill
class _StatRing extends PositionComponent {
  final String stat;
  final String label;
  int value;
  final int maxValue;
  final Color color;

  _StatRing({
    required this.stat,
    required this.label,
    required this.value,
    required this.maxValue,
    required this.color,
    required Vector2 position,
  }) : super(
          position: position,
          size: Vector2(70, 70),
          anchor: Anchor.center,
        );

  void updateValue(int newValue) {
    value = newValue.clamp(0, maxValue);
  }

  @override
  void render(Canvas canvas) {
    const ringRadius = 28.0;
    const ringWidth = 4.0;
    final center = Offset(size.x / 2, size.y / 2);

    // Calculate fill percentage
    final fillPercent = value.clamp(0, maxValue) / maxValue;
    final fillAngle = (fillPercent * 360) * (math.pi / 180);

    // Background ring (dark, unfilled portion)
    canvas.drawCircle(
      center,
      ringRadius,
      Paint()
        ..color = SynColors.bgDark.withOpacity(0.4)
        ..style = PaintingStyle.stroke
        ..strokeWidth = ringWidth,
    );

    // Filled ring (colored arc)
    final filledPaint = Paint()
      ..color = color
      ..style = PaintingStyle.stroke
      ..strokeWidth = ringWidth
      ..strokeCap = StrokeCap.round;

    final rect = Rect.fromCircle(center: center, radius: ringRadius);
    canvas.drawArc(
      rect,
      -math.pi / 2, // Start from top
      fillAngle,
      false,
      filledPaint,
    );

    // Center label (abbreviated stat name)
    final labelPainter = TextPainter(
      text: TextSpan(
        text: label,
        style: SynTextStyles.chip.copyWith(color: color),
      ),
      textDirection: TextDirection.ltr,
    )..layout();

    final labelX = center.dx - (labelPainter.width / 2);
    final labelY = center.dy - (labelPainter.height / 2);
    labelPainter.paint(canvas, Offset(labelX, labelY));

    // Value text below (small)
    final valuePainter = TextPainter(
      text: TextSpan(
        text: value.toString(),
        style: SynTextStyles.body.copyWith(fontSize: 10),
      ),
      textDirection: TextDirection.ltr,
    )..layout();

    final valueX = center.dx - (valuePainter.width / 2);
    final valueY = center.dy + 10;
    valuePainter.paint(canvas, Offset(valueX, valueY));
  }
}

/// Panel frame with angled Persona border (left side)
class _PanelFrame extends PositionComponent {
  _PanelFrame({required Vector2 size}) : super(size: size);

  @override
  void render(Canvas canvas) {
    const angleOffset = 12.0;

    // Background fill (dark semi-transparent)
    final bgPath = Path()
      ..moveTo(0, 0)
      ..lineTo(size.x, angleOffset)
      ..lineTo(size.x, size.y)
      ..lineTo(0, size.y - angleOffset)
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
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
          colors: [
            SynColors.bgDark.withOpacity(0.4),
            SynColors.bgPanel.withOpacity(0.25),
          ],
        ).createShader(Rect.fromLTWH(0, 0, size.x, size.y)),
    );

    // Border (cyan, angled)
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
      ..moveTo(innerOffset, innerOffset)
      ..lineTo(size.x - innerOffset, angleOffset + innerOffset)
      ..lineTo(size.x - innerOffset, size.y - innerOffset)
      ..lineTo(innerOffset, size.y - angleOffset - innerOffset)
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
