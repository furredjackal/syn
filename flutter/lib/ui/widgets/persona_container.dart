import 'package:flutter/material.dart';

/// A stylized container widget that applies a skewed transform to create
/// a Persona 5-style aesthetic with high-contrast borders and shadows.
///
/// The container content is counter-skewed to remain readable while the
/// container itself maintains the signature slanted appearance.
class PersonaContainer extends StatelessWidget {
  /// The widget to display inside the container.
  final Widget child;

  /// The background color of the container.
  final Color color;

  /// The skew factor to apply to the container.
  /// Positive values skew right, negative values skew left.
  /// Default is -0.15 for the signature Persona aesthetic.
  final double skew;

  const PersonaContainer({
    super.key,
    required this.child,
    this.color = Colors.black,
    this.skew = -0.15,
  });

  @override
  Widget build(BuildContext context) {
    return Transform(
      transform: Matrix4.skewX(skew),
      child: Container(
        decoration: BoxDecoration(
          color: color,
          border: Border.all(
            color: Colors.cyanAccent,
            width: 2.0,
          ),
          boxShadow: [
            BoxShadow(
              color: Colors.cyanAccent.withValues(alpha: 0.3),
              offset: const Offset(4, 4),
              blurRadius: 8,
              spreadRadius: 0,
            ),
            BoxShadow(
              color: Colors.black.withValues(alpha: 0.9),
              offset: const Offset(6, 6),
              blurRadius: 0,
              spreadRadius: 0,
            ),
          ],
        ),
        child: Transform(
          // CRUCIAL: Counter-skew the child content so it remains readable/upright
          transform: Matrix4.skewX(-skew),
          child: child,
        ),
      ),
    );
  }
}
