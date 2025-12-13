import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../theme/syn_theme.dart';
import '../helpers/animated_builder.dart';

/// Animated dock icon with hover effects and micro-interactions.
///
/// Features:
/// - Scale and glow on hover
/// - Icon rotation/bounce on hover
/// - Tooltip with Persona styling
/// - Active state indicator
/// - Keyboard focus support
class SynDockIcon extends StatefulWidget {
  /// Icon to display
  final IconData icon;

  /// Tooltip label
  final String label;

  /// Callback when tapped
  final VoidCallback? onTap;

  /// Whether this icon is currently active/selected
  final bool isActive;

  /// Accent color override
  final Color? accentColor;

  /// Icon size
  final double size;

  /// Whether to show the label below the icon
  final bool showLabel;

  const SynDockIcon({
    super.key,
    required this.icon,
    required this.label,
    this.onTap,
    this.isActive = false,
    this.accentColor,
    this.size = 28,
    this.showLabel = false,
  });

  @override
  State<SynDockIcon> createState() => _SynDockIconState();
}

class _SynDockIconState extends State<SynDockIcon>
    with SingleTickerProviderStateMixin {
  bool _isHovered = false;
  bool _isPressed = false;

  late AnimationController _bounceController;
  late Animation<double> _bounceAnimation;
  late Animation<double> _rotateAnimation;

  @override
  void initState() {
    super.initState();
    _bounceController = AnimationController(
      duration: const Duration(milliseconds: 600),
      vsync: this,
    );

    _bounceAnimation = TweenSequence<double>([
      TweenSequenceItem(tween: Tween(begin: 1.0, end: 1.2), weight: 30),
      TweenSequenceItem(tween: Tween(begin: 1.2, end: 0.9), weight: 30),
      TweenSequenceItem(tween: Tween(begin: 0.9, end: 1.05), weight: 20),
      TweenSequenceItem(tween: Tween(begin: 1.05, end: 1.0), weight: 20),
    ]).animate(CurvedAnimation(
      parent: _bounceController,
      curve: Curves.easeOut,
    ));

    _rotateAnimation = TweenSequence<double>([
      TweenSequenceItem(tween: Tween(begin: 0.0, end: -0.1), weight: 25),
      TweenSequenceItem(tween: Tween(begin: -0.1, end: 0.1), weight: 50),
      TweenSequenceItem(tween: Tween(begin: 0.1, end: 0.0), weight: 25),
    ]).animate(CurvedAnimation(
      parent: _bounceController,
      curve: Curves.easeInOut,
    ));
  }

  @override
  void dispose() {
    _bounceController.dispose();
    super.dispose();
  }

  Color get _accent => widget.accentColor ?? SynTheme.accent;

  void _onHoverStart() {
    setState(() => _isHovered = true);
    _bounceController.forward(from: 0);
  }

  void _onHoverEnd() {
    setState(() => _isHovered = false);
  }

  void _onTapDown() {
    setState(() => _isPressed = true);
    HapticFeedback.selectionClick();
  }

  void _onTapUp() {
    setState(() => _isPressed = false);
    widget.onTap?.call();
  }

  void _onTapCancel() {
    setState(() => _isPressed = false);
  }

  @override
  Widget build(BuildContext context) {
    final baseScale = _isPressed ? 0.85 : 1.0;

    return Tooltip(
      message: widget.label,
      preferBelow: false,
      decoration: BoxDecoration(
        color: SynTheme.bgCard,
        border: Border.all(color: _accent.withOpacity(0.5)),
        boxShadow: SynTheme.dramaticShadow(_accent),
      ),
      textStyle: SynTheme.label(color: _accent),
      waitDuration: const Duration(milliseconds: 500),
      child: MouseRegion(
        onEnter: (_) => _onHoverStart(),
        onExit: (_) => _onHoverEnd(),
        cursor: widget.onTap != null
            ? SystemMouseCursors.click
            : SystemMouseCursors.basic,
        child: GestureDetector(
          onTapDown: widget.onTap != null ? (_) => _onTapDown() : null,
          onTapUp: widget.onTap != null ? (_) => _onTapUp() : null,
          onTapCancel: widget.onTap != null ? _onTapCancel : null,
          child: AnimatedBuilder(
            animation: _bounceController,
            builder: (context, _) {
              return Transform.scale(
                scale: baseScale * _bounceAnimation.value,
                child: Transform.rotate(
                  angle: _rotateAnimation.value,
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      // Icon container
                      AnimatedContainer(
                        duration: SynTheme.fast,
                        padding: const EdgeInsets.all(14),
                        decoration: BoxDecoration(
                          color: widget.isActive || _isHovered
                              ? _accent.withOpacity(0.15)
                              : SynTheme.bgSurface,
                          border: Border.all(
                            color: widget.isActive
                                ? _accent
                                : _isHovered
                                    ? _accent.withOpacity(0.8)
                                    : _accent.withOpacity(0.3),
                            width: widget.isActive ? 2 : 1,
                          ),
                          boxShadow: [
                            if (_isHovered || widget.isActive)
                              BoxShadow(
                                color: _accent.withOpacity(
                                  widget.isActive ? 0.4 : 0.2,
                                ),
                                blurRadius: 15,
                                spreadRadius: -3,
                              ),
                            BoxShadow(
                              color: Colors.black.withOpacity(0.5),
                              offset: Offset(
                                _isHovered ? 4 : 2,
                                _isHovered ? 4 : 2,
                              ),
                              blurRadius: 0,
                            ),
                          ],
                        ),
                        child: Icon(
                          widget.icon,
                          color: widget.isActive || _isHovered
                              ? _accent
                              : _accent.withOpacity(0.6),
                          size: widget.size,
                        ),
                      ),
                      // Active indicator
                      if (widget.isActive) ...[
                        const SizedBox(height: 6),
                        Container(
                          width: 20,
                          height: 3,
                          decoration: BoxDecoration(
                            color: _accent,
                            boxShadow: [
                              BoxShadow(
                                color: _accent.withOpacity(0.5),
                                blurRadius: 8,
                              ),
                            ],
                          ),
                        ),
                      ],
                      // Label (if shown)
                      if (widget.showLabel) ...[
                        const SizedBox(height: 6),
                        Text(
                          widget.label.toUpperCase(),
                          style: SynTheme.caption(
                            color: widget.isActive || _isHovered
                                ? _accent
                                : SynTheme.textMuted,
                          ),
                        ),
                      ],
                    ],
                  ),
                ),
              );
            },
          ),
        ),
      ),
    );
  }
}
