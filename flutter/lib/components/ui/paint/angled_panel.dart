import 'package:flutter/material.dart';

void drawAngledPanel(
  Canvas canvas,
  Rect rect, {
  Color fill = const Color(0xFF000000),
  Color border = const Color(0xFFFFFFFF),
  double borderWidth = 2.0,
  bool cutTopLeft = false,
  bool cutTopRight = true,
  bool cutBottomLeft = false,
  bool cutBottomRight = false,
  double cutSize = 16.0,
}) {
  final double maxCut =
      cutSize.clamp(0.0, (rect.width < rect.height ? rect.width : rect.height) / 2);

  final path = Path();

  // Top-left
  if (cutTopLeft) {
    path.moveTo(rect.left + maxCut, rect.top);
  } else {
    path.moveTo(rect.left, rect.top);
  }

  // Top-right
  if (cutTopRight) {
    path.lineTo(rect.right - maxCut, rect.top);
    path.lineTo(rect.right, rect.top + maxCut);
  } else {
    path.lineTo(rect.right, rect.top);
  }

  // Bottom-right
  if (cutBottomRight) {
    path.lineTo(rect.right, rect.bottom - maxCut);
    path.lineTo(rect.right - maxCut, rect.bottom);
  } else {
    path.lineTo(rect.right, rect.bottom);
  }

  // Bottom-left
  if (cutBottomLeft) {
    path.lineTo(rect.left + maxCut, rect.bottom);
    path.lineTo(rect.left, rect.bottom - maxCut);
  } else {
    path.lineTo(rect.left, rect.bottom);
  }

  if (cutTopLeft) {
    path.lineTo(rect.left, rect.top + maxCut);
    path.lineTo(rect.left + maxCut, rect.top);
  } else {
    path.lineTo(rect.left, rect.top);
  }

  path.close();

  canvas.drawPath(path, Paint()..color = fill);
  if (borderWidth > 0) {
    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = borderWidth
        ..color = border,
    );
  }
}
