import 'package:flame/components.dart';
import 'package:flame/events.dart';

import 'base_button_component.dart';

/// Icon-only button stub with hover support.
class IconButtonComponent extends BaseButtonComponent with HoverCallbacks {
  IconButtonComponent({super.size, this.icon, super.onTap});

  final SpriteComponent? icon;
  bool isHovered = false;

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    if (icon != null) {
      await add(icon!);
    }
  }

  @override
  void onHoverEnter() => isHovered = true;

  @override
  void onHoverExit() => isHovered = false;
}
