
import 'package:flame/components.dart';
import 'package:flame/events.dart';
import 'package:flutter/material.dart';

import '../../../models/game_state.dart';

/// Compact NPC card showing relationship bars.
class NPCCardComponent extends PositionComponent with TapCallbacks {
  NPCCardComponent({
    required this.relationship,
    this.onTap,
    super.position,
    Vector2? size,
  }) : super(size: size ?? Vector2(220, 80));

  final RelationshipData relationship;
  final VoidCallback? onTap;
  bool _pressed = false;

  // Internal layout constants for styling and padding.
  static const double _kPadding = 12.0;
  static const double _kBarHeight = 8.0;
  static const double _kRadius = 12.0;
  static const double _kLabelToBarVerticalOffset = 14.0;

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final rrect = RRect.fromRectAndRadius(
      Rect.fromLTWH(0, 0, size.x, size.y),
      const Radius.circular(_kRadius),
    );
    canvas.drawRRect(
      rrect,
      Paint()
        ..color =
            _pressed ? const Color(0xFF0F1A2A) : const Color(0xFF0C1320),
    );
    canvas.drawRRect(
      rrect,
      Paint()
        ..style = PaintingStyle.stroke
        ..color = const Color(0x3300D9FF),
    );

    final name = TextPaint(
      style: const TextStyle(
        color: Colors.white,
        fontSize: 16,
        fontWeight: FontWeight.w800,
        letterSpacing: 0.8,
      ),
    ).toTextPainter(relationship.npcName.toUpperCase());
    name.paint(canvas, const Offset(_kPadding, 10));

    _drawBar(
      canvas,
      origin: const Offset(_kPadding, 36),
      label: 'BOND',
      value: _normalize(relationship.trust),
      color: const Color(0xFF00D9FF),
    );
    _drawBar(
      canvas,
      origin: const Offset(_kPadding, 56),
      label: 'CONFLICT',
      value: _normalize(relationship.resentment),
      color: const Color(0xFFFF4C4C),
    );
  }

  void _drawBar(
    Canvas canvas, {
    required Offset origin,
    required String label,
    required double value,
    required Color color,
  }) {
    final text = TextPaint(
      style: const TextStyle(
        color: Color(0xFFB8C2D6),
        fontSize: 11,
        fontWeight: FontWeight.w700,
      ),
    ).toTextPainter(label);
    text.paint(canvas, origin);

    final barY = origin.dy + _kLabelToBarVerticalOffset;
    final barWidth = size.x - (2 * _kPadding);
    final radius = const Radius.circular(4);
    canvas.drawRRect(
      RRect.fromRectAndRadius(
        Rect.fromLTWH(origin.dx, barY, barWidth, _kBarHeight),
        radius,
      ),
      Paint()..color = const Color(0x22111B2C),
    );
    canvas.drawRRect(
      RRect.fromRectAndRadius(
        Rect.fromLTWH(origin.dx, barY, barWidth * value, _kBarHeight),
        radius,
      ),
      Paint()..color = color,
    );

    final stateText = TextPaint(
      style: const TextStyle(
        color: Color(0xFF8FA3C2),
        fontSize: 11,
        fontWeight: FontWeight.w600,
      ),
    ).toTextPainter(relationship.state.toUpperCase());
    stateText.paint(
      canvas,
      Offset(size.x - stateText.width - _kPadding, origin.dy),
    );
  }

  double _normalize(double value) {
    return ((value + 10) / 20).clamp(0.0, 1.0);
  }

  @override
  void onTapDown(TapDownEvent event) {
    _pressed = true;
  }

  @override
  void onTapUp(TapUpEvent event) {
    _pressed = false;
    onTap?.call();
  }

  @override
  void onTapCancel(TapCancelEvent event) {
    _pressed = false;
  }
}
