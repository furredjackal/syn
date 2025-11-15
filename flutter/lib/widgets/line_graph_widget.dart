import 'package:flutter/material.dart';

class LineGraphWidget extends StatelessWidget {
  final List<double> dataPoints;
  final List<String> labels;
  final double height;
  final Color lineColor;
  final Color backgroundColor;
  final String title;

  const LineGraphWidget({
    Key? key,
    required this.dataPoints,
    this.labels = const [],
    this.height = 200,
    this.lineColor = Colors.cyan,
    this.backgroundColor = Colors.black,
    this.title = '',
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        if (title.isNotEmpty)
          Text(
            title,
            style: Theme.of(context).textTheme.titleMedium?.copyWith(color: Colors.cyan),
          ),
        if (title.isNotEmpty) const SizedBox(height: 8),
        SizedBox(
          height: height,
          child: CustomPaint(
            painter: LineGraphPainter(
              dataPoints: dataPoints,
              labels: labels,
              lineColor: lineColor,
              backgroundColor: backgroundColor,
            ),
            size: Size.infinite,
          ),
        ),
      ],
    );
  }
}

class LineGraphPainter extends CustomPainter {
  final List<double> dataPoints;
  final List<String> labels;
  final Color lineColor;
  final Color backgroundColor;

  LineGraphPainter({
    required this.dataPoints,
    required this.labels,
    required this.lineColor,
    required this.backgroundColor,
  });

  @override
  void paint(Canvas canvas, Size size) {
    if (dataPoints.isEmpty) return;

    final padding = 40.0;
    final graphWidth = size.width - (padding * 2);
    final graphHeight = size.height - (padding * 2);

    // Draw background
    canvas.drawRect(
      Rect.fromLTWH(0, 0, size.width, size.height),
      Paint()..color = backgroundColor,
    );

    // Draw grid
    final gridPaint = Paint()
      ..color = Colors.grey.withOpacity(0.2)
      ..strokeWidth = 1;

    for (int i = 0; i <= 4; i++) {
      final y = padding + (graphHeight / 4) * i;
      canvas.drawLine(Offset(padding, y), Offset(size.width - padding, y), gridPaint);
    }

    // Find min and max
    final minValue = dataPoints.reduce((a, b) => a < b ? a : b);
    final maxValue = dataPoints.reduce((a, b) => a > b ? a : b);
    final range = maxValue - minValue + 1;

    // Draw line
    final linePaint = Paint()
      ..color = lineColor
      ..strokeWidth = 2
      ..strokeCap = StrokeCap.round
      ..strokeJoin = StrokeJoin.round;

    final path = Path();
    for (int i = 0; i < dataPoints.length; i++) {
      final x = padding + (graphWidth / (dataPoints.length - 1 > 0 ? dataPoints.length - 1 : 1)) * i;
      final normalizedValue = (dataPoints[i] - minValue) / range;
      final y = size.height - padding - (graphHeight * normalizedValue);

      if (i == 0) {
        path.moveTo(x, y);
      } else {
        path.lineTo(x, y);
      }
    }

    canvas.drawPath(path, linePaint);

    // Draw points
    final pointPaint = Paint()
      ..color = lineColor
      ..style = PaintingStyle.fill;

    for (int i = 0; i < dataPoints.length; i++) {
      final x = padding + (graphWidth / (dataPoints.length - 1 > 0 ? dataPoints.length - 1 : 1)) * i;
      final normalizedValue = (dataPoints[i] - minValue) / range;
      final y = size.height - padding - (graphHeight * normalizedValue);
      canvas.drawCircle(Offset(x, y), 4, pointPaint);
    }

    // Draw axes
    final axisPaint = Paint()
      ..color = Colors.grey
      ..strokeWidth = 1;

    canvas.drawLine(Offset(padding, padding), Offset(padding, size.height - padding), axisPaint);
    canvas.drawLine(Offset(padding, size.height - padding), Offset(size.width - padding, size.height - padding), axisPaint);

    // Draw labels
    final textPainter = TextPainter(textDirection: TextDirection.ltr);
    if (labels.isNotEmpty) {
      for (int i = 0; i < labels.length && i < dataPoints.length; i++) {
        final x = padding + (graphWidth / (dataPoints.length - 1 > 0 ? dataPoints.length - 1 : 1)) * i;
        textPainter.text = TextSpan(
          text: labels[i],
          style: const TextStyle(color: Colors.grey, fontSize: 8),
        );
        textPainter.layout();
        textPainter.paint(canvas, Offset(x - textPainter.width / 2, size.height - padding + 5));
      }
    }
  }

  @override
  bool shouldRepaint(LineGraphPainter oldDelegate) =>
      oldDelegate.dataPoints != dataPoints ||
      oldDelegate.lineColor != lineColor;
}
