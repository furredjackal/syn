import 'package:flutter/material.dart';
import 'dart:math';

class PieChartWidget extends StatelessWidget {
  final Map<String, double> data;
  final double size;
  final String title;

  const PieChartWidget({
    Key? key,
    required this.data,
    this.size = 200,
    this.title = '',
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        if (title.isNotEmpty)
          Text(
            title,
            style: Theme.of(context).textTheme.titleMedium?.copyWith(color: Colors.cyan),
          ),
        if (title.isNotEmpty) const SizedBox(height: 8),
        SizedBox(
          width: size,
          height: size,
          child: CustomPaint(
            painter: PieChartPainter(data: data),
          ),
        ),
        const SizedBox(height: 12),
        Wrap(
          spacing: 16,
          runSpacing: 8,
          children: data.entries.map((entry) {
            final total = data.values.fold(0.0, (a, b) => a + b);
            final percentage = (entry.value / total * 100).toStringAsFixed(1);
            final color = _getColorForIndex(data.keys.toList().indexOf(entry.key));
            return Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Container(
                  width: 12,
                  height: 12,
                  decoration: BoxDecoration(
                    color: color,
                    borderRadius: BorderRadius.circular(2),
                  ),
                ),
                const SizedBox(width: 4),
                Text(
                  '${entry.key} ($percentage%)',
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(color: Colors.grey[400]),
                ),
              ],
            );
          }).toList(),
        ),
      ],
    );
  }

  Color _getColorForIndex(int index) {
    final colors = [
      Colors.cyan,
      Colors.purple,
      Colors.amber,
      Colors.green,
      Colors.red,
      Colors.blue,
    ];
    return colors[index % colors.length];
  }
}

class PieChartPainter extends CustomPainter {
  final Map<String, double> data;

  PieChartPainter({required this.data});

  @override
  void paint(Canvas canvas, Size size) {
    final center = Offset(size.width / 2, size.height / 2);
    final radius = min(size.width, size.height) / 2 - 10;
    final total = data.values.fold(0.0, (a, b) => a + b);

    double currentAngle = -pi / 2;
    int colorIndex = 0;

    final colors = [
      Colors.cyan,
      Colors.purple,
      Colors.amber,
      Colors.green,
      Colors.red,
      Colors.blue,
    ];

    data.forEach((key, value) {
      final sweepAngle = (value / total) * 2 * pi;
      final paint = Paint()
        ..color = colors[colorIndex % colors.length]
        ..style = PaintingStyle.fill;

      canvas.drawArc(
        Rect.fromCircle(center: center, radius: radius),
        currentAngle,
        sweepAngle,
        true,
        paint,
      );

      // Draw border
      final borderPaint = Paint()
        ..color = Colors.black
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1;

      canvas.drawArc(
        Rect.fromCircle(center: center, radius: radius),
        currentAngle,
        sweepAngle,
        true,
        borderPaint,
      );

      // Draw percentage label
      final labelAngle = currentAngle + sweepAngle / 2;
      final labelRadius = radius * 0.7;
      final labelX = center.dx + labelRadius * cos(labelAngle);
      final labelY = center.dy + labelRadius * sin(labelAngle);

      final percentage = (value / total * 100).toStringAsFixed(0);
      final textPainter = TextPainter(
        text: TextSpan(
          text: '$percentage%',
          style: const TextStyle(color: Colors.white, fontSize: 12, fontWeight: FontWeight.bold),
        ),
        textDirection: TextDirection.ltr,
      );
      textPainter.layout();
      textPainter.paint(
        canvas,
        Offset(labelX - textPainter.width / 2, labelY - textPainter.height / 2),
      );

      currentAngle += sweepAngle;
      colorIndex++;
    });
  }

  @override
  bool shouldRepaint(PieChartPainter oldDelegate) => oldDelegate.data != data;
}
