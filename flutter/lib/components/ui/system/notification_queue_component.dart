
import 'package:flame/components.dart';
import 'package:flutter/material.dart';

/// Notification queue that stacks angled toasts near the top-right corner.
class NotificationQueueComponent extends PositionComponent {
  NotificationQueueComponent({List<String>? initialMessages})
      : _messages = List.of(initialMessages ?? []);

  final List<String> _messages;
  final double _cardWidth = 260;
  final double _cardHeight = 40;

  void addMessage(String text) {
    _messages.insert(0, text);
    if (_messages.length > 4) {
      _messages.removeLast();
    }
  }

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    for (var i = 0; i < _messages.length; i++) {
      final y = i * (_cardHeight + 8);
      final rrect = RRect.fromRectAndRadius(
        Rect.fromLTWH(0, y.toDouble(), _cardWidth, _cardHeight),
        const Radius.circular(10),
      );
      canvas.drawShadow(Path()..addRRect(rrect), const Color(0xAA000000), 8, false);
      canvas.drawRRect(
        rrect,
        Paint()..color = const Color(0xFF101828),
      );
      canvas.drawRRect(
        rrect,
        Paint()
          ..style = PaintingStyle.stroke
          ..color = const Color(0xFF00D9FF),
      );
      final textPainter = TextPaint(
        style: const TextStyle(
          color: Colors.white,
          fontSize: 12,
          fontWeight: FontWeight.w700,
        ),
      ).toTextPainter(_messages[i]);
      textPainter.paint(
        canvas,
        Offset(12, y + (_cardHeight - textPainter.height) / 2),
      );
    }
  }
}
