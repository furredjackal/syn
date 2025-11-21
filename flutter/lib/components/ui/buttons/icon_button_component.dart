import 'package:flame/components.dart';
import 'package:flame/effects.dart';
import 'package:flame/events.dart';
import 'package:flutter/material.dart';

import 'base_button_component.dart';
import '../syn_theme.dart';

/// Icon-only button that supports both Sprites and Material Icons.
class IconButtonComponent extends BaseButtonComponent with HoverCallbacks {
  IconButtonComponent({
    super.size,
    this.icon,
    this.materialIcon, // <--- New: Support for Icons.close, Icons.settings, etc.
    this.iconColor,
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
  final IconData? materialIcon;
  final Color? iconColor;
  
  bool isHovered = false;
  Effect? _hoverEffect;
  final double _hoverScale;
  final double _hoverDuration;
  final void Function(bool)? onHoverChanged;
  final void Function(bool)? onPressChanged;

  @override
  Future<void> onLoad() async {
    await super.onLoad();

    // Priority 1: Sprite Icon
    if (icon != null) {
      add(icon!);
    } 
    // Priority 2: Material Icon (Rendered as Text)
    else if (materialIcon != null) {
      add(TextComponent(
        text: String.fromCharCode(materialIcon!.codePoint),
        textRenderer: TextPaint(
          style: TextStyle(
            color: iconColor ?? SynColors.primaryCyan,
            fontFamily: materialIcon!.fontFamily,
            package: materialIcon!.fontPackage,
            fontSize: size.y * 0.65, // Auto-scale to 65% of button height
          ),
        ),
        anchor: Anchor.center,
        position: size / 2,
      ));
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
      EffectController(duration: _hoverDuration),
    );
    add(_hoverEffect!);
  }
}