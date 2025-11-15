import 'package:flutter/material.dart';

class SliderControl extends StatelessWidget {
  final String label;
  final double value;
  final double min;
  final double max;
  final ValueChanged<double> onChanged;
  final String? unit;

  const SliderControl({
    Key? key,
    required this.label,
    required this.value,
    this.min = 0,
    this.max = 100,
    required this.onChanged,
    this.unit,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(label, style: Theme.of(context).textTheme.bodyMedium),
              Text(
                '${value.toStringAsFixed(0)}${unit ?? ''}',
                style: Theme.of(context).textTheme.bodySmall?.copyWith(color: Colors.cyan),
              ),
            ],
          ),
          const SizedBox(height: 8),
          SliderTheme(
            data: SliderThemeData(
              trackHeight: 4,
              thumbShape: const RoundSliderThumbShape(enabledThumbRadius: 8),
              activeTrackColor: const Color(0xFF00D9FF),
              inactiveTrackColor: Colors.grey.withOpacity(0.3),
              thumbColor: const Color(0xFF00D9FF),
              overlayColor: const Color(0xFF00D9FF).withOpacity(0.2),
            ),
            child: Slider(
              value: value.clamp(min, max),
              min: min,
              max: max,
              onChanged: onChanged,
            ),
          ),
        ],
      ),
    );
  }
}
