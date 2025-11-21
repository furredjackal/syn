import 'package:flame/components.dart';
import 'package:flutter/painting.dart';

/// Horizontal stat bar with a simple neon gradient fill.
class StatBarComponent extends PositionComponent {
  StatBarComponent({
    this.value = 0.5,
    super.position,
    Vector2? size,
  }) : super(size: size ?? Vector2(140, 10));

  double value;

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final clamped = value.clamp(0.0, 1.0);
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    final radius = Radius.circular(size.y / 2);

    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, radius),
      Paint()..color = const Color(0x22111B2C),
    );

    canvas.drawRRect(
      RRect.fromRectAndRadius(
        Rect.fromLTWH(0, 0, size.x * clamped, size.y),
        radius,
      ),
      Paint()
        ..shader = const LinearGradient(
          colors: [
            Color(0xFF00D9FF),
            Color(0xFF7B5CFF),
          ],
        ).createShader(rect),
    );
  }
}
