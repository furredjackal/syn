
import 'package:flame/components.dart';
import 'package:flame/events.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../../syn_game.dart';
import '../ui/system/background_layer_component.dart';

class MainMenuComponent extends PositionComponent
    with HasGameReference<SynGame>, KeyboardHandler {
  MainMenuComponent();

  final List<_MenuOptionButton> _buttons = [];
  int _selectedIndex = 0;

  @override
  Future<void> onLoad() async {
    size = game.size;
    add(BackgroundLayerComponent()..size = size);

    final labels = [
      _MenuAction('STORY', game.showCharacterCreation),
      _MenuAction('TUTORIAL', () => game.showComingSoon('Tutorial coming soon')),
      _MenuAction('SETTINGS', game.showSettings),
      _MenuAction('DATA LOAD', () => game.showComingSoon('Loadouts coming soon')),
      _MenuAction('DATA SAVE', () => game.showComingSoon('Saves coming soon')),
      _MenuAction('RETURN TO TITLE', game.showSplash),
    ];

    final column = PositionComponent()
      ..position = Vector2(size.x * 0.12, size.y * 0.25);
    add(column);

    for (var i = 0; i < labels.length; i++) {
      final button = _MenuOptionButton(
        label: labels[i].label,
        onSelected: labels[i].callback,
      )..position = Vector2(0, i * 78);
      column.add(button);
      _buttons.add(button);
    }

    add(_MenuSidebar());

    _activateButton(0);
  }

  void _activateButton(int index) {
    if (_buttons.isEmpty) return;
    _selectedIndex = (index + _buttons.length) % _buttons.length;
    for (var i = 0; i < _buttons.length; i++) {
      _buttons[i].isActive = i == _selectedIndex;
    }
  }

  void _triggerSelection() {
    if (_buttons.isEmpty) return;
    _buttons[_selectedIndex].trigger();
  }

  @override
  void onGameResize(Vector2 size) {
    super.onGameResize(size);
    this.size = size;
  }

  @override
  bool onKeyEvent(KeyEvent event, Set<LogicalKeyboardKey> keysPressed) {
    if (event is! KeyDownEvent) {
      return false;
    }
    if (event.logicalKey == LogicalKeyboardKey.arrowDown) {
      _activateButton(_selectedIndex + 1);
      return true;
    }
    if (event.logicalKey == LogicalKeyboardKey.arrowUp) {
      _activateButton(_selectedIndex - 1);
      return true;
    }
    if (event.logicalKey == LogicalKeyboardKey.enter ||
        event.logicalKey == LogicalKeyboardKey.space) {
      _triggerSelection();
      return true;
    }
    return false;
  }
}

class _MenuSidebar extends PositionComponent with HasGameReference<SynGame> {
  @override
  Future<void> onLoad() async {
    size = game.size;
    position = Vector2.zero();
  }

  @override
  void render(Canvas canvas) {
    final rect = Rect.fromLTWH(size.x * 0.55, size.y * 0.25, size.x * 0.4, size.y * 0.6);
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(24)),
      Paint()..color = const Color(0xFF0E0E0E),
    );
    final painter = TextPainter(
      text: const TextSpan(
        text:
            'An interface focusing on bold typography and layered shapes.\n\n'
            'Navigate with ↑/↓ and confirm with Enter, or click/tap a command.',
        style: TextStyle(
          color: Color(0xFFEEEEEE),
          fontSize: 18,
          height: 1.4,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: rect.width - 40);
    painter.paint(
      canvas,
      Offset(rect.left + 20, rect.top + 30),
    );
  }
}

class _MenuOptionButton extends PositionComponent with TapCallbacks {
  _MenuOptionButton({
    required this.label,
    required this.onSelected,
  }) : super(size: Vector2(360, 64));

  final String label;
  final VoidCallback onSelected;
  bool isActive = false;

  @override
  Future<void> onLoad() async {
    add(
      TextComponent(
        text: label,
        anchor: Anchor.centerLeft,
        position: Vector2(70, size.y / 2),
        textRenderer: TextPaint(
          style: const TextStyle(
            fontSize: 28,
            fontWeight: FontWeight.w900,
          ),
        ),
      ),
    );
  }

  @override
  void render(Canvas canvas) {
    final path = Path()
      ..moveTo(0, 0)
      ..lineTo(size.x - 40, 0)
      ..lineTo(size.x, size.y)
      ..lineTo(40, size.y)
      ..close();

    final fillColor = isActive ? const Color(0xFFFFFFFF) : const Color(0xFF060606);
    canvas.drawPath(path, Paint()..color = fillColor);
    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 4
        ..color = const Color(0xFF000000),
    );

    if (isActive) {
      final arrow = Path()
        ..moveTo(-26, size.y / 2 - 12)
        ..lineTo(-6, size.y / 2)
        ..lineTo(-26, size.y / 2 + 12)
        ..close();
      canvas.drawPath(arrow, Paint()..color = const Color(0xFF00D9FF));
    }
  }

  @override
  void update(double dt) {
    super.update(dt);
    final text = children.whereType<TextComponent>().first;
    text.textRenderer = TextPaint(
      style: TextStyle(
        fontSize: 28,
        fontWeight: FontWeight.w900,
        color: isActive ? const Color(0xFF000000) : const Color(0xFFFFFFFF),
      ),
    );
  }

  void trigger() {
    onSelected();
  }

  @override
  void onTapUp(TapUpEvent event) {
    trigger();
  }
}

class _MenuAction {
  _MenuAction(this.label, this.callback);
  final String label;
  final VoidCallback callback;
}
