import 'dart:math' as math;
import 'package:flutter/material.dart';
import '../theme/syn_theme.dart';

/// Instrumentation Layer Painters
///
/// These painters create the "simulation is running" visual layer:
/// - Scanlines: Slow horizontal sweep every 10-15s
/// - Grid overlay: Faint axis hints that appear on hover
/// - Bracket frames: Corner brackets with overshoot lines
///
/// Following UI Grammar rules:
/// - Lines never end flush—always overshoot or break
/// - All dividers: 1px solid OR 2px dashed (never mix arbitrarily)
/// - One accent color only (SYN cyan)

// =============================================================================
// SCANLINE PAINTER
// =============================================================================

/// Draws a slow-moving horizontal scanline that sweeps across the screen.
///
/// The scanline creates a subtle "CRT monitor" effect that reminds the player
/// they are interfacing with a simulation.
class ScanlinePainter extends CustomPainter {
  /// Progress of the scan (0.0 to 1.0), loops every ~10-15s
  final double progress;

  /// Primary color for the scanline
  final Color color;

  /// Opacity multiplier (recommend 0.02-0.05 for subtlety)
  final double opacity;

  /// Width of the scanline glow
  final double lineWidth;

  ScanlinePainter({
    required this.progress,
    this.color = SynTheme.accent,
    this.opacity = 0.03,
    this.lineWidth = 2.0,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final y = size.height * progress;

    // Main scanline
    final linePaint = Paint()
      ..color = color.withOpacity(opacity)
      ..strokeWidth = lineWidth
      ..style = PaintingStyle.stroke;

    canvas.drawLine(
      Offset(0, y),
      Offset(size.width, y),
      linePaint,
    );

    // Glow above and below (softer)
    final glowPaint = Paint()
      ..color = color.withOpacity(opacity * 0.3)
      ..strokeWidth = lineWidth * 4
      ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 8);

    canvas.drawLine(
      Offset(0, y),
      Offset(size.width, y),
      glowPaint,
    );

    // Faint horizontal lines pattern (CRT effect)
    final patternPaint = Paint()
      ..color = color.withOpacity(opacity * 0.1)
      ..strokeWidth = 1;

    const lineSpacing = 4.0;
    for (double py = 0; py < size.height; py += lineSpacing) {
      canvas.drawLine(
        Offset(0, py),
        Offset(size.width, py),
        patternPaint,
      );
    }
  }

  @override
  bool shouldRepaint(ScanlinePainter oldDelegate) =>
      oldDelegate.progress != progress || oldDelegate.opacity != opacity;
}

// =============================================================================
// GRID OVERLAY PAINTER
// =============================================================================

/// Draws a faint grid/axis overlay that can intensify on hover.
///
/// The grid communicates "you are looking at structured data space."
class GridOverlayPainter extends CustomPainter {
  /// Base opacity (0.0 to 1.0), recommend 0.02-0.05
  final double opacity;

  /// Hover intensity multiplier (1.0 = normal, 2.0 = double on hover)
  final double hoverIntensity;

  /// Grid cell size in logical pixels
  final double cellSize;

  /// Primary color
  final Color color;

  /// Whether to show major axis lines (center cross)
  final bool showAxes;

  /// Whether to show corner coordinates
  final bool showCoordinates;

  GridOverlayPainter({
    this.opacity = 0.03,
    this.hoverIntensity = 1.0,
    this.cellSize = 50.0,
    this.color = SynTheme.accent,
    this.showAxes = true,
    this.showCoordinates = false,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final effectiveOpacity = opacity * hoverIntensity;

    // Minor grid lines (dotted pattern via dash)
    final minorPaint = Paint()
      ..color = color.withOpacity(effectiveOpacity * 0.3)
      ..strokeWidth = 1
      ..style = PaintingStyle.stroke;

    // Draw vertical lines
    for (double x = 0; x < size.width; x += cellSize) {
      _drawDashedLine(
        canvas,
        Offset(x, 0),
        Offset(x, size.height),
        minorPaint,
        dashLength: 2,
        gapLength: 6,
      );
    }

    // Draw horizontal lines
    for (double y = 0; y < size.height; y += cellSize) {
      _drawDashedLine(
        canvas,
        Offset(0, y),
        Offset(size.width, y),
        minorPaint,
        dashLength: 2,
        gapLength: 6,
      );
    }

    // Major axis lines (center)
    if (showAxes) {
      final axisPaint = Paint()
        ..color = color.withOpacity(effectiveOpacity)
        ..strokeWidth = 1
        ..style = PaintingStyle.stroke;

      final centerX = size.width / 2;
      final centerY = size.height / 2;

      // Vertical axis with overshoot
      canvas.drawLine(
        Offset(centerX, -10), // Overshoot top
        Offset(centerX, size.height + 10), // Overshoot bottom
        axisPaint,
      );

      // Horizontal axis with overshoot
      canvas.drawLine(
        Offset(-10, centerY), // Overshoot left
        Offset(size.width + 10, centerY), // Overshoot right
        axisPaint,
      );

      // Axis tick marks
      final tickPaint = Paint()
        ..color = color.withOpacity(effectiveOpacity * 0.8)
        ..strokeWidth = 1;

      const tickSize = 4.0;

      // Ticks along horizontal axis
      for (double x = 0; x < size.width; x += cellSize) {
        canvas.drawLine(
          Offset(x, centerY - tickSize),
          Offset(x, centerY + tickSize),
          tickPaint,
        );
      }

      // Ticks along vertical axis
      for (double y = 0; y < size.height; y += cellSize) {
        canvas.drawLine(
          Offset(centerX - tickSize, y),
          Offset(centerX + tickSize, y),
          tickPaint,
        );
      }
    }
  }

  void _drawDashedLine(
    Canvas canvas,
    Offset start,
    Offset end,
    Paint paint, {
    double dashLength = 4,
    double gapLength = 4,
  }) {
    final dx = end.dx - start.dx;
    final dy = end.dy - start.dy;
    final distance = math.sqrt(dx * dx + dy * dy);
    final unitX = dx / distance;
    final unitY = dy / distance;

    var currentDistance = 0.0;
    var drawing = true;

    while (currentDistance < distance) {
      final segmentLength = drawing ? dashLength : gapLength;
      final nextDistance = math.min(currentDistance + segmentLength, distance);

      if (drawing) {
        canvas.drawLine(
          Offset(
            start.dx + unitX * currentDistance,
            start.dy + unitY * currentDistance,
          ),
          Offset(
            start.dx + unitX * nextDistance,
            start.dy + unitY * nextDistance,
          ),
          paint,
        );
      }

      currentDistance = nextDistance;
      drawing = !drawing;
    }
  }

  @override
  bool shouldRepaint(GridOverlayPainter oldDelegate) =>
      oldDelegate.opacity != opacity ||
      oldDelegate.hoverIntensity != hoverIntensity ||
      oldDelegate.cellSize != cellSize;
}

// =============================================================================
// BRACKET FRAME PAINTER
// =============================================================================

/// Draws corner brackets around a region.
///
/// Brackets are the focus element in our UI grammar (not glows).
/// They communicate "this region is active/selected."
class BracketFramePainter extends CustomPainter {
  /// Size of the bracket arms
  final double bracketSize;

  /// How much the lines overshoot past the corner
  final double overshoot;

  /// Line thickness
  final double strokeWidth;

  /// Primary color
  final Color color;

  /// Opacity
  final double opacity;

  /// Inset from edges
  final double inset;

  /// Which corners to draw (bitfield: TL=1, TR=2, BR=4, BL=8)
  final int corners;

  /// Whether brackets are animating (adds subtle pulse offset)
  final double pulsePhase;

  /// All corners
  static const int allCorners = 15;
  static const int topLeft = 1;
  static const int topRight = 2;
  static const int bottomRight = 4;
  static const int bottomLeft = 8;

  BracketFramePainter({
    this.bracketSize = 20.0,
    this.overshoot = 4.0,
    this.strokeWidth = 2.0,
    this.color = SynTheme.accent,
    this.opacity = 0.8,
    this.inset = 0.0,
    this.corners = allCorners,
    this.pulsePhase = 0.0,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final effectiveOpacity =
        opacity * (0.8 + 0.2 * math.sin(pulsePhase * 2 * math.pi));

    final paint = Paint()
      ..color = color.withOpacity(effectiveOpacity)
      ..strokeWidth = strokeWidth
      ..style = PaintingStyle.stroke
      ..strokeCap = StrokeCap.square;

    final left = inset;
    final top = inset;
    final right = size.width - inset;
    final bottom = size.height - inset;

    // Top-left bracket
    if (corners & topLeft != 0) {
      final path = Path()
        ..moveTo(left - overshoot, top + bracketSize)
        ..lineTo(left, top + bracketSize)
        ..lineTo(left, top)
        ..lineTo(left + bracketSize, top)
        ..lineTo(left + bracketSize + overshoot, top);
      canvas.drawPath(path, paint);
    }

    // Top-right bracket
    if (corners & topRight != 0) {
      final path = Path()
        ..moveTo(right + overshoot, top + bracketSize)
        ..lineTo(right, top + bracketSize)
        ..lineTo(right, top)
        ..lineTo(right - bracketSize, top)
        ..lineTo(right - bracketSize - overshoot, top);
      canvas.drawPath(path, paint);
    }

    // Bottom-right bracket
    if (corners & bottomRight != 0) {
      final path = Path()
        ..moveTo(right + overshoot, bottom - bracketSize)
        ..lineTo(right, bottom - bracketSize)
        ..lineTo(right, bottom)
        ..lineTo(right - bracketSize, bottom)
        ..lineTo(right - bracketSize - overshoot, bottom);
      canvas.drawPath(path, paint);
    }

    // Bottom-left bracket
    if (corners & bottomLeft != 0) {
      final path = Path()
        ..moveTo(left - overshoot, bottom - bracketSize)
        ..lineTo(left, bottom - bracketSize)
        ..lineTo(left, bottom)
        ..lineTo(left + bracketSize, bottom)
        ..lineTo(left + bracketSize + overshoot, bottom);
      canvas.drawPath(path, paint);
    }
  }

  @override
  bool shouldRepaint(BracketFramePainter oldDelegate) =>
      oldDelegate.bracketSize != bracketSize ||
      oldDelegate.overshoot != overshoot ||
      oldDelegate.opacity != opacity ||
      oldDelegate.pulsePhase != pulsePhase ||
      oldDelegate.corners != corners;
}

// =============================================================================
// HASH MARK PAINTER
// =============================================================================

/// Draws small numeric tick marks along panel edges.
///
/// These communicate "this is a measured, instrumented interface."
class HashMarkPainter extends CustomPainter {
  /// Which edge to draw on
  final Axis axis;

  /// Spacing between tick marks
  final double spacing;

  /// Length of tick marks
  final double tickLength;

  /// Color
  final Color color;

  /// Opacity
  final double opacity;

  /// Whether to show every 5th tick as longer
  final bool showMajorTicks;

  HashMarkPainter({
    this.axis = Axis.horizontal,
    this.spacing = 20.0,
    this.tickLength = 4.0,
    this.color = SynTheme.accent,
    this.opacity = 0.2,
    this.showMajorTicks = true,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = color.withOpacity(opacity)
      ..strokeWidth = 1
      ..style = PaintingStyle.stroke;

    final majorPaint = Paint()
      ..color = color.withOpacity(opacity * 1.5)
      ..strokeWidth = 1
      ..style = PaintingStyle.stroke;

    int tickIndex = 0;

    if (axis == Axis.horizontal) {
      for (double x = 0; x < size.width; x += spacing) {
        final isMajor = showMajorTicks && tickIndex % 5 == 0;
        final length = isMajor ? tickLength * 1.5 : tickLength;

        canvas.drawLine(
          Offset(x, 0),
          Offset(x, length),
          isMajor ? majorPaint : paint,
        );

        tickIndex++;
      }
    } else {
      for (double y = 0; y < size.height; y += spacing) {
        final isMajor = showMajorTicks && tickIndex % 5 == 0;
        final length = isMajor ? tickLength * 1.5 : tickLength;

        canvas.drawLine(
          Offset(0, y),
          Offset(length, y),
          isMajor ? majorPaint : paint,
        );

        tickIndex++;
      }
    }
  }

  @override
  bool shouldRepaint(HashMarkPainter oldDelegate) =>
      oldDelegate.spacing != spacing ||
      oldDelegate.tickLength != tickLength ||
      oldDelegate.opacity != opacity;
}

// =============================================================================
// TRACKING LINE PAINTER
// =============================================================================

/// Draws thin tracking lines that slide when panels move.
///
/// These create visual continuity during transitions.
class TrackingLinePainter extends CustomPainter {
  /// Position along the axis (0.0 to 1.0)
  final double position;

  /// Which axis the line runs along
  final Axis axis;

  /// Color
  final Color color;

  /// Opacity
  final double opacity;

  /// Whether the line is currently active (sliding)
  final bool isActive;

  TrackingLinePainter({
    required this.position,
    this.axis = Axis.horizontal,
    this.color = SynTheme.accent,
    this.opacity = 0.15,
    this.isActive = false,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final effectiveOpacity = isActive ? opacity * 2 : opacity;

    final paint = Paint()
      ..color = color.withOpacity(effectiveOpacity)
      ..strokeWidth = 1
      ..style = PaintingStyle.stroke;

    if (axis == Axis.horizontal) {
      final y = size.height * position;
      canvas.drawLine(
        Offset(0, y),
        Offset(size.width, y),
        paint,
      );
    } else {
      final x = size.width * position;
      canvas.drawLine(
        Offset(x, 0),
        Offset(x, size.height),
        paint,
      );
    }

    // Add terminus marks (lines never end flush)
    if (isActive) {
      final terminusPaint = Paint()
        ..color = color.withOpacity(effectiveOpacity * 1.5)
        ..strokeWidth = 2;

      if (axis == Axis.horizontal) {
        final y = size.height * position;
        // Left terminus
        canvas.drawLine(Offset(-4, y - 3), Offset(-4, y + 3), terminusPaint);
        // Right terminus
        canvas.drawLine(
          Offset(size.width + 4, y - 3),
          Offset(size.width + 4, y + 3),
          terminusPaint,
        );
      } else {
        final x = size.width * position;
        // Top terminus
        canvas.drawLine(Offset(x - 3, -4), Offset(x + 3, -4), terminusPaint);
        // Bottom terminus
        canvas.drawLine(
          Offset(x - 3, size.height + 4),
          Offset(x + 3, size.height + 4),
          terminusPaint,
        );
      }
    }
  }

  @override
  bool shouldRepaint(TrackingLinePainter oldDelegate) =>
      oldDelegate.position != position ||
      oldDelegate.isActive != isActive ||
      oldDelegate.opacity != opacity;
}
