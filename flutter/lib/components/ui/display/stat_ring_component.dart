import 'package:flame/components.dart';
import 'package:flame/effects.dart';
import 'package:flutter/material.dart';
import '../../../syn_game.dart';

class StatRingComponent extends PositionComponent with HasGameReference<SynGame> {
  final String label;
  final double value;
  final double maxValue;
  final Color color;

  StatRingComponent({
    required this.label,
    required this.value,
    this.maxValue = 100,
    required this.color,
    Vector2? position,
    double? radius,
  }) : super(
          position: position,
          size: Vector2.all(radius != null ? radius * 2 : 80),
        );

  @override
  Future<void> onLoad() async {
    final percentage = (value / maxValue).clamp(0.0, 1.0);

    // Background ring
    add(CircleComponent(
      radius: size.x / 2,
      paint: Paint()
        ..color = Colors.white10
        ..style = PaintingStyle.stroke
        ..strokeWidth = 4,
    ));

    // Progress ring
    add(CircleComponent(
      radius: size.x / 2,
      paint: Paint()
        ..color = color
        ..style = PaintingStyle.stroke
        ..strokeWidth = 4,
      angle: -3.14 / 2,
    )..add(RotateEffect.by(
        percentage * 2 * 3.14,
        EffectController(duration: 0.5),
      )));

    // Center content
    add(TextComponent(
      text: value.toStringAsFixed(0),
      textRenderer: TextPaint(
        style: TextStyle(color: color, fontSize: 24),
      ),
      anchor: Anchor.center,
      position: Vector2(size.x / 2, size.y / 2 - 10),
    ));

    add(TextComponent(
      text: label,
      textRenderer: TextPaint(
        style: const TextStyle(color: Colors.grey, fontSize: 12),
      ),
      anchor: Anchor.center,
      position: Vector2(size.x / 2, size.y / 2 + 10),
    ));
  }
}
