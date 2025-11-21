import 'package:flame/components.dart';
import 'package:flutter/material.dart';

import '../../../models/game_state.dart';
import '../../../syn_game.dart';
import '../display/stat_bar_component.dart';
import '../syn_theme.dart';

class StatPanelComponent extends PositionComponent
    with HasGameReference<SynGame> {
  StatPanelComponent({required this.gameState, super.position, super.size});

  final GameState gameState;

  // Visual polish constants, derived from syn_theme.dart
  static final _backgroundGradient = LinearGradient(
    colors: [SynColors.bgDark, SynColors.bgPanel],
    begin: Alignment.topCenter,
    end: Alignment.bottomCenter,
  );
  static const _borderColor = SynHudChrome.topBarBorderColorPrimary;
  static const _slashColor = SynHudChrome.topBarBorderColorPrimary;
  static const _borderWidth = SynLayout.borderWidthHeavy;

  @override
  Future<void> onLoad() async {
    await super.onLoad();

    final stats = _buildStats();
    var y = 24.0; // Increased top padding
    for (final stat in stats) {
      final row = _StatRow(
        label: stat.$1,
        value: stat.$2,
        rawValue: stat.$3,
      )
        ..position = Vector2(20, y) // Increased horizontal padding
        ..size = Vector2(size.x - 40, 52); // Adjusted height for spacing
      add(row);
      y += 58; // Adjusted vertical stride
    }
  }

  List<(String, double, int)> _buildStats() {
    return [
      ('HEALTH', gameState.health / 100, gameState.health),
      ('WEALTH', gameState.wealth / 100, gameState.wealth),
      ('CHARISMA', gameState.charisma / 100, gameState.charisma),
      ('INTELLECT', gameState.intelligence / 100, gameState.intelligence),
      ('STABILITY', gameState.stability / 100, gameState.stability),
      ('CREATIVITY', gameState.wisdom / 100, gameState.wisdom),
    ];
  }

  @override
  void render(Canvas canvas) {
    final path = Path()
      ..moveTo(16, 0)
      ..lineTo(size.x - 10, 6)
      ..lineTo(size.x, size.y * 0.25)
      ..lineTo(size.x, size.y - 10)
      ..lineTo(size.x - 16, size.y)
      ..lineTo(10, size.y - 6)
      ..lineTo(0, size.y * 0.15)
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

    // Subtler interior slashes
    final slash1 = Path()
      ..moveTo(size.x * 0.70, -10)
      ..lineTo(size.x * 0.9, size.y * 0.3)
      ..lineTo(size.x * 0.76, size.y * 0.3 + 18)
      ..close();
    canvas.drawPath(
      slash1,
      Paint()
        ..color = _slashColor.withOpacity(0.14)
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 3),
    );

    final slash2 = Path()
      ..moveTo(size.x * 0.12, size.y * 0.12)
      ..lineTo(size.x * 0.3, size.y * 0.45)
      ..lineTo(size.x * 0.18, size.y * 0.45 + 16)
      ..close();
    canvas.drawPath(
      slash2,
      Paint()
        ..color = _slashColor.withOpacity(0.08)
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 5),
    );
  }
}

class _StatRow extends PositionComponent {
  _StatRow({
    required this.label,
    required this.value,
    required this.rawValue,
  });

  final String label;
  final double value;
  final int rawValue;

  @override
  Future<void> onLoad() async {
    await super.onLoad();

    // Polished stat label
    final labelText = TextComponent(
      text: label,
      textRenderer: TextPaint(
        style: SynTextStyles.h2Strip.copyWith(
          fontSize: 13,
          color: const Color(0xFFE5ECF5),
          letterSpacing: 1.8,
        ),
      ),
      position: Vector2(0, 0),
    );
    add(labelText);

    final bar = StatBarComponent(
      value: value,
      position: Vector2(0, 24),
      size: Vector2(size.x, 10),
    );
    add(bar);

    // Right-aligned numeric value
    final numeric = TextComponent(
      text: rawValue.toString(),
      textRenderer: TextPaint(
        style: SynTextStyles.body.copyWith(
          color: SynColors.textSubtle,
          fontSize: 14,
          fontWeight: FontWeight.w600,
        ),
      ),
      anchor: Anchor.topRight,
      position: Vector2(size.x, 0),
    );
    add(numeric);
  }
}
