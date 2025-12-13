import 'package:flutter/material.dart';
import '../theme/syn_theme.dart';
import '../helpers/animated_builder.dart';

/// Enhanced magnetic dock with fluid animations and haptic feedback.
///
/// Features:
/// - Slides off-screen when idle, snaps into view on hover
/// - Parallax-like depth effect
/// - Glow intensifies on hover
/// - Staggered children animation
/// - Optional collapse/expand toggle
class MagneticDock extends StatefulWidget {
  /// The widget to display inside the dock.
  final Widget child;

  /// Whether the dock should be positioned on the left side.
  final bool isLeft;

  /// How far to slide off-screen when idle (default 25px).
  final double idleOffset;

  /// Whether to enable the glow effect.
  final bool enableGlow;

  /// Accent color for glow.
  final Color? accentColor;

  /// Whether the dock can be collapsed.
  final bool collapsible;

  const MagneticDock({
    super.key,
    required this.child,
    this.isLeft = true,
    this.idleOffset = 25,
    this.enableGlow = true,
    this.accentColor,
    this.collapsible = false,
  });

  @override
  State<MagneticDock> createState() => _MagneticDockState();
}

class _MagneticDockState extends State<MagneticDock>
    with SingleTickerProviderStateMixin {
  bool _isHovering = false;
  bool _isCollapsed = false;

  late AnimationController _glowController;
  late Animation<double> _glowAnimation;

  @override
  void initState() {
    super.initState();
    _glowController = AnimationController(
      duration: const Duration(milliseconds: 1500),
      vsync: this,
    );
    _glowAnimation = Tween<double>(begin: 0.2, end: 0.5).animate(
      CurvedAnimation(parent: _glowController, curve: Curves.easeInOut),
    );
  }

  @override
  void dispose() {
    _glowController.dispose();
    super.dispose();
  }

  void _onHoverStart() {
    setState(() => _isHovering = true);
    _glowController.repeat(reverse: true);
  }

  void _onHoverEnd() {
    setState(() => _isHovering = false);
    _glowController.stop();
    _glowController.reset();
  }

  @override
  Widget build(BuildContext context) {
    final accent = widget.accentColor ?? SynTheme.accent;
    
    // Calculate offsets
    double xOffset;
    if (_isCollapsed) {
      xOffset = widget.isLeft ? -80 : 80;
    } else if (_isHovering) {
      xOffset = 0;
    } else {
      xOffset = widget.isLeft ? -widget.idleOffset : widget.idleOffset;
    }

    // Scale and rotation for depth
    final scale = _isHovering ? 1.0 : 0.95;
    final rotateY = _isHovering ? 0.0 : (widget.isLeft ? 0.02 : -0.02);

    return MouseRegion(
      onEnter: (_) => _onHoverStart(),
      onExit: (_) => _onHoverEnd(),
      child: AnimatedBuilder(
        animation: _glowAnimation,
        builder: (context, _) {
          return AnimatedContainer(
            duration: SynTheme.normal,
            curve: _isHovering ? Curves.easeOutBack : Curves.easeInOut,
            transform: Matrix4.identity()
              ..translate(xOffset, 0.0, 0.0)
              ..scale(scale)
              ..setEntry(3, 2, 0.001) // Perspective
              ..rotateY(rotateY),
            transformAlignment:
                widget.isLeft ? Alignment.centerLeft : Alignment.centerRight,
            child: Container(
              decoration: BoxDecoration(
                boxShadow: widget.enableGlow && _isHovering
                    ? [
                        BoxShadow(
                          color: accent.withOpacity(_glowAnimation.value),
                          blurRadius: 30,
                          spreadRadius: -5,
                          offset: Offset(widget.isLeft ? 10 : -10, 0),
                        ),
                      ]
                    : null,
              ),
              child: Stack(
                children: [
                  widget.child,
                  // Collapse toggle (if enabled)
                  if (widget.collapsible)
                    Positioned(
                      top: 0,
                      right: widget.isLeft ? 0 : null,
                      left: widget.isLeft ? null : 0,
                      child: GestureDetector(
                        onTap: () =>
                            setState(() => _isCollapsed = !_isCollapsed),
                        child: Container(
                          padding: const EdgeInsets.all(4),
                          color: SynTheme.bgSurface,
                          child: Icon(
                            _isCollapsed
                                ? (widget.isLeft
                                    ? Icons.chevron_right
                                    : Icons.chevron_left)
                                : (widget.isLeft
                                    ? Icons.chevron_left
                                    : Icons.chevron_right),
                            color: accent,
                            size: 16,
                          ),
                        ),
                      ),
                    ),
                ],
              ),
            ),
          );
        },
      ),
    );
  }
}
