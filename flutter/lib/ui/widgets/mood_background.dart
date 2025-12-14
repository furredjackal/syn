import 'package:flutter/material.dart';
import '../../theme/theme.dart';
import '../painters/instrumentation_painters.dart';
import '../theme/syn_theme.dart' as syn_ui;

/// Mood-reactive background wrapper.
///
/// Applies a subtle color tint overlay based on the current mood value.
/// Creates atmospheric changes as the player's emotional state shifts.
///
/// Now includes scanline instrumentation layer for "simulation running" feel.
class MoodBackground extends StatefulWidget {
  final Widget child;

  /// Current mood value (-10 to +10)
  final double mood;

  /// Whether to show vignette effect
  final bool showVignette;

  /// Intensity of the color overlay (0-1)
  final double intensity;

  /// Whether to show scanline effect (instrumentation layer)
  final bool showScanline;

  /// Whether to show CRT-style horizontal lines
  final bool showCRTLines;

  const MoodBackground({
    super.key,
    required this.child,
    required this.mood,
    this.showVignette = true,
    this.intensity = 0.3,
    this.showScanline = true,
    this.showCRTLines = true,
  });

  @override
  State<MoodBackground> createState() => _MoodBackgroundState();
}

class _MoodBackgroundState extends State<MoodBackground>
    with TickerProviderStateMixin {
  late AnimationController _transitionController;
  late AnimationController _scanlineController;
  Color _currentColor = MoodColors.neutral;
  Color _targetColor = MoodColors.neutral;

  @override
  void initState() {
    super.initState();
    _transitionController = AnimationController(
      duration: const Duration(milliseconds: 1500),
      vsync: this,
    );
    _scanlineController = AnimationController(
      duration: syn_ui.SynInstrumentation.scanlineDuration,
      vsync: this,
    )..repeat();
    _currentColor = MoodColors.forMood(widget.mood);
    _targetColor = _currentColor;
  }

  @override
  void didUpdateWidget(MoodBackground oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.mood != widget.mood) {
      _currentColor = Color.lerp(
        _currentColor,
        _targetColor,
        _transitionController.value,
      )!;
      _targetColor = MoodColors.forMood(widget.mood);
      _transitionController.forward(from: 0);
    }
  }

  @override
  void dispose() {
    _transitionController.dispose();
    _scanlineController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: _transitionController,
      builder: (context, _) {
        final color = Color.lerp(
          _currentColor,
          _targetColor,
          Curves.easeInOut.transform(_transitionController.value),
        )!;

        return Stack(
          children: [
            widget.child,

            // Mood color overlay
            Positioned.fill(
              child: IgnorePointer(
                child: AnimatedContainer(
                  duration: const Duration(milliseconds: 500),
                  decoration: BoxDecoration(
                    gradient: RadialGradient(
                      center: Alignment.center,
                      radius: 1.5,
                      colors: [
                        color.withOpacity(widget.intensity * 0.5),
                        color.withOpacity(widget.intensity),
                      ],
                    ),
                  ),
                ),
              ),
            ),

            // Vignette effect
            if (widget.showVignette)
              Positioned.fill(
                child: IgnorePointer(
                  child: DecoratedBox(
                    decoration: BoxDecoration(
                      gradient: RadialGradient(
                        center: Alignment.center,
                        radius: 1.2,
                        colors: [
                          Colors.transparent,
                          Colors.black.withOpacity(0.6),
                        ],
                        stops: const [0.5, 1.0],
                      ),
                    ),
                  ),
                ),
              ),

            // Mood indicator (subtle edge glow)
            Positioned.fill(
              child: IgnorePointer(
                child: _buildEdgeGlow(color),
              ),
            ),

            // Scanline instrumentation layer
            if (widget.showScanline || widget.showCRTLines)
              Positioned.fill(
                child: IgnorePointer(
                  child: AnimatedBuilder(
                    animation: _scanlineController,
                    builder: (context, _) => CustomPaint(
                      painter: ScanlinePainter(
                        progress: _scanlineController.value,
                        opacity: syn_ui.SynInstrumentation.scanlineOpacity,
                        color: syn_ui.SynTheme.accent,
                      ),
                      size: Size.infinite,
                    ),
                  ),
                ),
              ),
          ],
        );
      },
    );
  }

  Widget _buildEdgeGlow(Color color) {
    // More intense glow for extreme moods
    final isExtreme = widget.mood.abs() >= 6;
    final glowIntensity = isExtreme ? 0.4 : 0.15;

    return Container(
      decoration: BoxDecoration(
        border: Border.all(
          color: color.withOpacity(glowIntensity),
          width: isExtreme ? 3 : 1,
        ),
        boxShadow: isExtreme
            ? [
                BoxShadow(
                  color: color.withOpacity(0.3),
                  blurRadius: 30,
                  spreadRadius: -10,
                ),
              ]
            : null,
      ),
    );
  }
}

/// Karma visual effects overlay.
///
/// Adds subtle visual effects based on karma value.
/// High positive: Ethereal glow
/// High negative: Red glitch effects
class KarmaOverlay extends StatefulWidget {
  final Widget child;

  /// Current karma value (-100 to +100)
  final int karma;

  const KarmaOverlay({
    super.key,
    required this.child,
    required this.karma,
  });

  @override
  State<KarmaOverlay> createState() => _KarmaOverlayState();
}

class _KarmaOverlayState extends State<KarmaOverlay>
    with SingleTickerProviderStateMixin {
  late AnimationController _pulseController;

  @override
  void initState() {
    super.initState();
    _pulseController = AnimationController(
      duration: const Duration(milliseconds: 2000),
      vsync: this,
    )..repeat(reverse: true);
  }

  @override
  void dispose() {
    _pulseController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final isHighPositive = widget.karma > 60;
    final isHighNegative = widget.karma < -60;

    if (!isHighPositive && !isHighNegative) {
      return widget.child;
    }

    return Stack(
      children: [
        widget.child,

        // Karma effect overlay
        if (isHighPositive)
          Positioned.fill(
            child: AnimatedBuilder(
              animation: _pulseController,
              builder: (context, _) {
                return IgnorePointer(
                  child: Container(
                    decoration: BoxDecoration(
                      gradient: RadialGradient(
                        center: Alignment.topCenter,
                        radius: 1.0,
                        colors: [
                          Colors.white.withOpacity(
                            0.05 + (_pulseController.value * 0.03),
                          ),
                          Colors.transparent,
                        ],
                      ),
                    ),
                  ),
                );
              },
            ),
          ),

        if (isHighNegative)
          Positioned.fill(
            child: AnimatedBuilder(
              animation: _pulseController,
              builder: (context, _) {
                return IgnorePointer(
                  child: Container(
                    decoration: BoxDecoration(
                      border: Border.all(
                        color: KarmaColors.highNegative.withOpacity(
                          0.2 + (_pulseController.value * 0.1),
                        ),
                        width: 2,
                      ),
                    ),
                  ),
                );
              },
            ),
          ),
      ],
    );
  }
}

/// Life stage theme provider.
///
/// Adjusts UI accent colors based on character's life stage.
class LifeStageThemeProvider extends InheritedWidget {
  final LifeStageTheme theme;

  const LifeStageThemeProvider({
    super.key,
    required this.theme,
    required super.child,
  });

  static LifeStageTheme of(BuildContext context) {
    final provider =
        context.dependOnInheritedWidgetOfExactType<LifeStageThemeProvider>();
    return provider?.theme ?? LifeStageTheme.adult;
  }

  @override
  bool updateShouldNotify(LifeStageThemeProvider oldWidget) {
    return theme.name != oldWidget.theme.name;
  }
}
