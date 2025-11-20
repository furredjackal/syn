import 'package:flame/components.dart';

import 'base_button_component.dart';

/// Text-only button stub built on BaseButtonComponent.
class TextButtonComponent extends BaseButtonComponent {
  TextButtonComponent({required this.label});

  final String label;
  TextComponent? _text;

  @override
  Future<void> onLoad() async {
    _text = TextComponent(text: label, anchor: Anchor.center);
    await add(_text!);
  }
}
