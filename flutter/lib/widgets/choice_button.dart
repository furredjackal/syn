import 'package:flutter/material.dart';

class ChoiceButton extends StatefulWidget {
  final String label;
  final String description;
  final VoidCallback onPressed;
  final Map<String, int> impacts;
  final String? hotkey;

  const ChoiceButton({
    Key? key,
    required this.label,
    required this.description,
    required this.onPressed,
    this.impacts = const {},
    this.hotkey,
  }) : super(key: key);

  @override
  State<ChoiceButton> createState() => _ChoiceButtonState();
}

class _ChoiceButtonState extends State<ChoiceButton> with SingleTickerProviderStateMixin {
  late AnimationController _animationController;
  bool _isHovering = false;

  @override
  void initState() {
    super.initState();
    _animationController = AnimationController(
      duration: const Duration(milliseconds: 200),
      vsync: this,
    );
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
        setState(() => _isHovering = true);
        _animationController.forward();
      },
      onExit: (_) {
        setState(() => _isHovering = false);
        _animationController.reverse();
      },
      child: GestureDetector(
        onTap: widget.onPressed,
        child: AnimatedBuilder(
          animation: _animationController,
          builder: (context, child) => Container(
            decoration: BoxDecoration(
              border: Border.all(
                color: Color.lerp(Colors.cyan, Colors.white, _animationController.value) ?? Colors.cyan,
                width: 1.5,
              ),
              borderRadius: BorderRadius.circular(4),
              color: Color.lerp(Colors.transparent, Colors.cyan.withOpacity(0.05), _animationController.value),
            ),
            padding: const EdgeInsets.all(12),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    Expanded(
                      child: Text(
                        widget.label,
                        style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                          color: Colors.cyan,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                    ),
                    if (widget.hotkey != null)
                      Container(
                        padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                        decoration: BoxDecoration(
                          border: Border.all(color: Colors.cyan.withOpacity(0.5)),
                          borderRadius: BorderRadius.circular(2),
                        ),
                        child: Text(
                          widget.hotkey!,
                          style: Theme.of(context).textTheme.labelSmall?.copyWith(color: Colors.cyan),
                        ),
                      ),
                  ],
                ),
                const SizedBox(height: 6),
                Text(
                  widget.description,
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(color: Colors.grey),
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                ),
                if (widget.impacts.isNotEmpty) ...[
                  const SizedBox(height: 8),
                  Wrap(
                    spacing: 8,
                    children: widget.impacts.entries.map((e) {
                      final color = e.value > 0 ? Colors.green : e.value < 0 ? Colors.red : Colors.grey;
                      return Text(
                        '${e.key} ${e.value > 0 ? '+' : ''}${e.value}',
                        style: Theme.of(context).textTheme.labelSmall?.copyWith(color: color),
                      );
                    }).toList(),
                  ),
                ],
              ],
            ),
          ),
        ),
      ),
    );
  }
}
