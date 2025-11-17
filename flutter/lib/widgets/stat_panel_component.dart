import 'package:flame/components.dart';
import 'package:flutter/material.dart';
import 'dart:math' as math;
import '../syn_game.dart';

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
  final Map<String, _StatRing> _statRings = {};
  late _PanelFrame _frame;

  @override
  Future<void> onLoad() async {
    // Panel frame (angled Persona border)
    _frame = _PanelFrame(size: size);
    add(_frame);

    final gameState = game.gameState;
    final stats = [
      _StatDescriptor(
        key: 'health',
        stat: 'Health',
        label: 'HP',
        value: gameState.health,
        color: const Color(0xFFFF4444),
      ),
      _StatDescriptor(
        key: 'wealth',
        stat: 'Wealth',
        label: '\$',
        value: gameState.wealth,
        color: const Color(0xFF44FF44),
      ),
      _StatDescriptor(
        key: 'charisma',
        stat: 'Charisma',
        label: 'CHR',
        value: gameState.charisma,
        color: const Color(0xFF00D9FF),
      ),
      _StatDescriptor(
        key: 'intelligence',
        stat: 'Intelligence',
        label: 'INT',
        value: gameState.intelligence,
        color: const Color(0xFFFFAA00),
      ),
      _StatDescriptor(
        key: 'wisdom',
        stat: 'Wisdom',
        label: 'WIS',
        value: gameState.wisdom,
        color: const Color(0xFFDD44FF),
      ),
      _StatDescriptor(
        key: 'strength',
        stat: 'Strength',
        label: 'STR',
        value: gameState.strength,
        color: const Color(0xFFFF8844),
      ),
    ];

    // Grid layout: 3 columns × 2 rows, centered within the angled frame
    const columns = 3;
    const rows = 2;
    const ringDiameter = 70.0;
    const horizontalInset = 26.0;
    const verticalInset = 44.0;
    final ringRadius = ringDiameter / 2;
    final widthAvailable =
        math.max(size.x - 2 * (horizontalInset + ringRadius), 0);
    final heightAvailable =
        math.max(size.y - 2 * (verticalInset + ringRadius), 0);
    final columnSpacing = columns > 1 ? widthAvailable / (columns - 1) : 0;
    final rowSpacing = rows > 1 ? heightAvailable / (rows - 1) : 0;

    for (var index = 0; index < stats.length; index++) {
      final descriptor = stats[index];
      final row = index ~/ columns;
      final col = index % columns;
      if (row >= rows) {
        break;
      }

      final position = Vector2(
        horizontalInset + ringRadius + columnSpacing * col,
        verticalInset + ringRadius + rowSpacing * row,
      );

      final ring = _StatRing(
        stat: descriptor.stat,
        label: descriptor.label,
        value: descriptor.value,
        maxValue: 100,
        color: descriptor.color,
        position: position,
      );
      _statRings[descriptor.key] = ring;
      add(ring);
    }

    // Title label (subtle)
    final title = TextComponent(
      text: 'STATS',
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
  }

  @override
  void update(double dt) {
    super.update(dt);
    final gameState = game.gameState;

    // Update stat values
    _statRings['health']?.updateValue(gameState.health);
    _statRings['wealth']?.updateValue(gameState.wealth);
    _statRings['charisma']?.updateValue(gameState.charisma);
    _statRings['intelligence']?.updateValue(gameState.intelligence);
    _statRings['wisdom']?.updateValue(gameState.wisdom);
    _statRings['strength']?.updateValue(gameState.strength);
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
        ..color = Colors.black.withOpacity(0.4)
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
        style: TextStyle(
          color: color,
          fontSize: 11,
          fontWeight: FontWeight.bold,
          height: 1.0,
        ),
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
        style: const TextStyle(
          color: Colors.white,
          fontSize: 9,
          height: 1.0,
        ),
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
        ..color = const Color(0xFF000000).withOpacity(0.65)
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
            const Color(0xFF1a1a1a).withOpacity(0.3),
            const Color(0xFF0a0a0a).withOpacity(0.2),
          ],
        ).createShader(Rect.fromLTWH(0, 0, size.x, size.y)),
    );

    // Border (cyan, angled)
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
      ..moveTo(innerOffset, innerOffset)
      ..lineTo(size.x - innerOffset, angleOffset + innerOffset)
      ..lineTo(size.x - innerOffset, size.y - innerOffset)
      ..lineTo(innerOffset, size.y - angleOffset - innerOffset)
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
