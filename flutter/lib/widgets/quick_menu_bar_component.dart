import 'package:flame/components.dart';
import 'package:flame/input.dart';
import 'package:flutter/material.dart';
import 'package:syn/flutter/lib/syn_game.dart';

class QuickMenuBarComponent extends PositionComponent with HasGameRef<SynGame> {
  @override
  Future<void> onLoad() async {
    final background = RectangleComponent(
      paint: Paint()..color = Colors.black.withOpacity(0.3),
      size: size,
    );
    add(background);

    double xOffset = size.x / 4;
    _addButton('MEMORY (M)', () => game.router.pushNamed('journal'), xOffset);
    xOffset += size.x / 4;
    _addButton('SAVE (Ctrl+S)', () {
      // TODO: Implement save
    }, xOffset);
    xOffset += size.x / 4;
    _addButton(
        'SETTINGS (ESC)', () => game.router.pushNamed('settings'), xOffset);
    xOffset += size.x / 4;
    _addButton('MENU', () => game.router.pushReplacementNamed('menu'), xOffset);
  }

  void _addButton(String label, VoidCallback onPressed, double x) {
    add(QuickMenuButton(
      label: label,
      onPressed: onPressed,
      position: Vector2(x, size.y / 2),
      anchor: Anchor.center,
    ));
  }
}

class QuickMenuButton extends PositionComponent with Tappable {
  final String label;
  final VoidCallback onPressed;

  QuickMenuButton({
    required this.label,
    required this.onPressed,
    Vector2? position,
    Anchor? anchor,
  }) : super(position: position, anchor: anchor);

  final _text = TextComponent();
  bool _isHovered = false;

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

  @override
  bool onTapDown(TapDownInfo info) {
    onPressed();
    return true;
  }

  // TODO: Add hover effect
}