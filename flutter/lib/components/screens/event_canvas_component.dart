// File: flutter/lib/components/screens/event_canvas_component.dart
import 'dart:ui';

import 'package:flame/components.dart';

class EventCanvasComponent extends PositionComponent {
  EventCanvasComponent({super.position, super.size});

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    size = size == Vector2.zero() ? Vector2(600, 400) : size;
  }

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(16)),
      Paint()..color = const Color(0x22111A2E),
    );
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(16)),
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = const Color(0xFF00D9FF),
    );
  }
}
