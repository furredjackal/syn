import 'package:flame/components.dart';
import 'package:flame/effects.dart';
import 'package:flame/events.dart';
import 'package:flutter/material.dart';

import 'base_button_component.dart';
import '../syn_theme.dart';

/// Icon-only button stub with hover support.
class IconButtonComponent extends BaseButtonComponent with HoverCallbacks {
  IconButtonComponent({
    super.size,
    this.icon,
    super.onTap,
    this.onHoverChanged,
    this.onPressChanged,
    double? hoverScale,
    double? hoverDuration,
    Color? baseColor,
    Color? hoverColor,
    Color? pressedColor,
    Color? strokeColor,
    double strokeWidth = 0,
  })  : _hoverScale = hoverScale ?? SynTopBar.actionHoverScale,
        _hoverDuration = hoverDuration ?? SynTopBar.actionHoverDuration,
        super(
          baseColor: baseColor ?? const Color(0x00000000),
          hoverColor: hoverColor ?? const Color(0x00000000),
          pressedColor: pressedColor ?? const Color(0x00000000),
          strokeColor: strokeColor,
          strokeWidth: strokeWidth,
        );

  final SpriteComponent? icon;
  bool isHovered = false;
  Effect? _hoverEffect;
  final double _hoverScale;
  final double _hoverDuration;
  final void Function(bool)? onHoverChanged;
  final void Function(bool)? onPressChanged;

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    if (icon != null) {
      await add(icon!);
    }
  }

  @override
  void onHoverEnter() {
    isHovered = true;
    setHovered(true);
    onHoverChanged?.call(true);
    _applyHoverScale(_hoverScale);
  }

  @override
  void onHoverExit() {
    isHovered = false;
    setHovered(false);
    onHoverChanged?.call(false);
    _applyHoverScale(1.0);
  }

  @override
  void onTapDown(TapDownEvent event) {
    super.onTapDown(event);
    onPressChanged?.call(true);
  }

  @override
  void onTapUp(TapUpEvent event) {
    super.onTapUp(event);
    onPressChanged?.call(false);
  }

  @override
  void onTapCancel(TapCancelEvent event) {
    super.onTapCancel(event);
    onPressChanged?.call(false);
  }

  void _applyHoverScale(double target) {
    _hoverEffect?.removeFromParent();
    _hoverEffect = ScaleEffect.to(
      Vector2.all(target),
      EffectController(
        duration: _hoverDuration,
      ),
    );
    add(_hoverEffect!);
  }
}
