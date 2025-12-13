import 'package:flutter/material.dart';
import '../theme/syn_theme.dart';
import '../helpers/animated_builder.dart';

/// Animated stat bar with Persona 5 aesthetics.
///
/// Features:
/// - Smooth fill animation on value changes
/// - Pulsing glow at current value
/// - Optional danger/warning colors at thresholds
/// - Diagonal slash end cap
/// - Value change indicator (+/- delta)
class SynStatBar extends StatefulWidget {
  /// Current value (0-100)
  final double value;

  /// Stat label
  final String label;

  /// Optional icon
  final IconData? icon;

  /// Bar color (defaults to accent)
  final Color? color;

  /// Show numeric value
  final bool showValue;

  /// Previous value for delta display
  final double? previousValue;

  /// Width of the bar
  final double width;

  /// Height of the bar
  final double height;

  /// Low threshold for warning color (default 25)
  final double lowThreshold;

  /// Critical threshold for danger color (default 10)
  final double criticalThreshold;

  const SynStatBar({
    super.key,
    required this.value,
    required this.label,
    this.icon,
    this.color,
    this.showValue = true,
    this.previousValue,
    this.width = 200,
    this.height = 24,
    this.lowThreshold = 25,
    this.criticalThreshold = 10,
  });

  @override
  State<SynStatBar> createState() => _SynStatBarState();
}

class _SynStatBarState extends State<SynStatBar>
    with SingleTickerProviderStateMixin {
  late AnimationController _pulseController;
  late Animation<double> _pulseAnimation;

  @override
  void initState() {
    super.initState();
    _pulseController = AnimationController(
      duration: const Duration(milliseconds: 2000),
      vsync: this,
    )..repeat(reverse: true);
    
    _pulseAnimation = Tween<double>(begin: 0.3, end: 0.7).animate(
      CurvedAnimation(parent: _pulseController, curve: Curves.easeInOut),
    );
  }

  @override
  void dispose() {
    _pulseController.dispose();
    super.dispose();
  }

  Color get _barColor {
    if (widget.color != null) return widget.color!;
    if (widget.value <= widget.criticalThreshold) {
      return SynTheme.accentHot;
    }
    if (widget.value <= widget.lowThreshold) {
      return SynTheme.accentWarm;
    }
    return SynTheme.accent;
  }

  double? get _delta {
    if (widget.previousValue == null) return null;
    return widget.value - widget.previousValue!;
  }

  @override
  Widget build(BuildContext context) {
    final fillPercent = (widget.value / 100).clamp(0.0, 1.0);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        // Label row
        Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            if (widget.icon != null) ...[
              Icon(
                widget.icon,
                color: _barColor,
                size: 16,
              ),
              const SizedBox(width: 8),
            ],
            Text(
              widget.label.toUpperCase(),
              style: SynTheme.caption(color: SynTheme.textSecondary),
            ),
            const Spacer(),
            if (widget.showValue) ...[
              Text(
                '${widget.value.toInt()}',
                style: SynTheme.label(color: _barColor),
              ),
              // Delta indicator
              if (_delta != null && _delta != 0) ...[
                const SizedBox(width: 6),
                _buildDeltaIndicator(),
              ],
            ],
          ],
        ),
        const SizedBox(height: 6),
        // Bar container
        SizedBox(
          width: widget.width,
          height: widget.height,
          child: AnimatedBuilder(
            animation: _pulseAnimation,
            builder: (context, _) {
              return CustomPaint(
                painter: _StatBarPainter(
                  fillPercent: fillPercent,
                  barColor: _barColor,
                  pulseValue: _pulseAnimation.value,
                  isCritical: widget.value <= widget.criticalThreshold,
                ),
              );
            },
          ),
        ),
      ],
    );
  }

  Widget _buildDeltaIndicator() {
    final isPositive = _delta! > 0;
    final color = isPositive ? Colors.greenAccent : SynTheme.accentHot;
    final prefix = isPositive ? '+' : '';

    return TweenAnimationBuilder<double>(
      tween: Tween(begin: 0, end: 1),
      duration: SynTheme.normal,
      curve: SynTheme.bounce,
      builder: (context, value, child) {
        return Transform.scale(
          scale: value,
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
            decoration: BoxDecoration(
              color: color.withOpacity(0.2),
              border: Border.all(color: color.withOpacity(0.5)),
            ),
            child: Text(
              '$prefix${_delta!.toInt()}',
              style: SynTheme.caption(color: color),
            ),
          ),
        );
      },
    );
  }
}

class _StatBarPainter extends CustomPainter {
  final double fillPercent;
  final Color barColor;
  final double pulseValue;
  final bool isCritical;

  _StatBarPainter({
    required this.fillPercent,
    required this.barColor,
    required this.pulseValue,
    required this.isCritical,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final slashWidth = size.height * 0.6;

    // Background track
    final bgPaint = Paint()
      ..color = SynTheme.bgSurface
      ..style = PaintingStyle.fill;

    final bgPath = Path()
      ..moveTo(slashWidth, 0)
      ..lineTo(size.width, 0)
      ..lineTo(size.width - slashWidth, size.height)
      ..lineTo(0, size.height)
      ..close();

    canvas.drawPath(bgPath, bgPaint);

    // Border
    final borderPaint = Paint()
      ..color = barColor.withOpacity(0.4)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1;

    canvas.drawPath(bgPath, borderPaint);

    // Fill bar
    if (fillPercent > 0) {
      final fillWidth = (size.width - slashWidth) * fillPercent + slashWidth;
      
      final fillPath = Path()
        ..moveTo(slashWidth, 0)
        ..lineTo(fillWidth, 0)
        ..lineTo(fillWidth - slashWidth, size.height)
        ..lineTo(0, size.height)
        ..close();

      // Gradient fill
      final fillPaint = Paint()
        ..shader = LinearGradient(
          colors: [
            barColor.withOpacity(0.8),
            barColor,
          ],
        ).createShader(Rect.fromLTWH(0, 0, fillWidth, size.height));

      canvas.drawPath(fillPath, fillPaint);

      // Glow at the edge
      final glowPaint = Paint()
        ..color = barColor.withOpacity(pulseValue * (isCritical ? 0.8 : 0.4))
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 8);

      final glowRect = Rect.fromLTWH(
        fillWidth - slashWidth - 10,
        0,
        20,
        size.height,
      );
      canvas.drawRect(glowRect, glowPaint);

      // Scanline effect (subtle)
      final scanPaint = Paint()
        ..color = Colors.white.withOpacity(0.1)
        ..strokeWidth = 1;

      for (var y = 0.0; y < size.height; y += 4) {
        canvas.drawLine(
          Offset(slashWidth, y),
          Offset(fillWidth - slashWidth, y),
          scanPaint,
        );
      }
    }

    // Critical pulse effect
    if (isCritical) {
      final pulsePaint = Paint()
        ..color = barColor.withOpacity(pulseValue * 0.3)
        ..maskFilter = const MaskFilter.blur(BlurStyle.outer, 15);
      
      canvas.drawPath(bgPath, pulsePaint);
    }
  }

  @override
  bool shouldRepaint(covariant _StatBarPainter oldDelegate) {
    return fillPercent != oldDelegate.fillPercent ||
        pulseValue != oldDelegate.pulseValue ||
        barColor != oldDelegate.barColor;
  }
}
