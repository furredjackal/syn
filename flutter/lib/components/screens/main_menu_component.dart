import 'dart:ui';
import 'package:flame/components.dart';
import 'package:flame/events.dart';
import 'package:flutter/material.dart' show Colors, TextStyle, FontWeight, TextPainter, TextSpan;
import 'package:flutter/services.dart';
import '../../syn_game.dart';
import '../ui/system/background_layer_component.dart';

/// Persona/Synthwave Styled Main Menu
///
/// Layout:
/// - Large angled title on left
/// - Vertical slash-buttons (STORY, TUTORIAL, SETTINGS...)
/// - Right neon sidebar with game tips (Persona-style sidebar panel)
class MainMenuComponent extends PositionComponent
    with HasGameReference<SynGame>, KeyboardHandler {
  MainMenuComponent();

  final List<_MenuOptionButton> _buttons = [];
  int _selectedIndex = 0;

  // Palette
  static const CYAN = Color(0xFF00E6FF);
  static const PURPLE = Color(0xFF9A27FF);
  static const BG_SOFT = Color(0xCC0A0A0F);
  static const SOFT_WHITE = Color(0xFFF6F6F6);

  // Typography
  late final TextPaint _titleText = TextPaint(
    style: const TextStyle(
      color: Colors.white,
      fontSize: 56,
      fontWeight: FontWeight.w900,
      letterSpacing: 6,
    ),
  );

  @override
  Future<void> onLoad() async {
    size = game.size;

    // Background
    add(BackgroundLayerComponent()..size = size);

    // Title
    add(
      TextComponent(
        text: 'SYN',
        textRenderer: _titleText,
        position: Vector2(size.x * 0.12, size.y * 0.10),
      ),
    );

    // Menu column
    final column = PositionComponent()
      ..position = Vector2(size.x * 0.12, size.y * 0.25)
      ..size = Vector2(500, 400);

    add(column);

    final labels = [
      _MenuAction('STORY', game.showCharacterCreation),
      _MenuAction('TUTORIAL',
          () => game.showComingSoon('Tutorial coming soon')),
      _MenuAction('SETTINGS', game.showSettings),
      _MenuAction('DATA LOAD',
          () => game.showComingSoon('Loadouts coming soon')),
      _MenuAction('DATA SAVE',
          () => game.showComingSoon('Saving coming soon')),
      _MenuAction('RETURN TO TITLE', game.showSplash),
    ];

    for (var i = 0; i < labels.length; i++) {
      final button = _MenuOptionButton(
        label: labels[i].label,
        onSelected: labels[i].callback,
      )
        ..position = Vector2(0, i * 90);

      column.add(button);
      _buttons.add(button);
    }

    add(_MenuSidebar());

    // Highlight first button
    _activateButton(0);
  }

  void _activateButton(int index) {
    if (_buttons.isEmpty) return;
    _selectedIndex = (index + _buttons.length) % _buttons.length;
    for (var i = 0; i < _buttons.length; i++) {
      _buttons[i].isActive = (i == _selectedIndex);
    }
  }

  void _triggerSelection() {
    if (_buttons.isEmpty) return;
    _buttons[_selectedIndex].trigger();
  }

  @override
  bool onKeyEvent(KeyEvent event, Set<LogicalKeyboardKey> keysPressed) {
    if (event is! KeyDownEvent) return false;

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

/// Sidebar panel with neon border
class _MenuSidebar extends PositionComponent with HasGameReference<SynGame> {
  static const CYAN = Color(0xFF00E6FF);
  static const BG = Color(0xFF0E0E17);

  @override
  Future<void> onLoad() async {
    size = game.size;
    position = Vector2.zero();
  }

  @override
  void render(Canvas canvas) {
    final rect = Rect.fromLTWH(size.x * 0.55, size.y * 0.25, size.x * 0.36, size.y * 0.52);

    // Panel background
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(20)),
      Paint()..color = BG.withValues(alpha: 0.92),
    );

    // Neon border
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(20)),
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 3
        ..color = CYAN,
    );

    // Text
    final painter = TextPainter(
      text: const TextSpan(
        text:
            'A menu designed with bold typography,\n'
            'angled geometry, and neon synthwave accents.\n\n'
            'Use ↑/↓ to navigate, Enter to select.\n'
            'Or click/tap a command.',
        style: TextStyle(
          color: Color(0xFFEFEFEF),
          fontSize: 18,
          height: 1.45,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: rect.width - 40);

    painter.paint(canvas, Offset(rect.left + 20, rect.top + 24));
  }
}

/// Slash-styled option button
class _MenuOptionButton extends PositionComponent with TapCallbacks {
  _MenuOptionButton({
    required this.label,
    required this.onSelected,
  }) : super(size: Vector2(380, 70));

  final String label;
  final VoidCallback onSelected;

  bool isActive = false;
  bool isHovered = false;

  static const CYAN = Color(0xFF00E6FF);
  static const PURPLE = Color(0xFF9A27FF);

  late TextComponent _text;

  @override
  Future<void> onLoad() async {
    _text = TextComponent(
      text: label,
      anchor: Anchor.centerLeft,
      position: Vector2(80, size.y / 2),
      textRenderer: _inactiveStyle,
    );
    add(_text);
  }

  TextPaint get _activeStyle => TextPaint(
        style: const TextStyle(
          fontSize: 30,
          fontWeight: FontWeight.w900,
          letterSpacing: 3,
          color: Colors.black,
        ),
      );

  TextPaint get _inactiveStyle => TextPaint(
        style: const TextStyle(
          fontSize: 28,
          fontWeight: FontWeight.w900,
          letterSpacing: 3,
          color: Colors.white,
        ),
      );

  @override
  void render(Canvas canvas) {
    // Slash shape
    final path = Path()
      ..moveTo(0, 0)
      ..lineTo(size.x - 50, 0)
      ..lineTo(size.x, size.y)
      ..lineTo(40, size.y)
      ..close();

    // Fill
    final paint = Paint()
      ..style = PaintingStyle.fill
      ..shader = (isActive)
          ? Gradient.linear(
              const Offset(0, 0),
              Offset(size.x, size.y),
              [CYAN, PURPLE],
            )
          : null
      ..color = isActive ? Colors.white : const Color(0xFF080808);

    canvas.drawPath(path, paint);

    // Border
    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = isActive ? 4 : 2
        ..color = isActive ? Colors.black : const Color(0xFF222222),
    );

    // Left neon arrow active indicator
    if (isActive) {
      final arrow = Path()
        ..moveTo(-30, size.y / 2 - 12)
        ..lineTo(-8, size.y / 2)
        ..lineTo(-30, size.y / 2 + 12)
        ..close();

      canvas.drawPath(arrow, Paint()..color = CYAN);
    }
  }

  @override
  void update(double dt) {
    super.update(dt);
    _text.textRenderer = isActive ? _activeStyle : _inactiveStyle;
  }

  @override
  void onTapUp(TapUpEvent event) => trigger();

  void trigger() => onSelected();
}

class _MenuAction {
  _MenuAction(this.label, this.callback);
  final String label;
  final VoidCallback callback;
}

class MainMenuScreen extends MainMenuComponent {}
