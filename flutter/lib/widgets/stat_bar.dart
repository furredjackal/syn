import 'package:flutter/material.dart';

class StatBar extends StatelessWidget {
  final String label;
  final int value;
  final int maxValue;
  final Color? customColor;

  const StatBar({
    required this.label,
    required this.value,
    this.maxValue = 100,
    this.customColor,
    Key? key,
  }) : super(key: key);

  Color get barColor {
    if (customColor != null) return customColor!;

    // Color based on value percentage
    final percentage = (value / maxValue).clamp(0.0, 1.0);
    if (percentage < 0.33) {
      return const Color(0xFFFF4444);
    } else if (percentage < 0.66) {
      return const Color(0xFFFFAA00);
    } else {
      return const Color(0xFF00FF00);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Text(
              label.toUpperCase(),
              style: Theme.of(context).textTheme.labelMedium?.copyWith(
                    color: Colors.white.withOpacity(0.8),
                  ),
            ),
            Text(
              '$value/$maxValue',
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: barColor,
                  ),
            ),
          ],
        ),
        const SizedBox(height: 6),
        ClipRRect(
          borderRadius: BorderRadius.circular(2),
          child: LinearProgressIndicator(
            value: (value / maxValue).clamp(0.0, 1.0),
            minHeight: 12,
            backgroundColor: Colors.white.withOpacity(0.1),
            valueColor: AlwaysStoppedAnimation<Color>(barColor),
          ),
        ),
      ],
    );
  }
}
