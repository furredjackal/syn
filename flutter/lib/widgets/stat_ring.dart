import 'package:flutter/material.dart';

class StatRing extends StatefulWidget {
  final String label;
  final double value;
  final double maxValue;
  final Color color;
  final double size;

  const StatRing({
    Key? key,
    required this.label,
    required this.value,
    this.maxValue = 100,
    required this.color,
    this.size = 80,
  }) : super(key: key);

  @override
  State<StatRing> createState() => _StatRingState();
}

class _StatRingState extends State<StatRing>
    with SingleTickerProviderStateMixin {
  late AnimationController _glowController;

  @override
  void initState() {
    super.initState();
    _glowController = AnimationController(
      duration: const Duration(milliseconds: 2000),
      vsync: this,
    )..repeat(reverse: true);
  }

  @override
  void dispose() {
    _glowController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final percentage = (widget.value / widget.maxValue).clamp(0.0, 1.0);

    return Center(
      child: SizedBox(
        width: widget.size + 20,
        height: widget.size + 20,
        child: Stack(
          alignment: Alignment.center,
          children: [
            // Glow effect
            AnimatedBuilder(
              animation: _glowController,
              builder: (context, child) {
                return Container(
                  width: widget.size + 20,
                  height: widget.size + 20,
                  decoration: BoxDecoration(
                    shape: BoxShape.circle,
                    boxShadow: [
                      BoxShadow(
                        color: widget.color
                            .withOpacity(0.3 * _glowController.value),
                        blurRadius: 20,
                        spreadRadius: 5,
                      ),
                    ],
                  ),
                );
              },
            ),
            // Ring container
            SizedBox(
              width: widget.size,
              height: widget.size,
              child: Stack(
                alignment: Alignment.center,
                children: [
                  // Background ring
                  SizedBox.expand(
                    child: CircularProgressIndicator(
                      value: 1.0,
                      strokeWidth: 4,
                      valueColor: AlwaysStoppedAnimation(Colors.white10),
                    ),
                  ),
                  // Progress ring
                  SizedBox.expand(
                    child: CircularProgressIndicator(
                      value: percentage,
                      strokeWidth: 4,
                      valueColor: AlwaysStoppedAnimation(widget.color),
                    ),
                  ),
                  // Center content
                  Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Text(
                        widget.value.toStringAsFixed(0),
                        style: Theme.of(context)
                            .textTheme
                            .titleSmall
                            ?.copyWith(color: widget.color),
                      ),
                      Text(
                        widget.label,
                        style: Theme.of(context)
                            .textTheme
                            .labelSmall
                            ?.copyWith(color: Colors.grey),
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}
