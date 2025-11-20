import 'package:flame/components.dart';
import 'package:flame/events.dart';
import 'package:flame/input.dart';
import 'package:flutter/material.dart';

import '../../../syn_game.dart';

/// Base button using a NinePatchComponent for its background.
class BaseButtonComponent extends PositionComponent
    with HasGameReference<SynGame>, TapCallbacks {
  BaseButtonComponent({
    required this.onTap,
    super.position,
    super.size,
    super.anchor,
  });

  final VoidCallback onTap;

  late final NinePatchComponent _background;
  bool _isPressed = false;

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    final image = await game.images.load('ui/buttons/button.png');
    _background = NinePatchComponent(
      image: image,
      destSize: size,
      // The center 1x1 pixel of the 16x16 image is the stretchable area.
      centerSlice: const Rect.fromLTWH(7, 7, 2, 2),
    );
    add(_background);
  }

  @override
  void onTapDown(TapDownEvent event) {
    _isPressed = true;
    _background.tint(const Color(0xAAFFFFFF)); // Lighten when pressed
  }

  @override
  void onTapUp(TapUpEvent event) {
    if (_isPressed) {
      onTap();
      _isPressed = false;
      _background.tint(const Color(0xFFFFFFFF)); // Reset tint
    }
  }

  @override
  void onTapCancel(TapCancelEvent event) {
    _isPressed = false;
    _background.tint(const Color(0xFFFFFFFF)); // Reset tint
  }
}
