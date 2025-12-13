import 'package:flutter/material.dart';
import '../theme/syn_theme.dart';
import '../helpers/animated_builder.dart';

/// Enhanced Persona-style container with hover effects and micro-interactions.
///
/// Features:
/// - Skewed transform with counter-skewed content
/// - Glow effect on hover
/// - Border pulse animation on focus
/// - Smooth scale response
/// - Optional clip path for diagonal cuts
class SynContainer extends StatefulWidget {
  /// The widget to display inside the container.
  final Widget child;

  /// The background color of the container.
  final Color? color;

  /// The skew factor. Positive = right, negative = left.
  final double skew;

  /// The accent/border color.
  final Color? accentColor;

  /// The border width.
  final double borderWidth;

  /// Whether to enable hover effects.
  final bool enableHover;

  /// Whether to apply glow effect.
  final bool enableGlow;

  /// Whether to apply the hard shadow.
  final bool enableShadow;

  /// Whether to clip diagonal corners.
  final bool clipCorners;

  /// Corner clip size (if clipCorners is true).
  final double clipSize;

  /// Callback when tapped.
  final VoidCallback? onTap;

  /// Padding inside the container.
  final EdgeInsets padding;

  const SynContainer({
    super.key,
    required this.child,
    this.color,
    this.skew = SynTheme.skewAngle,
    this.accentColor,
    this.borderWidth = SynTheme.borderWidth,
    this.enableHover = true,
    this.enableGlow = true,
    this.enableShadow = true,
    this.clipCorners = false,
    this.clipSize = 20,
    this.onTap,
    this.padding = const EdgeInsets.all(20),
  });

  @override
  State<SynContainer> createState() => _SynContainerState();
}

class _SynContainerState extends State<SynContainer>
    with SingleTickerProviderStateMixin {
  bool _isHovered = false;
  bool _isPressed = false;

  late AnimationController _glowController;
  late Animation<double> _glowAnimation;

  @override
  void initState() {
    super.initState();
    _glowController = AnimationController(
      duration: const Duration(milliseconds: 1500),
      vsync: this,
    );
    _glowAnimation = Tween<double>(begin: 0.3, end: 0.6).animate(
      CurvedAnimation(parent: _glowController, curve: Curves.easeInOut),
    );
  }

  @override
  void dispose() {
    _glowController.dispose();
    super.dispose();
  }

  void _onHoverStart() {
    if (!widget.enableHover) return;
    setState(() => _isHovered = true);
    _glowController.repeat(reverse: true);
  }

  void _onHoverEnd() {
    if (!widget.enableHover) return;
    setState(() => _isHovered = false);
    _glowController.stop();
    _glowController.reset();
  }

  void _onTapDown() {
    setState(() => _isPressed = true);
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
    final bgColor = widget.color ?? SynTheme.bgCard;
    final accent = widget.accentColor ?? SynTheme.accent;

    // Calculate scale based on interaction state
    final scale = _isPressed ? 0.98 : (_isHovered ? 1.02 : 1.0);

    Widget container = AnimatedBuilder(
      animation: _glowAnimation,
      builder: (context, child) {
        return AnimatedContainer(
          duration: SynTheme.fast,
          curve: SynTheme.snapIn,
          transform: Matrix4.identity()
            ..scale(scale)
            ..setEntry(3, 2, 0.001) // Perspective
            ..rotateY(_isHovered ? 0.01 : 0),
          transformAlignment: Alignment.center,
          decoration: BoxDecoration(
            color: bgColor,
            border: Border.all(
              color: _isHovered ? accent : accent.withOpacity(0.6),
              width: widget.borderWidth,
            ),
            boxShadow: [
              // Glow effect
              if (widget.enableGlow)
                BoxShadow(
                  color: accent.withOpacity(
                    _isHovered ? _glowAnimation.value : 0.2,
                  ),
                  blurRadius: _isHovered ? 25 : 10,
                  spreadRadius: _isHovered ? -2 : -5,
                ),
              // Hard shadow
              if (widget.enableShadow)
                BoxShadow(
                  color: Colors.black.withOpacity(_isHovered ? 0.9 : 0.7),
                  offset: Offset(_isHovered ? 8 : 5, _isHovered ? 8 : 5),
                  blurRadius: 0,
                ),
            ],
          ),
          child: Padding(
            padding: widget.padding,
            child: widget.child,
          ),
        );
      },
    );

    // Apply skew transform
    container = Transform(
      transform: Matrix4.skewX(widget.skew),
      alignment: Alignment.center,
      child: container,
    );

    // Optionally clip diagonal corners
    if (widget.clipCorners) {
      container = ClipPath(
        clipper: _DiagonalCornerClipper(clipSize: widget.clipSize),
        child: container,
      );
    }

    // Counter-skew the content so text is readable
    // Note: We need to wrap the entire thing so the child counter-skews
    return MouseRegion(
      onEnter: (_) => _onHoverStart(),
      onExit: (_) => _onHoverEnd(),
      cursor: widget.onTap != null
          ? SystemMouseCursors.click
          : SystemMouseCursors.basic,
      child: GestureDetector(
        onTapDown: widget.onTap != null ? (_) => _onTapDown() : null,
        onTapUp: widget.onTap != null ? (_) => _onTapUp() : null,
        onTapCancel: widget.onTap != null ? _onTapCancel : null,
        child: Transform(
          transform: Matrix4.skewX(widget.skew),
          alignment: Alignment.center,
          child: AnimatedBuilder(
            animation: _glowAnimation,
            builder: (context, _) {
              return AnimatedContainer(
                duration: SynTheme.fast,
                curve: SynTheme.snapIn,
                transform: Matrix4.identity()
                  ..scale(scale)
                  ..setEntry(3, 2, 0.001),
                transformAlignment: Alignment.center,
                decoration: BoxDecoration(
                  color: bgColor,
                  border: Border.all(
                    color: _isHovered ? accent : accent.withOpacity(0.6),
                    width: widget.borderWidth,
                  ),
                  boxShadow: [
                    if (widget.enableGlow)
                      BoxShadow(
                        color: accent.withOpacity(
                          _isHovered ? _glowAnimation.value : 0.2,
                        ),
                        blurRadius: _isHovered ? 25 : 10,
                        spreadRadius: _isHovered ? -2 : -5,
                      ),
                    if (widget.enableShadow)
                      BoxShadow(
                        color: Colors.black.withOpacity(_isHovered ? 0.9 : 0.7),
                        offset: Offset(_isHovered ? 8 : 5, _isHovered ? 8 : 5),
                        blurRadius: 0,
                      ),
                  ],
                ),
                child: Padding(
                  padding: widget.padding,
                  // Counter-skew the content
                  child: Transform(
                    transform: Matrix4.skewX(-widget.skew),
                    alignment: Alignment.center,
                    child: widget.child,
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

/// Clips diagonal corners for a more aggressive look
class _DiagonalCornerClipper extends CustomClipper<Path> {
  final double clipSize;

  _DiagonalCornerClipper({required this.clipSize});

  @override
  Path getClip(Size size) {
    final path = Path();
    
    // Start at top-left, but clipped
    path.moveTo(clipSize, 0);
    
    // Top-right corner clipped
    path.lineTo(size.width - clipSize, 0);
    path.lineTo(size.width, clipSize);
    
    // Bottom-right corner clipped
    path.lineTo(size.width, size.height - clipSize);
    path.lineTo(size.width - clipSize, size.height);
    
    // Bottom-left corner clipped
    path.lineTo(clipSize, size.height);
    path.lineTo(0, size.height - clipSize);
    
    // Back to top-left
    path.lineTo(0, clipSize);
    path.close();
    
    return path;
  }

  @override
  bool shouldReclip(covariant CustomClipper<Path> oldClipper) => false;
}
