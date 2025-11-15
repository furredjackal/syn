import 'package:flutter/material.dart';

class TextButtonWidget extends StatefulWidget {
  final String label;
  final VoidCallback onPressed;
  final bool isEnabled;
  final Color? color;

  const TextButtonWidget({
    Key? key,
    required this.label,
    required this.onPressed,
    this.isEnabled = true,
    this.color,
  }) : super(key: key);

  @override
  State<TextButtonWidget> createState() => _TextButtonWidgetState();
}

class _TextButtonWidgetState extends State<TextButtonWidget> {
  bool _isHovering = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _isHovering = true),
      onExit: (_) => setState(() => _isHovering = false),
      child: GestureDetector(
        onTap: widget.isEnabled ? widget.onPressed : null,
        child: Text(
          widget.label,
          style: Theme.of(context).textTheme.bodyMedium?.copyWith(
            color: _isHovering && widget.isEnabled
              ? (widget.color ?? Colors.cyan)
              : (widget.isEnabled ? Colors.white : Colors.grey),
            decoration: _isHovering ? TextDecoration.underline : null,
            fontWeight: FontWeight.bold,
          ),
        ),
      ),
    );
  }
}
