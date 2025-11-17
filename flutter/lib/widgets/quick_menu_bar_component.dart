
import 'package:flame/components.dart';
import 'package:flame/events.dart';
import 'package:flutter/material.dart';
import '../syn_game.dart';

class QuickMenuBarComponent extends PositionComponent
    with HasGameReference<SynGame> {
  @override
  Future<void> onLoad() async {
    final background = RectangleComponent(
      paint: Paint()..color = const Color(0x4D000000),
      size: size,
    );
    add(background);

    final items = [
      _QuickMenuConfig(
        label: 'MAIN MENU',
        onPressed: () => game.returnToTitle(),
      ),
      _QuickMenuConfig(
        label: 'PAUSE',
        onPressed: () => game.showPauseOverlay(),
      ),
      _QuickMenuConfig(
        label: 'JOURNAL',
        onPressed: () => game.showComingSoon('Journal coming soon'),
      ),
      _QuickMenuConfig(
        label: 'SAVE/LOAD',
        onPressed: () => game.showComingSoon('Save/Load coming soon'),
      ),
      _QuickMenuConfig(
        label: 'SETTINGS',
        onPressed: () => game.showSettings(),
      ),
    ];

    const horizontalPadding = 16.0;
    const spacing = 12.0;
    final totalSpacing = spacing * (items.length - 1);
    final availableWidth = size.x - (horizontalPadding * 2) - totalSpacing;
    final buttonWidth = availableWidth / items.length;
    final buttonHeight = size.y - 24;
    final startY = (size.y - buttonHeight) / 2;
    double currentX = horizontalPadding;

    for (final item in items) {
      final button = _QuickMenuButton(
        label: item.label,
        onPressed: item.onPressed,
        size: Vector2(buttonWidth, buttonHeight),
      )..position = Vector2(currentX, startY);
      add(button);
      currentX += buttonWidth + spacing;
    }
  }
}

class _QuickMenuConfig {
  final String label;
  final void Function() onPressed;

  _QuickMenuConfig({
    required this.label,
    required this.onPressed,
  });
}

class _QuickMenuButton extends PositionComponent with TapCallbacks {
  final String label;
  final void Function() onPressed;
  bool _isPressed = false;
  late RectangleComponent _background;
  late RectangleComponent _border;
  late TextComponent _text;

  _QuickMenuButton({
    required this.label,
    required this.onPressed,
    Vector2? size,
  }) : super(size: size ?? Vector2(140, 48));

  @override
  Future<void> onLoad() async {
    _background = RectangleComponent(
      size: size,
      paint: Paint()..color = const Color(0x14FFFFFF),
    );
    add(_background);

    _border = RectangleComponent(
      size: size,
      paint: Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = const Color(0x4DFFFFFF),
    );
    add(_border);

    _text = TextComponent(
      text: label,
      anchor: Anchor.center,
      position: size / 2,
      textRenderer: TextPaint(
        style: const TextStyle(
          color: Color(0xFFFFFFFF),
          fontSize: 12,
          letterSpacing: 1.2,
        ),
      ),
    );
    add(_text);
    _updateVisualState();
  }

  void _updateVisualState() {
    final backgroundOpacity = _isPressed ? 0.2 : 0.05;
    final borderOpacity = _isPressed ? 0.8 : 0.3;
    _background.paint =
        Paint()..color = Color.fromRGBO(255, 255, 255, backgroundOpacity);
    _border.paint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2
      ..color = Color.fromRGBO(255, 255, 255, borderOpacity);
  }

  @override
  void onTapDown(TapDownEvent event) {
    _isPressed = true;
    _updateVisualState();
  }

  @override
  void onTapUp(TapUpEvent event) {
    _isPressed = false;
    _updateVisualState();
    onPressed();
  }

  @override
  void onTapCancel(TapCancelEvent event) {
    _isPressed = false;
    _updateVisualState();
  }
}
