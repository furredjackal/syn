import 'dart:math' as math;
import 'package:flutter/material.dart';
import '../theme/syn_theme.dart';
import '../painters/instrumentation_painters.dart';

/// Instrumentation HUD Overlay
///
/// A persistent, non-interactive layer that communicates:
/// "This world is running. You are interfacing with it."
///
/// Features:
/// - Micro-labels (SIM, MEM, REL, LOD, STATE)
/// - Numeric ticks on edges
/// - Scanline sweep
/// - Corner brackets on active regions
///
/// This layer is:
/// - Always present (subtle)
/// - Largely non-interactive
/// - Cheap to render
/// - Extremely effective psychologically
class InstrumentationHUD extends StatefulWidget {
  final Widget child;

  /// Overall opacity of the instrumentation layer
  final double opacity;

  /// Whether to show the scanline effect
  final bool showScanline;

  /// Whether to show corner micro-labels
  final bool showMicroLabels;

  /// Whether to show edge tick marks
  final bool showEdgeTicks;

  /// Whether to show the faint grid
  final bool showGrid;

  /// Current simulation values (for micro-label display)
  final Map<String, String>? simulationValues;

  const InstrumentationHUD({
    super.key,
    required this.child,
    this.opacity = 1.0,
    this.showScanline = true,
    this.showMicroLabels = true,
    this.showEdgeTicks = true,
    this.showGrid = false,
    this.simulationValues,
  });

  @override
  State<InstrumentationHUD> createState() => _InstrumentationHUDState();
}

class _InstrumentationHUDState extends State<InstrumentationHUD>
    with TickerProviderStateMixin {
  late AnimationController _scanlineController;
  late AnimationController _pulseController;
  bool _isHovering = false;

  @override
  void initState() {
    super.initState();

    // Scanline sweeps every 12 seconds
    _scanlineController = AnimationController(
      duration: SynInstrumentation.scanlineDuration,
      vsync: this,
    )..repeat();

    // Subtle pulse for micro-labels
    _pulseController = AnimationController(
      duration: const Duration(seconds: 4),
      vsync: this,
    )..repeat();
  }

  @override
  void dispose() {
    _scanlineController.dispose();
    _pulseController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _isHovering = true),
      onExit: (_) => setState(() => _isHovering = false),
      child: Stack(
        children: [
          // Main content
          widget.child,

          // Instrumentation layer (non-interactive)
          Positioned.fill(
            child: IgnorePointer(
              child: Opacity(
                opacity: widget.opacity,
                child: Stack(
                  children: [
                    // Scanline effect
                    if (widget.showScanline)
                      AnimatedBuilder(
                        animation: _scanlineController,
                        builder: (context, _) => CustomPaint(
                          painter: ScanlinePainter(
                            progress: _scanlineController.value,
                            opacity: SynInstrumentation.scanlineOpacity,
                          ),
                          size: Size.infinite,
                        ),
                      ),

                    // Grid overlay (intensifies on hover)
                    if (widget.showGrid)
                      AnimatedBuilder(
                        animation: _pulseController,
                        builder: (context, _) => CustomPaint(
                          painter: GridOverlayPainter(
                            opacity: SynInstrumentation.gridOpacity,
                            hoverIntensity: _isHovering
                                ? SynInstrumentation.gridHoverMultiplier
                                : 1.0,
                            cellSize: SynInstrumentation.gridCellSize,
                          ),
                          size: Size.infinite,
                        ),
                      ),

                    // Edge tick marks
                    if (widget.showEdgeTicks) ...[
                      // Top edge
                      Positioned(
                        top: 0,
                        left: 0,
                        right: 0,
                        height: 8,
                        child: CustomPaint(
                          painter: HashMarkPainter(
                            axis: Axis.horizontal,
                            spacing: 30,
                            tickLength: 6,
                            opacity: 0.15,
                          ),
                        ),
                      ),
                      // Bottom edge
                      Positioned(
                        bottom: 0,
                        left: 0,
                        right: 0,
                        height: 8,
                        child: Transform.flip(
                          flipY: true,
                          child: CustomPaint(
                            painter: HashMarkPainter(
                              axis: Axis.horizontal,
                              spacing: 30,
                              tickLength: 6,
                              opacity: 0.15,
                            ),
                          ),
                        ),
                      ),
                      // Left edge
                      Positioned(
                        top: 0,
                        bottom: 0,
                        left: 0,
                        width: 8,
                        child: CustomPaint(
                          painter: HashMarkPainter(
                            axis: Axis.vertical,
                            spacing: 30,
                            tickLength: 6,
                            opacity: 0.15,
                          ),
                        ),
                      ),
                      // Right edge
                      Positioned(
                        top: 0,
                        bottom: 0,
                        right: 0,
                        width: 8,
                        child: Transform.flip(
                          flipX: true,
                          child: CustomPaint(
                            painter: HashMarkPainter(
                              axis: Axis.vertical,
                              spacing: 30,
                              tickLength: 6,
                              opacity: 0.15,
                            ),
                          ),
                        ),
                      ),
                    ],

                    // Corner micro-labels
                    if (widget.showMicroLabels) ...[
                      // Top-left: SIM
                      Positioned(
                        top: 12,
                        left: 16,
                        child: _MicroLabel(
                          label: 'SIM',
                          value: widget.simulationValues?['SIM'],
                          pulseController: _pulseController,
                        ),
                      ),
                      // Top-right: STATE
                      Positioned(
                        top: 12,
                        right: 16,
                        child: _MicroLabel(
                          label: 'STATE',
                          value: widget.simulationValues?['STATE'],
                          pulseController: _pulseController,
                          alignment: CrossAxisAlignment.end,
                        ),
                      ),
                      // Bottom-left: MEM
                      Positioned(
                        bottom: 12,
                        left: 16,
                        child: _MicroLabel(
                          label: 'MEM',
                          value: widget.simulationValues?['MEM'],
                          pulseController: _pulseController,
                        ),
                      ),
                      // Bottom-right: LOD
                      Positioned(
                        bottom: 12,
                        right: 16,
                        child: _MicroLabel(
                          label: 'LOD',
                          value: widget.simulationValues?['LOD'],
                          pulseController: _pulseController,
                          alignment: CrossAxisAlignment.end,
                        ),
                      ),
                    ],

                    // Screen corner brackets
                    AnimatedBuilder(
                      animation: _pulseController,
                      builder: (context, _) => CustomPaint(
                        painter: BracketFramePainter(
                          bracketSize: 30,
                          overshoot: 6,
                          strokeWidth: 1,
                          opacity: 0.2,
                          inset: 8,
                          pulsePhase: _pulseController.value,
                        ),
                        size: Size.infinite,
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// Small micro-label for instrumentation display
class _MicroLabel extends StatelessWidget {
  final String label;
  final String? value;
  final AnimationController pulseController;
  final CrossAxisAlignment alignment;

  const _MicroLabel({
    required this.label,
    required this.pulseController,
    this.value,
    this.alignment = CrossAxisAlignment.start,
  });

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: pulseController,
      builder: (context, _) {
        final pulseOpacity =
            0.3 + (0.1 * math.sin(pulseController.value * 2 * math.pi));

        return Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: alignment,
          children: [
            Text(
              label,
              style: TextStyle(
                fontFamily: 'monospace',
                fontSize: 8,
                fontWeight: FontWeight.w600,
                letterSpacing: 2,
                color: SynTheme.accent.withOpacity(pulseOpacity),
              ),
            ),
            if (value != null)
              Text(
                value!,
                style: TextStyle(
                  fontFamily: 'monospace',
                  fontSize: 10,
                  fontWeight: FontWeight.w400,
                  color: SynTheme.textMuted.withOpacity(pulseOpacity),
                ),
              ),
          ],
        );
      },
    );
  }
}

/// Focused bracket overlay for highlighting specific regions
///
/// Use this to draw attention to panels or interactive elements.
/// "Animated focus brackets that linger after interaction"
class FocusBrackets extends StatefulWidget {
  final Widget child;
  final bool isActive;
  final Duration lingerDuration;
  final double bracketSize;
  final double inset;

  const FocusBrackets({
    super.key,
    required this.child,
    this.isActive = false,
    this.lingerDuration = const Duration(milliseconds: 800),
    this.bracketSize = 16,
    this.inset = 0,
  });

  @override
  State<FocusBrackets> createState() => _FocusBracketsState();
}

class _FocusBracketsState extends State<FocusBrackets>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  bool _isVisible = false;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      duration: widget.lingerDuration,
      vsync: this,
    );
    _isVisible = widget.isActive;
    if (_isVisible) {
      _controller.repeat();
    }
  }

  @override
  void didUpdateWidget(FocusBrackets oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.isActive && !oldWidget.isActive) {
      setState(() => _isVisible = true);
      _controller.repeat();
    } else if (!widget.isActive && oldWidget.isActive) {
      // Linger before hiding
      Future.delayed(widget.lingerDuration, () {
        if (mounted && !widget.isActive) {
          setState(() => _isVisible = false);
          _controller.stop();
        }
      });
    }
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: [
        widget.child,
        if (_isVisible)
          Positioned.fill(
            child: IgnorePointer(
              child: AnimatedBuilder(
                animation: _controller,
                builder: (context, _) => AnimatedOpacity(
                  opacity: widget.isActive ? 1.0 : 0.0,
                  duration: const Duration(milliseconds: 200),
                  child: CustomPaint(
                    painter: BracketFramePainter(
                      bracketSize: widget.bracketSize,
                      overshoot: 4,
                      strokeWidth: 2,
                      opacity: 0.8,
                      inset: widget.inset,
                      pulsePhase: _controller.value,
                    ),
                  ),
                ),
              ),
            ),
          ),
      ],
    );
  }
}

/// Tracking line that follows panel movements
///
/// "Thin tracking lines that slide when panels move"
class TrackingLines extends StatelessWidget {
  final Widget child;
  final List<double> horizontalPositions;
  final List<double> verticalPositions;
  final bool isActive;

  const TrackingLines({
    super.key,
    required this.child,
    this.horizontalPositions = const [],
    this.verticalPositions = const [],
    this.isActive = false,
  });

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: [
        child,
        Positioned.fill(
          child: IgnorePointer(
            child: CustomPaint(
              painter: _MultiTrackingLinePainter(
                horizontalPositions: horizontalPositions,
                verticalPositions: verticalPositions,
                isActive: isActive,
              ),
            ),
          ),
        ),
      ],
    );
  }
}

class _MultiTrackingLinePainter extends CustomPainter {
  final List<double> horizontalPositions;
  final List<double> verticalPositions;
  final bool isActive;

  _MultiTrackingLinePainter({
    required this.horizontalPositions,
    required this.verticalPositions,
    required this.isActive,
  });

  @override
  void paint(Canvas canvas, Size size) {
    for (final pos in horizontalPositions) {
      TrackingLinePainter(
        position: pos,
        axis: Axis.horizontal,
        isActive: isActive,
      ).paint(canvas, size);
    }

    for (final pos in verticalPositions) {
      TrackingLinePainter(
        position: pos,
        axis: Axis.vertical,
        isActive: isActive,
      ).paint(canvas, size);
    }
  }

  @override
  bool shouldRepaint(_MultiTrackingLinePainter oldDelegate) =>
      oldDelegate.horizontalPositions != horizontalPositions ||
      oldDelegate.verticalPositions != verticalPositions ||
      oldDelegate.isActive != isActive;
}
