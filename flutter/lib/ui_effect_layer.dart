import 'package:flame/components.dart';
import 'package:flutter/material.dart';
import 'syn_game.dart';

/// Global UI effect layer that applies mood-based visual filters to the entire game.
/// Renders vignette, saturation, brightness, and blur adjustments based on current mood.
/// Mood range: -10 (despair) to +10 (euphoria)
class UIEffectLayer extends PositionComponent with HasGameRef<SynGame> {
  // Effect parameters interpolated from mood
  double vignetteOpacity = 0.2;
  double saturation = 1.0;
  double brightness = 1.0;
  double blurRadius = 0.0;
  double animationSpeed = 1.0;

  // Mood-to-effect mapping constants
  static const Map<String, Map<String, dynamic>> moodTiers = {
    'despair': {
      'moodRange': [-10, -6],
      'vignetteOpacity': 0.8,
      'saturation': 0.6,
      'brightness': 0.75,
      'blurRadius': 2.0,
      'animationSpeed': 0.8,
      'grainOpacity': 0.15,
    },
    'troubled': {
      'moodRange': [-5, -1],
      'vignetteOpacity': 0.5,
      'saturation': 0.85,
      'brightness': 0.9,
      'blurRadius': 1.0,
      'animationSpeed': 0.9,
      'grainOpacity': 0.05,
    },
    'neutral': {
      'moodRange': [-0.5, 0.5],
      'vignetteOpacity': 0.2,
      'saturation': 1.0,
      'brightness': 1.0,
      'blurRadius': 0.0,
      'animationSpeed': 1.0,
      'grainOpacity': 0.0,
    },
    'content': {
      'moodRange': [1, 5],
      'vignetteOpacity': 0.1,
      'saturation': 1.1,
      'brightness': 1.05,
      'blurRadius': 0.0,
      'animationSpeed': 1.05,
      'grainOpacity': 0.0,
    },
    'euphoric': {
      'moodRange': [6, 10],
      'vignetteOpacity': 0.05,
      'saturation': 1.3,
      'brightness': 1.15,
      'blurRadius': 0.0,
      'animationSpeed': 1.2,
      'bloomIntensity': 0.3,
    },
  };

  @override
  Future<void> onLoad() async {
    // Layer covers entire screen, renders on top of game
    size = game.size;
    position = Vector2.zero();
  }

  @override
  void update(double dt) {
    super.update(dt);

    // Update effect parameters based on current mood
    _updateEffectsFromMood();
  }

  @override
  void render(Canvas canvas) {
    // Apply effects in order: vignette, scanlines (optional), grain (optional)
    _renderVignette(canvas);
    _renderSaturationBrightnessOverlay(canvas);
    // Blur is harder to achieve in Canvas; could be deferred to shader or skipped for perf
  }

  /// Updates effect parameters based on mood tier.
  void _updateEffectsFromMood() {
    final mood = game.gameState.mood.toDouble();

    // Determine tier and interpolate between tiers
    final targetParams = _getMoodEffectParams(mood);

    // Smooth interpolation to target values
    const transitionSpeed = 0.1; // Adjust for faster/slower transitions
    vignetteOpacity = _lerpValue(vignetteOpacity,
        targetParams['vignetteOpacity'] as double, transitionSpeed);
    saturation = _lerpValue(
        saturation, targetParams['saturation'] as double, transitionSpeed);
    brightness = _lerpValue(
        brightness, targetParams['brightness'] as double, transitionSpeed);
    blurRadius = _lerpValue(
        blurRadius, targetParams['blurRadius'] as double, transitionSpeed);
    animationSpeed = _lerpValue(animationSpeed,
        targetParams['animationSpeed'] as double, transitionSpeed);
  }

  /// Returns effect parameters for given mood value.
  Map<String, dynamic> _getMoodEffectParams(double mood) {
    if (mood <= -6) {
      return moodTiers['despair']!;
    } else if (mood <= -1) {
      return _interpolateMoodParams('troubled', 'despair', mood, -5, -1);
    } else if (mood <= 0.5) {
      return moodTiers['neutral']!;
    } else if (mood <= 5) {
      return _interpolateMoodParams('content', 'neutral', mood, 1, 5);
    } else {
      return moodTiers['euphoric']!;
    }
  }

  /// Linearly interpolates between two mood tier effect params.
  Map<String, dynamic> _interpolateMoodParams(
    String tierA,
    String tierB,
    double mood,
    double rangeMin,
    double rangeMax,
  ) {
    final paramsA = moodTiers[tierA]!;
    final paramsB = moodTiers[tierB]!;

    // Normalize mood to 0..1 within range
    final t = ((mood - rangeMin) / (rangeMax - rangeMin)).clamp(0.0, 1.0);

    return {
      'vignetteOpacity': (paramsB['vignetteOpacity'] as double) +
          ((paramsA['vignetteOpacity'] as double) -
                  (paramsB['vignetteOpacity'] as double)) *
              t,
      'saturation': (paramsB['saturation'] as double) +
          ((paramsA['saturation'] as double) -
                  (paramsB['saturation'] as double)) *
              t,
      'brightness': (paramsB['brightness'] as double) +
          ((paramsA['brightness'] as double) -
                  (paramsB['brightness'] as double)) *
              t,
      'blurRadius': (paramsB['blurRadius'] as double) +
          ((paramsA['blurRadius'] as double) -
                  (paramsB['blurRadius'] as double)) *
              t,
      'animationSpeed': (paramsB['animationSpeed'] as double) +
          ((paramsA['animationSpeed'] as double) -
                  (paramsB['animationSpeed'] as double)) *
              t,
    };
  }

  /// Linearly interpolates a single value.
  double _lerpValue(double from, double to, double t) {
    return from + (to - from) * t;
  }

  /// Renders a radial vignette overlay.
  /// Center is transparent, edges fade to black based on vignetteOpacity.
  void _renderVignette(Canvas canvas) {
    final center = Offset(size.x / 2, size.y / 2);
    final radius = size.x.abs() > size.y.abs() ? size.x / 1.5 : size.y / 1.5;

    final gradient = RadialGradient(
      center: Alignment.center,
      radius: 1.0,
      colors: [
        Colors.transparent,
        Colors.black.withOpacity(vignetteOpacity * 0.5),
        Colors.black.withOpacity(vignetteOpacity),
      ],
      stops: [0.3, 0.7, 1.0],
    );

    canvas.drawCircle(
      center,
      radius,
      Paint()
        ..shader = gradient
            .createShader(Rect.fromCircle(center: center, radius: radius)),
    );
  }

  /// Applies saturation and brightness adjustments via color overlay.
  /// Uses a semi-transparent overlay with blend modes.
  void _renderSaturationBrightnessOverlay(Canvas canvas) {
    // Desaturation: apply gray overlay with reduced opacity
    if (saturation < 1.0) {
      final desatAmount = 1.0 - saturation;
      canvas.drawRect(
        Rect.fromLTWH(0, 0, size.x, size.y),
        Paint()
          ..color = Colors.grey.withOpacity(desatAmount * 0.3)
          ..blendMode = BlendMode.multiply,
      );
    }

    // Darkening: apply black overlay
    if (brightness < 1.0) {
      final darkenAmount = 1.0 - brightness;
      canvas.drawRect(
        Rect.fromLTWH(0, 0, size.x, size.y),
        Paint()
          ..color = Colors.black.withOpacity(darkenAmount * 0.5)
          ..blendMode = BlendMode.darken,
      );
    }

    // Brightening: apply white overlay with screen blend mode
    if (brightness > 1.0) {
      final brightenAmount = brightness - 1.0;
      canvas.drawRect(
        Rect.fromLTWH(0, 0, size.x, size.y),
        Paint()
          ..color = Colors.white.withOpacity(brightenAmount * 0.3)
          ..blendMode = BlendMode.screen,
      );
    }
  }

  /// Optional: Render scanlines for troubled/despair moods.
  /// Skipped for now due to performance concerns.
  void _renderScanlines(Canvas canvas) {
    const lineHeight = 2.0;
    const lineSpacing = 4.0;

    final scanlinePaint = Paint()
      ..color = Colors.black.withOpacity(0.05)
      ..strokeWidth = lineHeight;

    for (double y = 0; y < size.y; y += lineSpacing) {
      canvas.drawLine(Offset(0, y), Offset(size.x, y), scanlinePaint);
    }
  }

  /// Gets current animation speed multiplier (for use by child components).
  /// Components can call this to adjust their animation durations.
  double getAnimationSpeedMultiplier() => animationSpeed;
}
