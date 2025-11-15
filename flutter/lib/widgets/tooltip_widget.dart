import 'package:flutter/material.dart';

class TooltipWidget extends StatefulWidget {
  final Widget child;
  final String message;
  final Duration delay;

  const TooltipWidget({
    Key? key,
    required this.child,
    required this.message,
    this.delay = const Duration(milliseconds: 500),
  }) : super(key: key);

  @override
  State<TooltipWidget> createState() => _TooltipWidgetState();
}

class _TooltipWidgetState extends State<TooltipWidget> {
  bool _showTooltip = false;
  late Timer _timer;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) {
        _timer = Timer(widget.delay, () {
          if (mounted) setState(() => _showTooltip = true);
        });
      },
      onExit: (_) {
        _timer.cancel();
        setState(() => _showTooltip = false);
      },
      child: Stack(
        clipBehavior: Clip.none,
        children: [
          widget.child,
          if (_showTooltip)
            Positioned(
              top: -40,
              left: 0,
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                decoration: BoxDecoration(
                  color: Colors.black.withOpacity(0.9),
                  border: Border.all(color: Colors.cyan.withOpacity(0.5)),
                  borderRadius: BorderRadius.circular(4),
                ),
                child: Text(
                  widget.message,
                  style: Theme.of(context).textTheme.labelSmall?.copyWith(color: Colors.cyan),
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
            ),
        ],
      ),
    );
  }

  @override
  void dispose() {
    _timer.cancel();
    super.dispose();
  }
}
