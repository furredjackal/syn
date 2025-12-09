import 'package:flutter/material.dart';

/// A widget that docks to the side of the screen and slides out on hover.
///
/// When idle, the widget slides 20px off-screen. When hovered,
/// it snaps into view with a smooth easeOutExpo animation curve.
class MagneticDock extends StatefulWidget {
  /// The widget to display inside the dock.
  final Widget child;

  /// Whether the dock should be positioned on the left side.
  /// If false, it will dock to the right side.
  final bool isLeft;

  const MagneticDock({
    super.key,
    required this.child,
    this.isLeft = true,
  });

  @override
  State<MagneticDock> createState() => _MagneticDockState();
}

class _MagneticDockState extends State<MagneticDock> {
  /// Whether the mouse is currently hovering over the dock.
  bool _isHovering = false;

  @override
  Widget build(BuildContext context) {
    // Calculate the offset based on dock side and hover state
    // When not hovering: slide 20px off-screen (negative for left, positive for right)
    // When hovering: snap to 0px
    final double offset = _isHovering ? 0.0 : (widget.isLeft ? -20.0 : 20.0);

    return MouseRegion(
      onEnter: (_) => setState(() => _isHovering = true),
      onExit: (_) => setState(() => _isHovering = false),
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 400),
        curve: Curves.easeOutExpo,
        transform: Matrix4.translationValues(offset, 0, 0),
        child: widget.child,
      ),
    );
  }
}
