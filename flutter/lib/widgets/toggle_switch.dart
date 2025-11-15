import 'package:flutter/material.dart';

class ToggleSwitch extends StatefulWidget {
  final String label;
  final bool value;
  final ValueChanged<bool> onChanged;
  final String? subtitle;

  const ToggleSwitch({
    Key? key,
    required this.label,
    required this.value,
    required this.onChanged,
    this.subtitle,
  }) : super(key: key);

  @override
  State<ToggleSwitch> createState() => _ToggleSwitchState();
}

class _ToggleSwitchState extends State<ToggleSwitch> with SingleTickerProviderStateMixin {
  late AnimationController _animationController;

  @override
  void initState() {
    super.initState();
    _animationController = AnimationController(
      duration: const Duration(milliseconds: 300),
      vsync: this,
      value: widget.value ? 1.0 : 0.0,
    );
  }

  @override
  void didUpdateWidget(ToggleSwitch oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.value != widget.value) {
      _animationController.animateTo(widget.value ? 1.0 : 0.0);
    }
  }

  @override
  void dispose() {
    _animationController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 12),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(widget.label, style: Theme.of(context).textTheme.bodyMedium),
              if (widget.subtitle != null)
                Text(widget.subtitle!, style: Theme.of(context).textTheme.labelSmall?.copyWith(color: Colors.grey)),
            ],
          ),
          GestureDetector(
            onTap: () => widget.onChanged(!widget.value),
            child: AnimatedBuilder(
              animation: _animationController,
              builder: (context, child) => Container(
                width: 50,
                height: 28,
                decoration: BoxDecoration(
                  borderRadius: BorderRadius.circular(14),
                  border: Border.all(
                    color: Color.lerp(Colors.grey, const Color(0xFF00D9FF), _animationController.value) ?? Colors.grey,
                    width: 1.5,
                  ),
                  color: Color.lerp(Colors.black45, const Color(0xFF00D9FF).withOpacity(0.2), _animationController.value),
                ),
                child: Stack(
                  children: [
                    AnimatedPositioned(
                      duration: const Duration(milliseconds: 300),
                      left: widget.value ? 24 : 2,
                      top: 2,
                      child: Container(
                        width: 24,
                        height: 24,
                        decoration: BoxDecoration(
                          borderRadius: BorderRadius.circular(12),
                          color: widget.value ? const Color(0xFF00D9FF) : Colors.grey,
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
