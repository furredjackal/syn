import 'package:flame/components.dart';

import 'base_button_component.dart';

/// Text-only button stub built on BaseButtonComponent.
class TextButtonComponent extends BaseButtonComponent {
  TextButtonComponent({
    required this.label,
    super.onTap,
    super.size,
    super.anchor,
    super.position,
  });

  final String label;
  TextComponent? _text;

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    _text = TextComponent(text: label, anchor: Anchor.center, position: size / 2);
    await add(_text!);
  }
}
