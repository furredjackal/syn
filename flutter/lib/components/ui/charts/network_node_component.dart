import 'package:flame/components.dart';
import 'package:flame/events.dart';
import 'package:flutter/material.dart';

/// A draggable node in the relationship network.
class NetworkNodeComponent extends PositionComponent with DragCallbacks {
  NetworkNodeComponent({
    required Vector2 position,
    required this.label,
  }) : super(
          position: position,
          size: Vector2.all(50),
          anchor: Anchor.center,
        );

  final String label;

  @override
  void onDragUpdate(DragUpdateEvent event) {
    position += event.delta;
  }

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final paint = Paint()
      ..color = Colors.white
      ..style = PaintingStyle.stroke;
    canvas.drawCircle(Offset.zero, size.x / 2, paint);

    final textPainter = TextPainter(
      text: TextSpan(
        text: label,
        style: const TextStyle(
          color: Colors.white,
          fontSize: 12,
        ),
      ),
      textDirection: TextDirection.ltr,
    );
    textPainter.layout();
    textPainter.paint(
      canvas,
      Offset(
        -textPainter.width / 2,
        -textPainter.height / 2,
      ),
    );
  }
}