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
  }) : onTap = onTap ?? _noop;

  final VoidCallback onTap;

  late final RectangleComponent _background;
  bool _isPressed = false;

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    _background = RectangleComponent(
      size: size,
      paint: Paint()..color = const Color(0xFF101420),
    );
    add(_background);
  }

  @override
  void onTapDown(TapDownEvent event) {
    _isPressed = true;
    _background.paint.color = const Color(0xFF1A2438);
  }

  @override
  void onTapUp(TapUpEvent event) {
    if (_isPressed) {
      onTap();
      _isPressed = false;
      _background.paint.color = const Color(0xFF101420);
    }
  }

  @override
  void onTapCancel(TapCancelEvent event) {
    _isPressed = false;
    _background.paint.color = const Color(0xFF101420);
  }

  static void _noop() {}
}
