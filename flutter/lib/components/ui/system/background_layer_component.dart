import 'package:flame/components.dart';
import 'package:flutter/material.dart';

/// Ambient background: dark base with subtle slashes and a soft grid.
class BackgroundLayerComponent extends PositionComponent {
  @override
  void onGameResize(Vector2 size) {
    super.onGameResize(size);
    this.size = size;
  }

  @override
  void render(Canvas canvas) {
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    final base = Paint()
      ..shader = const LinearGradient(
        colors: [Color(0xFF05050A), Color(0xFF0A0A0F)],
        begin: Alignment.topLeft,
        end: Alignment.bottomRight,
      ).createShader(rect);
    canvas.drawRect(rect, base);

    _drawGrid(canvas);
    _drawSlashes(canvas);
    _drawNoise(canvas);
  }

  void _drawGrid(Canvas canvas) {
    final paint = Paint()
      ..color = const Color(0x2200D9FF)
      ..strokeWidth = 1.0;
    const cell = 60.0;
    for (double x = -size.y * 0.2; x < size.x + size.y * 0.2; x += cell) {
      canvas.drawLine(
        Offset(x, 0),
        Offset(x + size.y * 0.2, size.y),
        paint,
      );
    }
    for (double y = 0; y < size.y + cell; y += cell) {
      canvas.drawLine(
        Offset(0, y),
        Offset(size.x, y),
        paint,
      );
    }
  }

  void _drawSlashes(Canvas canvas) {
    final slashPaint = Paint()
      ..shader = const LinearGradient(
        colors: [
          Color(0x2217D2FF),
          Color(0x3300FFC8),
        ],
        begin: Alignment.topLeft,
        end: Alignment.bottomRight,
      ).createShader(Rect.fromLTWH(0, 0, size.x, size.y));
    final bands = [
      Rect.fromLTWH(size.x * 0.05, size.y * -0.1, size.x * 0.35, size.y * 0.4),
      Rect.fromLTWH(size.x * 0.6, size.y * 0.2, size.x * 0.32, size.y * 0.38),
      Rect.fromLTWH(size.x * -0.12, size.y * 0.55, size.x * 0.4, size.y * 0.4),
    ];
    for (final band in bands) {
      final path = Path()
        ..moveTo(band.left, band.top + band.height * 0.2)
        ..lineTo(band.right, band.top)
        ..lineTo(band.right - band.width * 0.18, band.bottom)
        ..lineTo(band.left - band.width * 0.12, band.bottom)
        ..close();
      canvas.drawPath(path, slashPaint);
    }
  }

  void _drawNoise(Canvas canvas) {
    final dotPaint = Paint()..color = const Color(0x1100FFFF);
    const spacing = 32.0;
    for (double y = 0; y < size.y; y += spacing) {
      for (double x = (y / spacing) % 2 == 0 ? 0 : spacing / 2;
          x < size.x;
          x += spacing) {
        canvas.drawCircle(Offset(x, y), 1.2, dotPaint);
      }
    }
  }
}
