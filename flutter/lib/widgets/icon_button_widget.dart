import 'package:flutter/material.dart';

class IconButtonWidget extends StatefulWidget {
  final IconData icon;
  final VoidCallback onPressed;
  final bool isEnabled;
  final Color? color;
  final double size;
  final String? tooltip;

  const IconButtonWidget({
    Key? key,
    required this.icon,
    required this.onPressed,
    this.isEnabled = true,
    this.color,
    this.size = 24,
    this.tooltip,
  }) : super(key: key);

  @override
  State<IconButtonWidget> createState() => _IconButtonWidgetState();
}

class _IconButtonWidgetState extends State<IconButtonWidget> {
  bool _isHovering = false;

  @override
  Widget build(BuildContext context) {
    return Tooltip(
      message: widget.tooltip ?? '',
      child: MouseRegion(
        onEnter: (_) => setState(() => _isHovering = true),
        onExit: (_) => setState(() => _isHovering = false),
        child: GestureDetector(
          onTap: widget.isEnabled ? widget.onPressed : null,
          child: Container(
            padding: const EdgeInsets.all(8),
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              color: _isHovering ? Colors.cyan.withOpacity(0.1) : Colors.transparent,
            ),
            child: Icon(
              widget.icon,
              color: widget.isEnabled ? (widget.color ?? Colors.cyan) : Colors.grey,
              size: widget.size,
            ),
          ),
        ),
      ),
    );
  }
}
