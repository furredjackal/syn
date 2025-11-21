// File: flutter/lib/components/ui/display/mood_pulse_component.dart
import 'dart:math' as math;
import 'dart:ui';

import 'package:flame/components.dart';
import 'package:flutter/material.dart';

class MoodPulseComponent extends PositionComponent {
  MoodPulseComponent({super.position, super.size});

  double mood = 0.5;
  double pulse = 0.0;

  @override
  void update(double dt) {
    super.update(dt);
    pulse = (pulse + dt * 2) % (math.pi * 2);
  }

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(12)),
      Paint()..color = const Color(0xFF101A2E),
    );

    final waveHeight = (size.y / 4) * (0.5 + 0.5 * (mood.clamp(0.0, 1.0)));
    final offsetY = (size.y / 2) + waveHeight * (0.5 * (1 + math.sin(pulse)));

    final path = Path()..moveTo(0, offsetY);
    for (double x = 0; x <= size.x; x += 6) {
      final y =
          offsetY + waveHeight * 0.3 * math.sin(x / size.x * 6.28318 + pulse);
      path.lineTo(x, y);
    }
    canvas.drawPath(
      path,
      Paint()
        ..color = const Color(0xFF00D9FF)
        ..strokeWidth = 3
        ..style = PaintingStyle.stroke,
    );
  }
}
