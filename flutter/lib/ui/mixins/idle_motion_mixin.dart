import 'dart:math' as math;
import 'package:flutter/material.dart';
import 'package:flutter/gestures.dart';

/// Idle Motion Mixin
///
/// Provides low-frequency idle motion for desktop UI:
/// - Slow parallax drift (8-12s cycles)
/// - Micro-positional breathing
/// - Cursor-proximity reactions
///
/// "On desktop, nothing moving = nothing happening."
/// This mixin makes UI feel alive even when the player is thinking.

/// Configuration for idle motion behavior
class IdleMotionConfig {
  /// Duration of the primary oscillation cycle
  final Duration cycleDuration;

  /// Maximum offset for the drift effect in logical pixels
  final double maxDrift;

  /// Intensity of the breathing effect (0.0-1.0)
  final double breathingIntensity;

  /// Whether to enable cursor proximity reactions
  final bool enableCursorReaction;

  /// Radius for cursor proximity detection in logical pixels
  final double cursorReactionRadius;

  /// Maximum offset when cursor is near
  final double cursorMaxOffset;

  const IdleMotionConfig({
    this.cycleDuration = const Duration(seconds: 10),
    this.maxDrift = 3.0,
    this.breathingIntensity = 0.02,
    this.enableCursorReaction = true,
    this.cursorReactionRadius = 150.0,
    this.cursorMaxOffset = 4.0,
  });

  /// Subtle motion preset
  static const subtle = IdleMotionConfig(
    cycleDuration: Duration(seconds: 12),
    maxDrift: 2.0,
    breathingIntensity: 0.01,
  );

  /// Standard motion preset
  static const standard = IdleMotionConfig();

  /// Dramatic motion preset (for hero elements)
  static const dramatic = IdleMotionConfig(
    cycleDuration: Duration(seconds: 8),
    maxDrift: 5.0,
    breathingIntensity: 0.03,
    cursorMaxOffset: 8.0,
  );
}

/// Mixin that provides idle motion animation capabilities to StatefulWidgets.
///
/// Usage:
/// ```dart
/// class _MyWidgetState extends State<MyWidget>
///     with SingleTickerProviderStateMixin, IdleMotionMixin {
///   @override
///   IdleMotionConfig get idleMotionConfig => IdleMotionConfig.standard;
///
///   @override
///   Widget build(BuildContext context) {
///     return Transform.translate(
///       offset: idleOffset,
///       child: Opacity(
///         opacity: 1.0 + breathingValue,
///         child: ...
///       ),
///     );
///   }
/// }
/// ```
mixin IdleMotionMixin<T extends StatefulWidget>
    on State<T>, TickerProviderStateMixin<T> {
  late AnimationController _idleController;
  Offset _cursorPosition = Offset.zero;
  bool _isCursorNear = false;

  /// Override to customize idle motion behavior
  IdleMotionConfig get idleMotionConfig => IdleMotionConfig.standard;

  /// Unique phase offset for this widget (set in initState or pass via widget)
  double get phaseOffset => 0.0;

  /// Current idle animation value (0.0 to 1.0)
  double get idleProgress => _idleController.value;

  /// Current offset for position drift effect
  Offset get idleOffset {
    final phase = (idleProgress * 2 * math.pi) + phaseOffset;
    final config = idleMotionConfig;

    // Base drift from oscillation
    var dx = math.sin(phase) * config.maxDrift;
    var dy = math.cos(phase * 0.7) * config.maxDrift * 0.6;

    // Add cursor reaction if enabled and cursor is near
    if (config.enableCursorReaction && _isCursorNear) {
      final cursorOffset = _calculateCursorReaction();
      dx += cursorOffset.dx;
      dy += cursorOffset.dy;
    }

    return Offset(dx, dy);
  }

  /// Current scale for breathing effect (centered around 1.0)
  double get breathingScale {
    final phase = (idleProgress * 2 * math.pi) + phaseOffset;
    final config = idleMotionConfig;
    return 1.0 + (math.sin(phase * 0.5) * config.breathingIntensity);
  }

  /// Current opacity modifier for breathing effect
  double get breathingValue {
    final phase = (idleProgress * 2 * math.pi) + phaseOffset;
    final config = idleMotionConfig;
    return math.sin(phase * 0.5) * config.breathingIntensity;
  }

  /// Current rotation for subtle tilt effect (radians)
  double get idleTilt {
    final phase = (idleProgress * 2 * math.pi) + phaseOffset;
    return math.sin(phase * 0.3) * 0.005; // Very subtle
  }

  @override
  void initState() {
    super.initState();
    _idleController = AnimationController(
      duration: idleMotionConfig.cycleDuration,
      vsync: this,
    )..repeat();
  }

  @override
  void dispose() {
    _idleController.dispose();
    super.dispose();
  }

  /// Call this to update cursor position for proximity reactions.
  /// Typically called from a MouseRegion or Listener at a higher level.
  void updateCursorPosition(Offset globalPosition, RenderBox? renderBox) {
    if (renderBox == null || !idleMotionConfig.enableCursorReaction) return;

    final localPosition = renderBox.globalToLocal(globalPosition);
    final size = renderBox.size;
    final center = Offset(size.width / 2, size.height / 2);

    _cursorPosition = localPosition;
    final distance = (localPosition - center).distance;
    _isCursorNear = distance < idleMotionConfig.cursorReactionRadius;
  }

  Offset _calculateCursorReaction() {
    if (!_isCursorNear) return Offset.zero;

    // Calculate reaction based on cursor position
    // Elements subtly push away from cursor
    final config = idleMotionConfig;
    final dx = _cursorPosition.dx;
    final dy = _cursorPosition.dy;

    // Normalize and invert (push away)
    final distance = math.sqrt(dx * dx + dy * dy);
    if (distance < 1) return Offset.zero;

    final proximity =
        1.0 - (distance / config.cursorReactionRadius).clamp(0.0, 1.0);
    final reactionStrength = proximity * config.cursorMaxOffset;

    return Offset(
      -(dx / distance) * reactionStrength,
      -(dy / distance) * reactionStrength,
    );
  }

  /// Wrap child widget with idle motion transforms.
  /// Convenience method for common use case.
  Widget applyIdleMotion({
    required Widget child,
    bool applyOffset = true,
    bool applyScale = true,
    bool applyTilt = false,
  }) {
    return AnimatedBuilder(
      animation: _idleController,
      builder: (context, _) {
        Widget result = child;

        if (applyScale) {
          result = Transform.scale(
            scale: breathingScale,
            child: result,
          );
        }

        if (applyTilt) {
          result = Transform.rotate(
            angle: idleTilt,
            child: result,
          );
        }

        if (applyOffset) {
          result = Transform.translate(
            offset: idleOffset,
            child: result,
          );
        }

        return result;
      },
    );
  }
}

/// A simpler version that can be used without the mixin,
/// as a standalone widget.
class IdleMotionWidget extends StatefulWidget {
  final Widget child;
  final IdleMotionConfig config;
  final double phaseOffset;
  final bool applyOffset;
  final bool applyScale;
  final bool applyTilt;

  const IdleMotionWidget({
    super.key,
    required this.child,
    this.config = const IdleMotionConfig(),
    this.phaseOffset = 0.0,
    this.applyOffset = true,
    this.applyScale = true,
    this.applyTilt = false,
  });

  @override
  State<IdleMotionWidget> createState() => _IdleMotionWidgetState();
}

class _IdleMotionWidgetState extends State<IdleMotionWidget>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      duration: widget.config.cycleDuration,
      vsync: this,
    )..repeat();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: _controller,
      builder: (context, child) {
        final phase = (_controller.value * 2 * math.pi) + widget.phaseOffset;
        final config = widget.config;

        Widget result = child!;

        if (widget.applyScale) {
          final scale =
              1.0 + (math.sin(phase * 0.5) * config.breathingIntensity);
          result = Transform.scale(scale: scale, child: result);
        }

        if (widget.applyTilt) {
          final tilt = math.sin(phase * 0.3) * 0.005;
          result = Transform.rotate(angle: tilt, child: result);
        }

        if (widget.applyOffset) {
          final dx = math.sin(phase) * config.maxDrift;
          final dy = math.cos(phase * 0.7) * config.maxDrift * 0.6;
          result = Transform.translate(offset: Offset(dx, dy), child: result);
        }

        return result;
      },
      child: widget.child,
    );
  }
}

/// Global cursor tracker for proximity-based effects.
///
/// Wrap your app or screen with this to enable cursor proximity
/// reactions across all IdleMotionMixin widgets.
class CursorProximityTracker extends StatefulWidget {
  final Widget child;

  const CursorProximityTracker({super.key, required this.child});

  static Offset? of(BuildContext context) {
    return context
        .dependOnInheritedWidgetOfExactType<_CursorPositionInherited>()
        ?.position;
  }

  @override
  State<CursorProximityTracker> createState() => _CursorProximityTrackerState();
}

class _CursorProximityTrackerState extends State<CursorProximityTracker> {
  Offset _cursorPosition = Offset.zero;

  @override
  Widget build(BuildContext context) {
    return Listener(
      onPointerHover: (event) {
        setState(() {
          _cursorPosition = event.position;
        });
      },
      onPointerMove: (event) {
        setState(() {
          _cursorPosition = event.position;
        });
      },
      child: _CursorPositionInherited(
        position: _cursorPosition,
        child: widget.child,
      ),
    );
  }
}

class _CursorPositionInherited extends InheritedWidget {
  final Offset position;

  const _CursorPositionInherited({
    required this.position,
    required super.child,
  });

  @override
  bool updateShouldNotify(_CursorPositionInherited oldWidget) =>
      position != oldWidget.position;
}
