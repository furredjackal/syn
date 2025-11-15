import 'package:flutter/material.dart';
import 'dart:math';

class RadarChartWidget extends StatelessWidget {
  final Map<String, double> data; // axis name -> value (-10 to 10)
  final double size;
  final Color primaryColor;
  final Color gridColor;

  const RadarChartWidget({
    Key? key,
    required this.data,
    this.size = 200,
    this.primaryColor = Colors.cyan,
    this.gridColor = Colors.grey,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      width: size,
      height: size,
      child: CustomPaint(
        painter: RadarChartPainter(
          data: data,
          primaryColor: primaryColor,
          gridColor: gridColor,
        ),
      ),
    );
  }
}

class RadarChartPainter extends CustomPainter {
  final Map<String, double> data;
  final Color primaryColor;
  final Color gridColor;

  RadarChartPainter({
    required this.data,
    required this.primaryColor,
    required this.gridColor,
  });

  @override
  void paint(Canvas canvas, Size size) {
    final center = Offset(size.width / 2, size.height / 2);
    final radius = size.width / 2 - 20;
    final axisCount = data.length;
    final angleStep = (2 * pi) / axisCount;

    // Draw grid circles
    final gridPaint = Paint()
      ..color = gridColor.withOpacity(0.3)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1;

    for (int i = 1; i <= 5; i++) {
      canvas.drawCircle(center, (radius / 5) * i, gridPaint);
    }

    // Draw grid lines and labels
    final textPainter = TextPainter(textDirection: TextDirection.ltr);
    final labelPaint = Paint()..color = gridColor.withOpacity(0.5);

    List<MapEntry<String, double>> entries = data.entries.toList();
    for (int i = 0; i < axisCount; i++) {
      final angle = angleStep * i - pi / 2;
      final x = center.dx + radius * cos(angle);
      final y = center.dy + radius * sin(angle);

      // Draw axis line
      canvas.drawLine(center, Offset(x, y), labelPaint);

      // Draw label
      textPainter.text = TextSpan(
        text: entries[i].key,
        style: const TextStyle(color: Colors.cyan, fontSize: 10),
      );
      textPainter.layout();
      textPainter.paint(
        canvas,
        Offset(x - textPainter.width / 2, y + 10),
      );
    }

    // Draw data polygon
    final dataPaint = Paint()
      ..color = primaryColor.withOpacity(0.6)
      ..style = PaintingStyle.fill;

    final dataStrokePaint = Paint()
      ..color = primaryColor
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2;

    final path = Path();
    for (int i = 0; i < axisCount; i++) {
      final angle = angleStep * i - pi / 2;
      final value = entries[i].value;
      final normalizedValue = (value + 10) / 20; // Convert -10..10 to 0..1
      final distance = radius * normalizedValue;
      final x = center.dx + distance * cos(angle);
      final y = center.dy + distance * sin(angle);

      if (i == 0) {
        path.moveTo(x, y);
      } else {
        path.lineTo(x, y);
      }
    }
    path.close();

    canvas.drawPath(path, dataPaint);
    canvas.drawPath(path, dataStrokePaint);

    // Draw value points
    final pointPaint = Paint()
      ..color = primaryColor
      ..style = PaintingStyle.fill;

    for (int i = 0; i < axisCount; i++) {
      final angle = angleStep * i - pi / 2;
      final value = entries[i].value;
      final normalizedValue = (value + 10) / 20;
      final distance = radius * normalizedValue;
      final x = center.dx + distance * cos(angle);
      final y = center.dy + distance * sin(angle);
      canvas.drawCircle(Offset(x, y), 4, pointPaint);
    }
  }

  @override
  bool shouldRepaint(RadarChartPainter oldDelegate) =>
      oldDelegate.data != data ||
      oldDelegate.primaryColor != primaryColor ||
      oldDelegate.gridColor != gridColor;
}
