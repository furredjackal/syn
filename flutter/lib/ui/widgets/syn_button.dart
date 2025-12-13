import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../theme/syn_theme.dart';
import '../helpers/animated_builder.dart';

/// High-impact button with Persona 5 × Destiny 2 aesthetics.
///
/// Features:
/// - Aggressive skew with counter-skewed text
/// - Pulsing glow on hover
/// - Satisfying press animation
/// - Optional icon with animated entrance
/// - Keyboard focus support
enum SynButtonStyle {
  /// Primary action - full accent background
  primary,
  /// Secondary action - outline only
  secondary,
  /// Danger/destructive action
  danger,
  /// Ghost - minimal, text only with hover reveal
  ghost,
}

class SynButton extends StatefulWidget {
  /// Button label text
  final String label;

  /// Optional leading icon
  final IconData? icon;

  /// Callback when pressed
  final VoidCallback? onPressed;

  /// Button style variant
  final SynButtonStyle style;

  /// Whether the button is in loading state
  final bool isLoading;

  /// Whether the button takes full width
  final bool fullWidth;

  /// Skew angle (use 0 for no skew)
  final double skew;

  /// Custom accent color override
  final Color? accentColor;

  const SynButton({
    super.key,
    required this.label,
    this.icon,
    this.onPressed,
    this.style = SynButtonStyle.primary,
    this.isLoading = false,
    this.fullWidth = false,
    this.skew = SynTheme.skewAngle,
    this.accentColor,
  });

  @override
  State<SynButton> createState() => _SynButtonState();
}

class _SynButtonState extends State<SynButton>
    with SingleTickerProviderStateMixin {
  bool _isHovered = false;
  bool _isPressed = false;
  bool _isFocused = false;

  late AnimationController _pulseController;
  late Animation<double> _pulseAnimation;

  @override
  void initState() {
    super.initState();
    _pulseController = AnimationController(
      duration: const Duration(milliseconds: 1200),
      vsync: this,
    );
    _pulseAnimation = Tween<double>(begin: 0.0, end: 1.0).animate(
      CurvedAnimation(parent: _pulseController, curve: Curves.easeInOut),
    );
  }

  @override
  void dispose() {
    _pulseController.dispose();
    super.dispose();
  }

  Color get _accentColor {
    if (widget.accentColor != null) return widget.accentColor!;
    switch (widget.style) {
      case SynButtonStyle.primary:
        return SynTheme.accent;
      case SynButtonStyle.secondary:
        return SynTheme.accent;
      case SynButtonStyle.danger:
        return SynTheme.accentHot;
      case SynButtonStyle.ghost:
        return SynTheme.textMuted;
    }
  }

  Color get _backgroundColor {
    switch (widget.style) {
      case SynButtonStyle.primary:
        if (_isPressed) return _accentColor.withOpacity(0.8);
        if (_isHovered) return _accentColor;
        return _accentColor.withOpacity(0.9);
      case SynButtonStyle.secondary:
      case SynButtonStyle.ghost:
        if (_isPressed) return _accentColor.withOpacity(0.2);
        if (_isHovered) return _accentColor.withOpacity(0.1);
        return Colors.transparent;
      case SynButtonStyle.danger:
        if (_isPressed) return _accentColor.withOpacity(0.8);
        if (_isHovered) return _accentColor;
        return _accentColor.withOpacity(0.9);
    }
  }

  Color get _textColor {
    switch (widget.style) {
      case SynButtonStyle.primary:
      case SynButtonStyle.danger:
        return SynTheme.bgBlack;
      case SynButtonStyle.secondary:
        if (_isHovered || _isPressed) return _accentColor;
        return _accentColor.withOpacity(0.8);
      case SynButtonStyle.ghost:
        if (_isHovered || _isPressed) return SynTheme.textPrimary;
        return SynTheme.textMuted;
    }
  }

  Color get _borderColor {
    switch (widget.style) {
      case SynButtonStyle.primary:
      case SynButtonStyle.danger:
        return _isHovered ? Colors.white.withOpacity(0.3) : Colors.transparent;
      case SynButtonStyle.secondary:
        return _accentColor.withOpacity(_isHovered ? 1.0 : 0.6);
      case SynButtonStyle.ghost:
        return _isHovered ? _accentColor.withOpacity(0.3) : Colors.transparent;
    }
  }

  void _handleHoverStart() {
    setState(() => _isHovered = true);
    _pulseController.repeat(reverse: true);
  }

  void _handleHoverEnd() {
    setState(() => _isHovered = false);
    _pulseController.stop();
    _pulseController.reset();
  }

  void _handleTapDown() {
    setState(() => _isPressed = true);
    HapticFeedback.lightImpact();
  }

  void _handleTapUp() {
    setState(() => _isPressed = false);
    widget.onPressed?.call();
  }

  void _handleTapCancel() {
    setState(() => _isPressed = false);
  }

  @override
  Widget build(BuildContext context) {
    final scale = _isPressed ? 0.95 : (_isHovered ? 1.03 : 1.0);
    final shadowOffset = _isPressed ? 2.0 : (_isHovered ? 6.0 : 4.0);

    Widget button = Focus(
      onFocusChange: (focused) => setState(() => _isFocused = focused),
      child: MouseRegion(
        onEnter: (_) => _handleHoverStart(),
        onExit: (_) => _handleHoverEnd(),
        cursor: widget.onPressed != null
            ? SystemMouseCursors.click
            : SystemMouseCursors.forbidden,
        child: GestureDetector(
          onTapDown: widget.onPressed != null ? (_) => _handleTapDown() : null,
          onTapUp: widget.onPressed != null ? (_) => _handleTapUp() : null,
          onTapCancel: widget.onPressed != null ? _handleTapCancel : null,
          child: AnimatedBuilder(
            animation: _pulseAnimation,
            builder: (context, _) {
              return AnimatedContainer(
                duration: SynTheme.fast,
                curve: SynTheme.snapIn,
                transform: Matrix4.identity()..scale(scale),
                transformAlignment: Alignment.center,
                padding: const EdgeInsets.symmetric(
                  horizontal: 28,
                  vertical: 14,
                ),
                decoration: BoxDecoration(
                  color: _backgroundColor,
                  border: Border.all(color: _borderColor, width: 2),
                  boxShadow: [
                    // Glow on hover
                    if (_isHovered && widget.style != SynButtonStyle.ghost)
                      BoxShadow(
                        color: _accentColor.withOpacity(
                          0.3 + (_pulseAnimation.value * 0.2),
                        ),
                        blurRadius: 20,
                        spreadRadius: -3,
                      ),
                    // Hard shadow
                    if (widget.style != SynButtonStyle.ghost)
                      BoxShadow(
                        color: Colors.black.withOpacity(0.8),
                        offset: Offset(shadowOffset, shadowOffset),
                        blurRadius: 0,
                      ),
                    // Focus ring
                    if (_isFocused)
                      BoxShadow(
                        color: _accentColor.withOpacity(0.5),
                        blurRadius: 0,
                        spreadRadius: 2,
                      ),
                  ],
                ),
                child: widget.isLoading
                    ? SizedBox(
                        width: 20,
                        height: 20,
                        child: CircularProgressIndicator(
                          strokeWidth: 2,
                          valueColor:
                              AlwaysStoppedAnimation<Color>(_textColor),
                        ),
                      )
                    : Row(
                        mainAxisSize: widget.fullWidth
                            ? MainAxisSize.max
                            : MainAxisSize.min,
                        mainAxisAlignment: MainAxisAlignment.center,
                        children: [
                          if (widget.icon != null) ...[
                            AnimatedContainer(
                              duration: SynTheme.fast,
                              transform: Matrix4.translationValues(
                                _isHovered ? -4 : 0,
                                0,
                                0,
                              ),
                              child: Icon(
                                widget.icon,
                                color: _textColor,
                                size: 20,
                              ),
                            ),
                            const SizedBox(width: 10),
                          ],
                          AnimatedDefaultTextStyle(
                            duration: SynTheme.fast,
                            style: SynTheme.label(color: _textColor).copyWith(
                              letterSpacing: _isHovered ? 3 : 1.5,
                            ),
                            child: Text(widget.label),
                          ),
                        ],
                      ),
              );
            },
          ),
        ),
      ),
    );

    // Apply skew
    if (widget.skew != 0) {
      button = Transform(
        transform: Matrix4.skewX(widget.skew),
        alignment: Alignment.center,
        child: Transform(
          transform: Matrix4.skewX(-widget.skew),
          alignment: Alignment.center,
          child: button,
        ),
      );
    }

    if (widget.fullWidth) {
      button = SizedBox(width: double.infinity, child: button);
    }

    return button;
  }
}
