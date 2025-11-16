import 'package:flame/components.dart';
import 'package:flutter/material.dart';
import 'dart:math' as math;
import '../syn_game.dart';

class StatBarComponent extends PositionComponent with HasGameReference<SynGame> {
  final String label;
  int value;
  final int maxValue;
  final Color? customColor;

  // Animation state
  late int displayedValue;
  late RectangleComponent foregroundBar;
  late TextComponent valueText;
  double _counterElapsed = 0;
  double _counterDuration = 0;
  int _counterStartValue = 0;
  int _counterEndValue = 0;

  StatBarComponent({
    required this.label,
    required int value,
    this.maxValue = 100,
    this.customColor,
    Vector2? position,
    Vector2? size,
  })  : value = value,
        super(position: position, size: size);

  @override
  Future<void> onLoad() async {
    displayedValue = value;
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
        fontWeight: FontWeight.bold,
      ),
    );

    add(TextComponent(
      text: label.toUpperCase(),
      textRenderer: textStyle,
      position: Vector2(0, 0),
    ));

    valueText = TextComponent(
      text: '$displayedValue/$maxValue',
      textRenderer: valueTextStyle,
      position: Vector2(size.x - 4, 0),
      anchor: Anchor.topRight,
    );
    add(valueText);

    final backgroundBar = RectangleComponent(
      position: Vector2(0, 20),
      size: Vector2(size.x, 12),
      paint: Paint()..color = Colors.white.withOpacity(0.1),
    );
    add(backgroundBar);

    foregroundBar = RectangleComponent(
      position: Vector2(0, 20),
      size: Vector2(size.x * (displayedValue / maxValue).clamp(0.0, 1.0), 12),
      paint: Paint()..color = barColor,
    );
    add(foregroundBar);

    _counterStartValue = value;
    _counterEndValue = value;
  }

  void updateValue(int newValue) {
    final clampedValue = newValue.clamp(0, maxValue).toInt();
    if (clampedValue == value) return;
    final previous = value;
    value = clampedValue;
    _animateValueChange(previous, clampedValue);
  }

  /// Animates the transition from previous value to new value.
  /// Includes counter tick animation, bar fill, and delta indicator.
  void _animateValueChange(int from, int to) {
    final delta = to - from;
    if (delta == 0) return;

    final barColor = _getBarColor(to);
    final isIncrease = delta > 0;
    final deltaColor =
        isIncrease ? const Color(0xFF00FF00) : const Color(0xFFFF4444);

    // 1. Animate value counter (0.5s tick animation)
    _animateCounter(from, to, 0.5);

    // 2. Animate bar fill (parallel with counter, 0.5s)
    _animateBarFill(to, 0.5, barColor);

    // 3. Show floating delta indicator (1.0s float-up animation)
    _showDeltaIndicator(delta, deltaColor);

    // 4. Particle burst on large changes (if delta > 5)
    if (delta.abs() >= 5) {
      _burstParticles(delta, barColor);
    }
  }

  /// Counter tick animation: interpolates displayed value smoothly.
  void _animateCounter(int from, int to, double duration) {
    _counterElapsed = 0;
    _counterDuration = duration;
    _counterStartValue = from;
    _counterEndValue = to;
  }

  /// Bar fill animation: extends/shrinks the foreground bar width.
  void _animateBarFill(int targetValue, double duration, Color targetColor) {
    final targetWidth = size.x * (targetValue / maxValue).clamp(0.0, 1.0);
    final startWidth = foregroundBar.size.x;
    double elapsed = 0;

    // Create a custom update function
    void barUpdate(double dt) {
      elapsed += dt;
      final progress = (elapsed / duration).clamp(0.0, 1.0);
      final easeProgress = _easeOutCubic(progress);
      final newWidth = startWidth + (targetWidth - startWidth) * easeProgress;
      foregroundBar.size = Vector2(newWidth, foregroundBar.size.y);
    }

    // Attach to a temporary component for update calls
    final animator = _BarAnimator(barUpdate, duration);
    add(animator);
    foregroundBar.paint.color = targetColor;
  }

  /// Shows a floating delta indicator (e.g., "+10") that floats up and fades.
  void _showDeltaIndicator(int delta, Color color) {
    final sign = delta > 0 ? '+' : '';
    final deltaText = _FloatingDeltaText(
      text: '$sign$delta',
      color: color,
      duration: 1.0,
      floatDistance: 20,
      startPosition: Vector2(size.x / 2, 28),
    );

    add(deltaText);
  }

  /// Spawns particle burst at the bar location.
  void _burstParticles(int delta, Color color) {
    final particleCount = (delta.abs() / 5).ceil().clamp(3, 12);
    final isIncrease = delta > 0;
    const burstRadius = 8.0;

    for (int i = 0; i < particleCount; i++) {
      final angle = (i / particleCount) * 2 * math.pi;
      final speedX = math.cos(angle) * burstRadius;
      final speedY = (isIncrease ? math.sin(angle) - 1 : math.sin(angle) + 1) *
          burstRadius;

      final particle = _Particle(
        position: Vector2(size.x / 2, 26),
        initialVelocity: Vector2(speedX, speedY),
        color: color,
        lifetime: 0.6,
      );

      add(particle);
    }
  }

  /// Easing function: ease out cubic
  double _easeOutCubic(double t) {
    return 1 - math.pow(1 - t, 3) as double;
  }

  Color _getBarColor([int? valueOverride]) {
    if (customColor != null) return customColor!;

    final currentValue = valueOverride ?? value;
    final percentage = (currentValue / maxValue).clamp(0.0, 1.0);
    if (percentage < 0.33) {
      return const Color(0xFFFF4444);
    } else if (percentage < 0.66) {
      return const Color(0xFFFFAA00);
    } else {
      return const Color(0xFF00FF00);
    }
  }

  @override
  void update(double dt) {
    super.update(dt);

    // Handle counter animation
    if (_counterDuration > 0 && _counterElapsed < _counterDuration) {
      _counterElapsed += dt;
      final progress = (_counterElapsed / _counterDuration).clamp(0.0, 1.0);
      final easeProgress = _easeOutCubic(progress);
      displayedValue =
          (_counterStartValue + (_counterEndValue - _counterStartValue) * easeProgress)
              .round();

      // Update text
      final valueTextStyle = TextPaint(
        style: TextStyle(
          color: _getBarColor(displayedValue),
          fontSize: 12,
          fontWeight: FontWeight.bold,
        ),
      );
      valueText.textRenderer = valueTextStyle;
      valueText.text = '$displayedValue/$maxValue';

      if (_counterElapsed >= _counterDuration) {
        displayedValue = _counterEndValue;
        _counterDuration = 0;
      }
    }
  }
}

class _FloatingDeltaText extends PositionComponent {
  final String text;
  final Color color;
  final double duration;
  final double floatDistance;
  final Vector2 startPosition;
  late TextComponent _textComponent;
  double _elapsed = 0;

  _FloatingDeltaText({
    required this.text,
    required this.color,
    required this.duration,
    required this.floatDistance,
    required this.startPosition,
  }) : super(
          position: startPosition.clone(),
          size: Vector2.zero(),
          anchor: Anchor.center,
        );

  @override
  Future<void> onLoad() async {
    _textComponent = TextComponent(
      text: text,
      textRenderer: TextPaint(
        style: TextStyle(
          color: color,
          fontSize: 14,
          fontWeight: FontWeight.bold,
        ),
      ),
      anchor: Anchor.center,
    );
    add(_textComponent);
  }

  @override
  void update(double dt) {
    super.update(dt);
    _elapsed += dt;
    final progress = (_elapsed / duration).clamp(0.0, 1.0);
    position = Vector2(startPosition.x, startPosition.y - floatDistance * progress);

    final opacity = 1.0 - progress;
    _textComponent.textRenderer = TextPaint(
      style: TextStyle(
        color: color.withOpacity(opacity),
        fontSize: 14,
        fontWeight: FontWeight.bold,
      ),
    );

    if (_elapsed >= duration) {
      removeFromParent();
    }
  }
}

/// Simple particle component for burst effects
class _Particle extends PositionComponent {
  final Vector2 initialVelocity;
  final Color color;
  final double lifetime;
  double elapsedTime = 0;
  late Vector2 currentVelocity;

  _Particle({
    required Vector2 position,
    required this.initialVelocity,
    required this.color,
    required this.lifetime,
  }) : super(
          position: position,
          size: Vector2(4, 4),
        ) {
    currentVelocity = initialVelocity.clone();
  }

  @override
  void update(double dt) {
    super.update(dt);
    elapsedTime += dt;

    // Move particle
    position += currentVelocity * dt;

    // Apply gravity/damping
    currentVelocity *= 0.95;

    // Fade out
    if (elapsedTime >= lifetime) {
      removeFromParent();
    }
  }

  @override
  void render(Canvas canvas) {
    final opacity = (1.0 - (elapsedTime / lifetime)).clamp(0.0, 1.0);
    canvas.drawCircle(
      Offset(size.x / 2, size.y / 2),
      2,
      Paint()..color = color.withOpacity(opacity),
    );
  }
}

/// Helper component for animating bar width
class _BarAnimator extends Component {
  final Function(double) onUpdate;
  final double duration;
  double elapsed = 0;

  _BarAnimator(this.onUpdate, this.duration);

  @override
  void update(double dt) {
    super.update(dt);
    elapsed += dt;
    onUpdate(dt);
    if (elapsed >= duration) {
      removeFromParent();
    }
  }
}
