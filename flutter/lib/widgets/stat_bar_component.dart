import 'package:flame/components.dart';
import 'package:flutter/material.dart';
import 'package:syn/flutter/lib/syn_game.dart';

class StatBarComponent extends PositionComponent with HasGameRef<SynGame> {
  final String label;
  final int value;
  final int maxValue;
  final Color? customColor;

  StatBarComponent({
    required this.label,
    required this.value,
    this.maxValue = 100,
    this.customColor,
    Vector2? position,
    Vector2? size,
  }) : super(position: position, size: size);

  @override
  Future<void> onLoad() async {
    final barColor = _getBarColor();

    final textStyle = TextPaint(
      style: TextStyle(
        color: Colors.white.withOpacity(0.8),
        fontSize: 12,
      ),
    );

    final valueTextStyle = TextPaint(
      style: TextStyle(
        color: barColor,
        fontSize: 12,
      ),
    );

    add(TextComponent(
      text: label.toUpperCase(),
      textRenderer: textStyle,
      position: Vector2(0, 0),
    ));

    add(TextComponent(
      text: '$value/$maxValue',
      textRenderer: valueTextStyle,
      position: Vector2(size.x, 0),
      anchor: Anchor.topRight,
    ));

    final backgroundBar = RectangleComponent(
      position: Vector2(0, 20),
      size: Vector2(size.x, 12),
      paint: Paint()..color = Colors.white.withOpacity(0.1),
    );
    add(backgroundBar);

    final foregroundBar = RectangleComponent(
      position: Vector2(0, 20),
      size: Vector2(size.x * (value / maxValue).clamp(0.0, 1.0), 12),
      paint: Paint()..color = barColor,
    );
    add(foregroundBar);
  }

  Color _getBarColor() {
    if (customColor != null) return customColor!;

    final percentage = (value / maxValue).clamp(0.0, 1.0);
    if (percentage < 0.33) {
      return const Color(0xFFFF4444);
    } else if (percentage < 0.66) {
      return const Color(0xFFFFAA00);
    } else {
      return const Color(0xFF00FF00);
    }
  }
}
