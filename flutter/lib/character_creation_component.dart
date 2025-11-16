import 'dart:ui';

import 'package:flame/components.dart';
import 'package:flame/events.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'syn_game.dart';

class CharacterCreationComponent extends PositionComponent
    with HasGameReference<SynGame>, KeyboardHandler {
  final List<String> archetypes = [
    'STORYTELLER',
    'ANALYST',
    'DREAMER',
    'CHALLENGER',
  ];

  int _selected = 0;
  final List<_CreationOption> _options = [];

  @override
  Future<void> onLoad() async {
    size = game.size;
    add(_CreationBackground()..size = size);
    _layoutOptions();
  }

  void _layoutOptions() {
    final column = PositionComponent()
      ..position = Vector2(size.x * 0.1, size.y * 0.25);
    add(column);

    for (var i = 0; i < archetypes.length; i++) {
      final option = _CreationOption(
        label: archetypes[i],
        description: _describe(archetypes[i]),
        onSelected: () => _choose(i),
      )..position = Vector2(0, i * 90);
      column.add(option);
      _options.add(option);
    }

    add(
      _StartButton(
        onPressed: () => game.startGameplay(),
      )..position = Vector2(size.x * 0.1, size.y * 0.75),
    );

    _choose(0);
  }

  String _describe(String label) {
    switch (label) {
      case 'STORYTELLER':
        return 'High empathy, balanced stats, thrives in social choices.';
      case 'ANALYST':
        return 'Intellect focused path with deliberate choices.';
      case 'DREAMER':
        return 'Unpredictable, creative, volatile narrative beats.';
      case 'CHALLENGER':
      default:
        return 'Bold, competitive, seeks risky opportunities.';
    }
  }

  void _choose(int index) {
    _selected = (index + archetypes.length) % archetypes.length;
    for (var i = 0; i < _options.length; i++) {
      _options[i].isActive = i == _selected;
    }
  }

  @override
  void onGameResize(Vector2 size) {
    super.onGameResize(size);
    this.size = size;
  }

  @override
  bool onKeyEvent(KeyEvent event, Set<LogicalKeyboardKey> keysPressed) {
    if (event is! KeyDownEvent) return false;
    if (event.logicalKey == LogicalKeyboardKey.arrowDown) {
      _choose(_selected + 1);
      return true;
    }
    if (event.logicalKey == LogicalKeyboardKey.arrowUp) {
      _choose(_selected - 1);
      return true;
    }
    if (event.logicalKey == LogicalKeyboardKey.enter ||
        event.logicalKey == LogicalKeyboardKey.space) {
      game.startGameplay();
      return true;
    }
    if (event.logicalKey == LogicalKeyboardKey.escape) {
      game.showMainMenu();
      return true;
    }
    return false;
  }
}

class _CreationBackground extends PositionComponent {
  @override
  void render(Canvas canvas) {
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    canvas.drawRect(rect, Paint()..color = const Color(0xFF050505));

    final accent = Paint()
      ..shader = const LinearGradient(
        colors: [
          Color(0xFF111111),
          Color(0xFF1F1F1F),
        ],
        begin: Alignment(0, -1),
        end: Alignment(0.3, 1),
      ).createShader(rect);
    canvas.drawRect(rect, accent);

    final title = TextPainter(
      text: const TextSpan(
        text: 'BEGIN YOUR LIFE',
        style: TextStyle(
          color: Color(0xFFFFFFFF),
          fontSize: 40,
          fontWeight: FontWeight.w900,
          letterSpacing: 2,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    title.paint(canvas, Offset(size.x * 0.1, size.y * 0.12));

    final subtitle = TextPainter(
      text: const TextSpan(
        text: 'Select an archetype to seed your first narrative loop.',
        style: TextStyle(
          color: Color(0xFFB3B3B3),
          fontSize: 18,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: size.x * 0.5);
    subtitle.paint(canvas, Offset(size.x * 0.1, size.y * 0.18));
  }
}

class _CreationOption extends PositionComponent with TapCallbacks {
  _CreationOption({
    required this.label,
    required this.description,
    required this.onSelected,
  }) : super(size: Vector2(520, 70));

  final String label;
  final String description;
  final VoidCallback onSelected;
  bool isActive = false;

  @override
  void render(Canvas canvas) {
    final rect = RRect.fromRectAndRadius(
      Rect.fromLTWH(0, 0, size.x, size.y),
      const Radius.circular(12),
    );
    canvas.drawRRect(
      rect,
      Paint()
        ..color =
            isActive ? const Color(0xFFFFFFFF) : const Color(0x22000000),
    );
    canvas.drawRRect(
      rect,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = const Color(0xFFFFFFFF),
    );

    final title = TextPainter(
      text: TextSpan(
        text: label,
        style: TextStyle(
          color: isActive ? const Color(0xFF000000) : const Color(0xFFFFFFFF),
          fontSize: 22,
          fontWeight: FontWeight.w800,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: size.x - 20);
    title.paint(canvas, const Offset(16, 8));

    final descPainter = TextPainter(
      text: TextSpan(
        text: description,
        style: TextStyle(
          color:
              isActive ? const Color(0xFF1E1E1E) : const Color(0xFFCCCCCC),
          fontSize: 14,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: size.x - 20);
    descPainter.paint(canvas, const Offset(16, 36));
  }

  @override
  void onTapUp(TapUpEvent event) {
    onSelected();
  }
}

class _StartButton extends PositionComponent with TapCallbacks {
  _StartButton({required this.onPressed})
      : super(size: Vector2(300, 60));

  final VoidCallback onPressed;

  @override
  void render(Canvas canvas) {
    final path = Path()
      ..moveTo(0, 0)
      ..lineTo(size.x - 30, 0)
      ..lineTo(size.x, size.y)
      ..lineTo(30, size.y)
      ..close();
    canvas.drawPath(path, Paint()..color = const Color(0xFF00D9FF));
    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 3
        ..color = const Color(0xFF000000),
    );
    final painter = TextPainter(
      text: const TextSpan(
        text: 'BEGIN LIFE',
        style: TextStyle(
          color: Color(0xFF000000),
          fontSize: 22,
          fontWeight: FontWeight.w900,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    painter.paint(
      canvas,
      Offset(
        (size.x - painter.width) / 2,
        (size.y - painter.height) / 2,
      ),
    );
  }

  @override
  void onTapUp(TapUpEvent event) {
    onPressed();
  }
}
