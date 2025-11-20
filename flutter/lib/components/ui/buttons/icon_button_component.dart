import 'package:flame/components.dart';
import 'package:flame/events.dart';

import 'base_button_component.dart';

/// Icon-only button stub with hover support.
class IconButtonComponent extends BaseButtonComponent with HoverCallbacks {
  IconButtonComponent({this.icon});

  final SpriteComponent? icon;
  bool isHovered = false;

  @override
  Future<void> onLoad() async {
    if (icon != null) {
      await add(icon!);
    }
  }

  @override
  void onHoverEnter() => isHovered = true;

  @override
  void onHoverExit() => isHovered = false;
}
