// File: flutter/lib/components/ui/effects/particle_system_component.dart
import 'dart:ui';

import 'package:flame/components.dart';
import 'package:flutter/material.dart';

class ParticleSystemComponent extends Component {
  ParticleSystemComponent();

  double _timer = 0;

  @override
  void update(double dt) {
    super.update(dt);
    _timer += dt;
  }

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final radius = 4 + 2 * (_timer % 1);
    final paint = Paint()..color = const Color(0xFF00D9FF).withValues(alpha: 0.6);
    canvas.drawCircle(const Offset(0, 0), radius, paint);
    canvas.drawCircle(const Offset(12, -8), radius * 0.8, paint);
    canvas.drawCircle(const Offset(-14, 6), radius * 0.6, paint);
  }
}
