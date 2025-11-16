import 'package:flame/components.dart';
import 'package:flutter/material.dart';
import '../syn_game.dart';

class QuickMenuBarComponent extends PositionComponent with HasGameRef<SynGame> {
  @override
  Future<void> onLoad() async {
    final background = RectangleComponent(
      paint: Paint()..color = Colors.black.withOpacity(0.3),
      size: size,
    );
    add(background);

    // TODO: Implement quick menu buttons with proper navigation
    // For now, just show placeholder
  }
}

// TODO: Implement QuickMenuButton when Flutter routing is integrated
/* 
class QuickMenuButton extends PositionComponent {
  final String label;
  final VoidCallback onPressed;

  QuickMenuButton({
    required this.label,
    required this.onPressed,
    Vector2? position,
    Anchor? anchor,
  }) : super(position: position, anchor: anchor);

  final _text = TextComponent();

  @override
  Future<void> onLoad() async {
    _text.text = label;
    _text.textRenderer = TextPaint(
      style: TextStyle(
        color: Colors.white.withOpacity(0.7),
      ),
    );
    _text.anchor = Anchor.center;
    add(_text);
  }
}
*/