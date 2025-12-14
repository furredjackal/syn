import 'dart:math';
import 'dart:ui';
import 'package:flame/components.dart';
import 'package:flame/game.dart';
import 'package:flame/particles.dart';
import 'package:flutter/material.dart' hide Image;

import '../../../dev_tools/inspectable_mixin.dart';

/// Data motes that float upward (vertical lines)
/// 
/// Visual: 5-10px vertical lines (cyan/magenta) that spawn at bottom,
/// float upward, fade out over ~4s
class ParticleSystemComponent extends Component with InspectableMixin {
  double _timeSinceLastSpawn = 0;
  final double _spawnInterval = 0.1; // Spawn every 0.1s
  final Random _random = Random();
  Vector2 _gameSize = Vector2.zero();
  int _particleCount = 0;

  @override
  String get inspectorName => 'ParticleSystem';

  @override
  String get inspectorCategory => 'Effects';

  @override
  List<InspectableProperty> get inspectorProperties => [
    InspectableProperty.number('SpawnInterval', _spawnInterval, min: 0.01, max: 1.0),
    InspectableProperty.number('ParticleCount', _particleCount.toDouble()),
    InspectableProperty.number('TimeSinceSpawn', _timeSinceLastSpawn, min: 0, max: 1),
    InspectableProperty.size('GameSize', Size(_gameSize.x, _gameSize.y)),
  ];

  @override
  Future<void> onLoad() async {
    priority = -5; // In front of background, behind UI
  }

  @override
  void onMount() {
    super.onMount();
    if (parent is FlameGame) {
      _gameSize = (parent as FlameGame).size;
    }
  }

  @override
  void update(double dt) {
    super.update(dt);
    
    // Update game size in case of resize
    if (parent is FlameGame) {
      _gameSize = (parent as FlameGame).size;
    }
    
    _timeSinceLastSpawn += dt;
    
    if (_timeSinceLastSpawn >= _spawnInterval) {
      _spawnParticle();
      _timeSinceLastSpawn = 0;
    }
  }

  void _spawnParticle() {
    if (_gameSize == Vector2.zero()) return;
    
    final spawnX = _random.nextDouble() * _gameSize.x;
    final spawnY = _gameSize.y; // Start at bottom
    
    final lineHeight = 5 + _random.nextDouble() * 5; // 5-10px
    final lifespan = 4.0; // 4 seconds to fade out
    
    // Random cyan or magenta
    final color = _random.nextBool()
        ? const Color(0xFF00E6FF) // Cyan
        : const Color(0xFFFF006E); // Magenta
    
    final particle = MovingParticle(
      from: Vector2(spawnX, spawnY),
      to: Vector2(spawnX, spawnY - 200), // Move upward 200px
      lifespan: lifespan,
      child: ComputedParticle(
        lifespan: lifespan,
        renderer: (canvas, particle) {
          final progress = particle.progress;
          final opacity = (1 - progress).clamp(0.0, 1.0);
          final scale = 1.0 - (progress * 0.5); // Scale down to 50%
          
          final paint = Paint()
            ..color = color.withValues(alpha: opacity * 0.8)
            ..strokeWidth = 1.5 * scale
            ..style = PaintingStyle.stroke;
          
          // Draw vertical line
          canvas.drawLine(
            Offset(0, 0),
            Offset(0, lineHeight * scale),
            paint,
          );
        },
      ),
    );
    
    // Add particle directly to parent game
    // In Flame, ParticleSystemComponent wraps a single particle and handles its lifecycle
    if (parent is FlameGame) {
      final wrapper = _SingleParticleComponent(particle);
      (parent as FlameGame).add(wrapper);
    }
  }
}

/// Simple wrapper component that renders a single particle
class _SingleParticleComponent extends Component {
  final Particle _particle;
  
  _SingleParticleComponent(this._particle) {
    priority = -4; // Just above the spawner
  }

  @override
  void update(double dt) {
    super.update(dt);
    _particle.update(dt);
    
    // Remove this component when particle is done
    if (_particle.shouldRemove) {
      removeFromParent();
    }
  }

  @override
  void render(Canvas canvas) {
    _particle.render(canvas);
  }
}
