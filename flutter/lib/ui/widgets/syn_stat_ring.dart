import 'dart:math' as math;
import 'package:flutter/material.dart';
import '../theme/syn_theme.dart';

/// Circular stat ring with Persona 5 aesthetics.
///
/// Features:
/// - Animated arc fill on value changes
/// - Pulsing glow at critical thresholds
/// - Central icon and value display
/// - Scanline overlay effect
/// - Outer tick marks
class SynStatRing extends StatefulWidget {
  /// Current value (0-100)
  final double value;

  /// Stat label (displayed below value)
  final String label;

  /// Central icon
  final IconData icon;

  /// Ring color (defaults to accent)
  final Color? color;

  /// Size of the ring
  final double size;

  /// Ring thickness
  final double thickness;

  /// Previous value for delta animation
  final double? previousValue;

  /// Low threshold for warning color (default 25)
  final double lowThreshold;

  /// Critical threshold for danger color (default 10)
  final double criticalThreshold;

  const SynStatRing({
    super.key,
    required this.value,
    required this.label,
    required this.icon,
    this.color,
    this.size = 100,
    this.thickness = 8,
    this.previousValue,
    this.lowThreshold = 25,
    this.criticalThreshold = 10,
  });

  @override
  State<SynStatRing> createState() => _SynStatRingState();
}

class _SynStatRingState extends State<SynStatRing>
    with TickerProviderStateMixin {
  late AnimationController _pulseController;
  late AnimationController _fillController;
  late Animation<double> _pulseAnimation;
  late Animation<double> _fillAnimation;
  double _previousValue = 0;

  @override
  void initState() {
    super.initState();
    _previousValue = widget.previousValue ?? widget.value;

    _pulseController = AnimationController(
      duration: const Duration(milliseconds: 1500),
      vsync: this,
    )..repeat(reverse: true);

    _pulseAnimation = Tween<double>(begin: 0.2, end: 0.6).animate(
      CurvedAnimation(parent: _pulseController, curve: Curves.easeInOut),
    );

    _fillController = AnimationController(
      duration: SynTheme.normal,
      vsync: this,
    );

    _fillAnimation = Tween<double>(
      begin: _previousValue / 100,
      end: widget.value / 100,
    ).animate(CurvedAnimation(
      parent: _fillController,
      curve: SynTheme.snapIn,
    ));

    _fillController.forward();
  }

  @override
  void didUpdateWidget(SynStatRing oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.value != widget.value) {
      _previousValue = oldWidget.value;
      _fillAnimation = Tween<double>(
        begin: _previousValue / 100,
        end: widget.value / 100,
      ).animate(CurvedAnimation(
        parent: _fillController,
        curve: SynTheme.snapIn,
      ));
      _fillController.forward(from: 0);
    }
  }

  @override
  void dispose() {
    _pulseController.dispose();
    _fillController.dispose();
    super.dispose();
  }

  Color get _ringColor {
    if (widget.color != null) return widget.color!;
    if (widget.value <= widget.criticalThreshold) {
      return SynTheme.accentHot;
    }
    if (widget.value <= widget.lowThreshold) {
      return SynTheme.accentWarm;
    }
    return SynTheme.accent;
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: Listenable.merge([_pulseAnimation, _fillAnimation]),
      builder: (context, _) {
        return SizedBox(
          width: widget.size,
          height: widget.size,
          child: CustomPaint(
            painter: _StatRingPainter(
              fillPercent: _fillAnimation.value,
              ringColor: _ringColor,
              pulseValue: _pulseAnimation.value,
              isCritical: widget.value <= widget.criticalThreshold,
              thickness: widget.thickness,
            ),
            child: Center(
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(
                    widget.icon,
                    color: _ringColor,
                    size: widget.size * 0.25,
                  ),
                  const SizedBox(height: 2),
                  Text(
                    '${widget.value.toInt()}',
                    style: SynTheme.title(color: _ringColor),
                  ),
                  Text(
                    widget.label.toUpperCase(),
                    style: SynTheme.caption(color: SynTheme.textMuted),
                  ),
                ],
              ),
            ),
          ),
        );
      },
    );
  }
}

class _StatRingPainter extends CustomPainter {
  final double fillPercent;
  final Color ringColor;
  final double pulseValue;
  final bool isCritical;
  final double thickness;

  _StatRingPainter({
    required this.fillPercent,
    required this.ringColor,
    required this.pulseValue,
    required this.isCritical,
    required this.thickness,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final center = Offset(size.width / 2, size.height / 2);
    final radius = (size.width - thickness) / 2;

    // Background ring
    final bgPaint = Paint()
      ..color = SynTheme.bgSurface
      ..style = PaintingStyle.stroke
      ..strokeWidth = thickness
      ..strokeCap = StrokeCap.round;

    canvas.drawCircle(center, radius, bgPaint);

    // Tick marks (outer)
    final tickPaint = Paint()
      ..color = ringColor.withOpacity(0.3)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1;

    for (var i = 0; i < 36; i++) {
      final angle = (i * 10) * math.pi / 180 - math.pi / 2;
      final innerR = radius + thickness / 2 + 2;
      final outerR = radius + thickness / 2 + (i % 9 == 0 ? 8 : 4);
      final start = Offset(
        center.dx + innerR * math.cos(angle),
        center.dy + innerR * math.sin(angle),
      );
      final end = Offset(
        center.dx + outerR * math.cos(angle),
        center.dy + outerR * math.sin(angle),
      );
      canvas.drawLine(start, end, tickPaint);
    }

    // Fill arc
    if (fillPercent > 0) {
      final sweepAngle = 2 * math.pi * fillPercent;

      // Glow effect
      final glowPaint = Paint()
        ..color = ringColor.withOpacity(pulseValue * (isCritical ? 0.6 : 0.3))
        ..style = PaintingStyle.stroke
        ..strokeWidth = thickness + 8
        ..strokeCap = StrokeCap.round
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 10);

      canvas.drawArc(
        Rect.fromCircle(center: center, radius: radius),
        -math.pi / 2,
        sweepAngle,
        false,
        glowPaint,
      );

      // Main arc
      final arcPaint = Paint()
        ..shader = SweepGradient(
          startAngle: -math.pi / 2,
          endAngle: -math.pi / 2 + sweepAngle,
          colors: [
            ringColor.withOpacity(0.6),
            ringColor,
          ],
        ).createShader(Rect.fromCircle(center: center, radius: radius))
        ..style = PaintingStyle.stroke
        ..strokeWidth = thickness
        ..strokeCap = StrokeCap.round;

      canvas.drawArc(
        Rect.fromCircle(center: center, radius: radius),
        -math.pi / 2,
        sweepAngle,
        false,
        arcPaint,
      );

      // End cap glow
      final endAngle = -math.pi / 2 + sweepAngle;
      final endPoint = Offset(
        center.dx + radius * math.cos(endAngle),
        center.dy + radius * math.sin(endAngle),
      );

      final capGlow = Paint()
        ..color = ringColor.withOpacity(pulseValue)
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 8);

      canvas.drawCircle(endPoint, thickness / 2 + 4, capGlow);
    }

    // Inner decorative ring
    final innerRing = Paint()
      ..color = ringColor.withOpacity(0.15)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1;

    canvas.drawCircle(center, radius - thickness / 2 - 4, innerRing);

    // Critical pulse overlay
    if (isCritical) {
      final pulsePaint = Paint()
        ..color = ringColor.withOpacity(pulseValue * 0.2)
        ..style = PaintingStyle.fill;

      canvas.drawCircle(center, radius - thickness / 2, pulsePaint);
    }
  }

  @override
  bool shouldRepaint(covariant _StatRingPainter oldDelegate) {
    return fillPercent != oldDelegate.fillPercent ||
        pulseValue != oldDelegate.pulseValue ||
        ringColor != oldDelegate.ringColor;
  }
}
