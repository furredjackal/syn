// File: flutter/lib/components/ui/buttons/menu_button.dart
import 'package:flutter/material.dart';

import '../syn_theme.dart';

class MenuButton extends StatefulWidget {
  const MenuButton({
    super.key,
    required this.label,
    this.onPressed,
    this.isDestructive = false,
  });

  final String label;
  final VoidCallback? onPressed;
  final bool isDestructive;

  @override
  State<MenuButton> createState() => _MenuButtonState();
}

class _MenuButtonState extends State<MenuButton> {
  bool _hovering = false;

  @override
  Widget build(BuildContext context) {
    final borderColor =
        widget.isDestructive ? SynColors.accentRed : SynColors.primaryCyan;
    final bgColor = _hovering
        ? borderColor.withValues(alpha: 0.2)
        : SynColors.bgPanel.withValues(alpha: 0.6);

    return MouseRegion(
      onEnter: (_) => setState(() => _hovering = true),
      onExit: (_) => setState(() => _hovering = false),
      child: GestureDetector(
        onTap: widget.onPressed,
        child: ClipPath(
          clipper: _MenuButtonClipper(),
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 150),
            margin: EdgeInsets.symmetric(vertical: SynLayout.paddingSmall),
            padding: EdgeInsets.symmetric(
              vertical: SynLayout.paddingSmall,
              horizontal: SynLayout.paddingLarge,
            ),
            decoration: BoxDecoration(
              color: bgColor,
              border: Border.all(
                color: borderColor,
                width: SynLayout.borderWidthNormal,
              ),
            ),
            child: Center(
              child: Text(
                widget.label,
                style: SynTextStyles.body.copyWith(
                  letterSpacing: 2,
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

class _MenuButtonClipper extends CustomClipper<Path> {
  @override
  Path getClip(Size size) {
    return Path()
      ..moveTo(12, 0)
      ..lineTo(size.width, 0)
      ..lineTo(size.width - 12, size.height)
      ..lineTo(0, size.height)
      ..close();
  }

  @override
  bool shouldReclip(covariant CustomClipper<Path> oldClipper) => false;
}
