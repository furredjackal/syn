import 'package:flutter/material.dart';

class SynButton extends StatefulWidget {
  final String label;
  final VoidCallback onPressed;
  final bool isEnabled;
  final bool isLoading;
  final IconData? icon;
  final double width;

  const SynButton({
    Key? key,
    required this.label,
    required this.onPressed,
    this.isEnabled = true,
    this.isLoading = false,
    this.icon,
    this.width = double.infinity,
  }) : super(key: key);

  @override
  State<SynButton> createState() => _SynButtonState();
}

class _SynButtonState extends State<SynButton> with SingleTickerProviderStateMixin {
  late AnimationController _animationController;
  bool _isHovering = false;

  @override
  void initState() {
    super.initState();
    _animationController = AnimationController(duration: const Duration(milliseconds: 200), vsync: this);
  }

  @override
  void dispose() {
    _animationController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) {
        if (widget.isEnabled && !widget.isLoading) {
          setState(() => _isHovering = true);
          _animationController.forward();
        }
      },
      onExit: (_) {
        setState(() => _isHovering = false);
        _animationController.reverse();
      },
      child: GestureDetector(
        onTap: widget.isEnabled && !widget.isLoading ? widget.onPressed : null,
        child: AnimatedBuilder(
          animation: _animationController,
          builder: (context, child) => Container(
            width: widget.width,
            padding: const EdgeInsets.symmetric(vertical: 12, horizontal: 16),
            decoration: BoxDecoration(
              border: Border.all(color: const Color(0xFF00D9FF).withOpacity(_isHovering ? 1.0 : 0.6), width: 1.5),
              borderRadius: BorderRadius.circular(4),
              color: _isHovering ? const Color(0xFF00D9FF).withOpacity(0.1) : Colors.transparent,
            ),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              mainAxisSize: MainAxisSize.min,
              children: [
                if (widget.isLoading)
                  SizedBox(
                    width: 16,
                    height: 16,
                    child: CircularProgressIndicator(strokeWidth: 2, valueColor: AlwaysStoppedAnimation(Colors.cyan.withOpacity(0.7))),
                  )
                else if (widget.icon != null) ...[
                  Icon(widget.icon, color: widget.isEnabled ? Colors.cyan : Colors.grey, size: 18),
                  const SizedBox(width: 8),
                ],
                Text(
                  widget.label,
                  style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                        color: widget.isEnabled ? Colors.cyan : Colors.grey,
                        fontWeight: FontWeight.bold,
                      ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
