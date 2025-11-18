import 'package:flame/components.dart';
import 'package:flutter/material.dart';
import 'dart:math' as math;
import '../syn_game.dart';
import '../models/life_stage_profile.dart';

/// Flame-based particle system component for mood-driven environmental effects
class ParticleSystemComponent extends PositionComponent
    with HasGameReference<SynGame> {
  bool isEnabled = true;
  /// Current mood value (-10 to +10)
  int currentMood = 0;

  /// Current life stage
  String lifeStage = 'adult';

  /// Active particles
  final List<ParticleData> particles = [];

  /// Emission state
  double emissionRate = 0; // Particles per second

  ParticleSystemComponent({
    Vector2? position,
    Vector2? size,
  }) : super(position: position, size: size);

  @override
  Future<void> onLoad() async {
    // Set up initial size to match game bounds
    size = game.size;
  }

  @override
  void update(double dt) {
    if (!isEnabled) {
      return;
    }
    super.update(dt);

    // Update emission rate based on mood and life stage
    _updateEmissionRate();

    // Emit new particles
    _emitParticles(dt);

    // Update all existing particles
    for (var i = particles.length - 1; i >= 0; i--) {
      particles[i].update(dt);
      if (particles[i].isDead) {
        particles.removeAt(i);
      }
    }
  }

  /// Update emission rate based on mood tier and life stage
  void _updateEmissionRate() {
    // Determine mood tier
    final moodTier = _getMoodTier(currentMood);
    final profile = LifeStageProfile.forStage(lifeStage);

    // Calculate emission multiplier based on mood
    double moodMultiplier = 1.0;
    switch (moodTier) {
      case 'despair':
        moodMultiplier = 0.3; // Very little particle emission in despair
        break;
      case 'troubled':
        moodMultiplier = 0.6; // Reduced emission
        break;
      case 'neutral':
        moodMultiplier = 1.0; // Baseline
        break;
      case 'content':
        moodMultiplier = 1.4; // Increased emission
        break;
      case 'euphoric':
        moodMultiplier = 2.0; // Heavy particle emission in euphoria
        break;
    }

    // Apply life stage emission rate + mood multiplier
    emissionRate = profile.particleEmissionRate * moodMultiplier;
  }

  /// Emit new particles based on emission rate
  void _emitParticles(double dt) {
    if (emissionRate > 0) {
      final particlesToEmit = (emissionRate * dt).floor();
      for (int i = 0; i < particlesToEmit; i++) {
        particles.add(_createParticle());
      }

      // Handle fractional emission
      final fractionalRate = (emissionRate * dt) - particlesToEmit;
      if (math.Random().nextDouble() < fractionalRate) {
        particles.add(_createParticle());
      }
    }
  }

  /// Create a new particle based on current life stage
  ParticleData _createParticle() {
    final moodTier = _getMoodTier(currentMood);

    // Get particle properties from mood tier
    final (particleColor, particleSize) = _getParticleProperties(moodTier);

    // Random emitter position (slightly spread across screen)
    final emitterX = math.Random().nextDouble() * size.x;
    final emitterY = math.Random().nextDouble() * 20; // Top of screen

    // Velocity based on mood
    final baseSpeed = 20.0 + (currentMood.abs() * 5.0);
    final angle = math.Random().nextDouble() * math.pi; // Emit downward
    final vx = math.cos(angle - math.pi / 2) * baseSpeed;
    final vy =
        math.sin(angle - math.pi / 2) * baseSpeed + 10; // Slight downward bias

    return ParticleData(
      position: Vector2(emitterX, emitterY),
      velocity: Vector2(vx, vy),
      lifetime: 2.0 + (currentMood.abs() * 0.1), // Euphoria = longer lifetime
      color: particleColor,
      size: particleSize,
    );
  }

  /// Get particle color and size based on mood and life stage
  (Color, double) _getParticleProperties(String moodTier) {
    Color baseColor = const Color(0xFF00D9FF);
    double size = 2.0;

    // Mood-based color modulation
    switch (moodTier) {
      case 'despair':
        baseColor = Colors.red.withOpacity(0.6);
        size = 1.5;
        break;
      case 'troubled':
        baseColor = Colors.orange.withOpacity(0.7);
        size = 1.8;
        break;
      case 'neutral':
        baseColor = const Color(0xFF00D9FF).withOpacity(0.8);
        size = 2.0;
        break;
      case 'content':
        baseColor = Colors.green.withOpacity(0.8);
        size = 2.2;
        break;
      case 'euphoric':
        baseColor = Colors.purple.withOpacity(0.9);
        size = 2.5;
        break;
    }

    return (baseColor, size);
  }

  /// Get mood tier from mood value
  String _getMoodTier(int mood) {
    if (mood <= -6) return 'despair';
    if (mood < -2) return 'troubled';
    if (mood < 2) return 'neutral';
    if (mood < 6) return 'content';
    return 'euphoric';
  }

  /// Burst emission for event outcomes (large stat change or special event)
  void burstParticles({required int count, Color? color}) {
    final burstColor = color ?? const Color(0xFF00D9FF);

    // Create centered burst
    for (int i = 0; i < count; i++) {
      final angle = (i / count) * 2 * math.pi;
      final speed = 80.0 + math.Random().nextDouble() * 120;
      final vx = math.cos(angle) * speed;
      final vy = math.sin(angle) * speed;

      particles.add(ParticleData(
        position: Vector2(size.x / 2, size.y / 2),
        velocity: Vector2(vx, vy),
        lifetime: 1.5,
        color: burstColor,
        size: 3.0,
      ));
    }
  }

  /// Update mood value and trigger visual changes
  void updateMood(int newMood) {
    currentMood = newMood.clamp(-10, 10);
    // Emission rate will be recalculated in next update() call
  }

  /// Update life stage
  void updateLifeStage(String newStage) {
    lifeStage = newStage;
  }

  @override
  void render(Canvas canvas) {
    if (!isEnabled) {
      return;
    }
    super.render(canvas);

    // Draw all particles
    for (var particle in particles) {
      particle.render(canvas);
    }
  }

  void setActive(bool active) {
    isEnabled = active;
  }
}

/// Individual particle data
class ParticleData {
  Vector2 position;
  Vector2 velocity;
  double lifetime;
  final double maxLifetime;
  final Color color;
  final double size;

  bool get isDead => lifetime <= 0;
  double get opacity => (lifetime / maxLifetime).clamp(0.0, 1.0);

  ParticleData({
    required this.position,
    required this.velocity,
    required this.lifetime,
    required this.color,
    required this.size,
  }) : maxLifetime = lifetime;

  void update(double dt) {
    position += velocity * dt;
    lifetime -= dt;

    // Apply gravity and damping
    velocity.y += 10 * dt; // Gravity
    velocity *= 0.98; // Damping
  }

  void render(Canvas canvas) {
    final paint = Paint()
      ..color = color.withOpacity(opacity * 0.8)
      ..style = PaintingStyle.fill;

    canvas.drawCircle(
      Offset(position.x, position.y),
      size,
      paint,
    );
  }
}
