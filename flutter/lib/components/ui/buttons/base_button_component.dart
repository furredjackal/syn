import 'package:flame/components.dart';
import 'package:flame/events.dart';
import 'package:flutter/material.dart';

import '../../../syn_game.dart';

/// Base button rendered as a simple rounded rectangle.
class BaseButtonComponent extends PositionComponent
    with HasGameReference<SynGame>, TapCallbacks {
  BaseButtonComponent({
    VoidCallback? onTap,
    super.position,
    super.size,
    super.anchor,
    this.hoverColor,
    Color baseColor = const Color(0xFF101420),
    Color pressedColor = const Color(0xFF1A2438),
    Color? strokeColor,
    double strokeWidth = 0,
  })  : onTap = onTap ?? _noop,
        _baseColor = baseColor,
        _pressedColor = pressedColor,
        _strokeColor = strokeColor,
        _strokeWidth = strokeWidth;

  final VoidCallback onTap;
  final Color _baseColor;
  final Color _pressedColor;
  final Color? hoverColor;
  final Color? _strokeColor;
  final double _strokeWidth;

  late final RectangleComponent _background;
  RectangleComponent? _strokeOverlay;
  bool _isPressed = false;
  bool _isHovered = false;

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    _background = RectangleComponent(
      size: size,
      paint: Paint()..color = _baseColor,
    );
    add(_background);

    if (_strokeColor != null && _strokeWidth > 0) {
      final strokeOverlay = RectangleComponent(
        size: size,
        paint: Paint()
          ..color = _strokeColor!
          ..style = PaintingStyle.stroke
          ..strokeWidth = _strokeWidth,
      );
      _strokeOverlay = strokeOverlay;
      add(strokeOverlay);
    }
  }

  @override
  void update(double dt) {
    super.update(dt);
    _background.size = size;
    _strokeOverlay?.size = size;
  }

  @override
  void onTapDown(TapDownEvent event) {
    _isPressed = true;
    _background.paint.color = _pressedColor;
  }

  @override
  void onTapUp(TapUpEvent event) {
    if (_isPressed) {
      onTap();
      _isPressed = false;
      _background.paint.color =
          _isHovered && hoverColor != null ? hoverColor! : _baseColor;
    }
  }

  @override
  void onTapCancel(TapCancelEvent event) {
    _isPressed = false;
    _background.paint.color =
        _isHovered && hoverColor != null ? hoverColor! : _baseColor;
  }

  void setHovered(bool hovered) {
    _isHovered = hovered;
    if (_isPressed) {
      return;
    }
    _background.paint.color =
        hovered && hoverColor != null ? hoverColor! : _baseColor;
  }

  static void _noop() {}
}
