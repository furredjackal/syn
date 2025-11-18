
import 'package:flame/components.dart';
import 'package:flame/events.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../syn_game.dart';

class CharacterCreationComponent extends PositionComponent
    with HasGameReference<SynGame>, KeyboardHandler {
  final List<String> archetypes = [
    'STORYTELLER',
    'ANALYST',
    'DREAMER',
    'CHALLENGER',
  ];

  final List<String> difficultyLevels = [
    'FORGIVING',
    'BALANCED',
    'HARSH',
  ];

  int _selected = 0;
  int _selectedDifficulty = 1; // BALANCED
  bool _sfwMode = true;
  String _playerName = '';
  String _nameInputText = '';
  bool _nameInputActive = false;

  late final _CreationBackground _background;
  late final PositionComponent _column;

  final List<_CreationOption> _options = [];
  late _NameInputField _nameInput;
  late _ContentModeToggle _contentModeToggle;
  late _DifficultyToggle _difficultyToggle;
  late _BeginLifeButton _beginButton;

  @override
  Future<void> onLoad() async {
    size = game.size;

    _background = _CreationBackground()..size = size;
    add(_background);

    _column = PositionComponent();
    add(_column);

    // Archetype options
    for (var i = 0; i < archetypes.length; i++) {
      final option = _CreationOption(
        label: archetypes[i],
        description: _describe(archetypes[i]),
        onSelected: () => _choose(i),
      );
      _column.add(option);
      _options.add(option);
    }

    // Name input
    _nameInput = _NameInputField(
      onTextChanged: (text) => _nameInputText = text,
      onSubmit: _handleBeginLife,
      onFocused: () => _nameInputActive = true,
    );
    add(_nameInput);

    // Content mode
    _contentModeToggle = _ContentModeToggle(
      value: _sfwMode,
      onChanged: (value) => _sfwMode = value,
    );
    add(_contentModeToggle);

    // Difficulty
    _difficultyToggle = _DifficultyToggle(
      selected: _selectedDifficulty,
      onSelected: (index) => _selectedDifficulty = index,
    );
    add(_difficultyToggle);

    // Begin button
    _beginButton = _BeginLifeButton(
      onPressed: _handleBeginLife,
    );
    add(_beginButton);

    _choose(0);
    _layout(); // initial layout
  }

  void _layout() {
    // keep background full screen
    _background.size = size;

    // overall scaling so it doesn't get tiny on 4K or huge on small windows
    const baseWidth = 1280.0;
    final scale = (size.x / baseWidth).clamp(0.85, 1.1);

    final leftMargin = size.x * 0.10;
    final topMargin = size.y * 0.22;

    final cardWidth = 520.0 * scale;
    final optionHeight = 70.0 * scale;
    final verticalGap = 14.0 * scale;

    _column
      ..position = Vector2(leftMargin, topMargin)
      ..size = Vector2(cardWidth, 0);

    // Lay out archetype options in a clean vertical stack
    double y = 0;
    for (final option in _options) {
      option
        ..size = Vector2(cardWidth, optionHeight)
        ..position = Vector2(0, y);
      y += optionHeight + verticalGap;
    }

    // Name field directly under the archetype list
    final nameTop = topMargin + y + 20.0 * scale;
    _nameInput
      ..size = Vector2(cardWidth, 60.0 * scale)
      ..position = Vector2(leftMargin, nameTop);

    // Content mode row
    final contentTop = nameTop + _nameInput.size.y + 18.0 * scale;
    _contentModeToggle
      ..size = Vector2(cardWidth, 70.0 * scale)
      ..position = Vector2(leftMargin, contentTop);

    // Difficulty row
    final difficultyTop = contentTop + _contentModeToggle.size.y + 12.0 * scale;
    _difficultyToggle
      ..size = Vector2(cardWidth, 70.0 * scale)
      ..position = Vector2(leftMargin, difficultyTop);

    // Big BEGIN LIFE button at the bottom of the stack
    final buttonTop = difficultyTop + _difficultyToggle.size.y + 24.0 * scale;
    _beginButton
      ..size = Vector2(300.0 * scale, 60.0 * scale)
      ..position = Vector2(leftMargin, buttonTop);
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

  Future<void> _handleBeginLife() async {
    if (_nameInputText.isEmpty) {
      // Show error feedback - could add an error message component here
      return;
    }

    _playerName = _nameInputText;
    final difficulty = difficultyLevels[_selectedDifficulty];

    // Pass character creation data to game state
    await game.startGameplayWithCharacter(
      name: _playerName,
      archetype: archetypes[_selected],
      sfwMode: _sfwMode,
      difficulty: difficulty,
    );
  }

  @override
  void onGameResize(Vector2 newSize) {
    super.onGameResize(newSize);
    size = newSize;
    _layout();
  }

  @override
  bool onKeyEvent(KeyEvent event, Set<LogicalKeyboardKey> keysPressed) {
    if (event is! KeyDownEvent) return false;

    // While the name input is focused, let *it* handle keys.
    if (_nameInputActive) {
      // Only special-case ESC to also clear our flag.
      if (event.logicalKey == LogicalKeyboardKey.escape) {
        _nameInputActive = false;
        // tell the field to visually unfocus too
        _nameInput.unfocus();
        return true;
      }

      // Do NOT forward to _nameInput here – Flame already will.
      // Also don't run archetype navigation / main menu shortcuts.
      return false;
    }

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
      _handleBeginLife();
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
        text: 'Select an archetype, name your character, and choose your path.',
        style: TextStyle(
          color: Color(0xFFB3B3B3),
          fontSize: 16,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: size.x * 0.6);
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
        ..color = isActive ? const Color(0xFFFFFFFF) : const Color(0x22000000),
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
          color: isActive ? const Color(0xFF1E1E1E) : const Color(0xFFCCCCCC),
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

class _NameInputField extends PositionComponent
    with TapCallbacks, KeyboardHandler {
  _NameInputField({
    required this.onTextChanged,
    required this.onSubmit,
    this.onFocused,
  }) : super(size: Vector2(520, 60));

  final Function(String) onTextChanged;
  final VoidCallback onSubmit;
  final VoidCallback? onFocused;
  String _text = '';
  bool _focused = false;

  @override
  void render(Canvas canvas) {
    final rect = RRect.fromRectAndRadius(
      Rect.fromLTWH(0, 0, size.x, size.y),
      const Radius.circular(8),
    );

    canvas.drawRRect(
      rect,
      Paint()
        ..color = _focused ? const Color(0x44FFFFFF) : const Color(0x22000000),
    );
    canvas.drawRRect(
      rect,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = _focused ? const Color(0xFF00D9FF) : const Color(0xFFFFFFFF),
    );

    final labelPainter = TextPainter(
      text: const TextSpan(
        text: 'NAME',
        style: TextStyle(
          color: Color(0xFFB3B3B3),
          fontSize: 12,
          fontWeight: FontWeight.bold,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    labelPainter.paint(canvas, const Offset(12, 6));

    final textPainter = TextPainter(
      text: TextSpan(
        text: _text.isEmpty && !_focused ? 'Enter your character name' : _text,
        style: TextStyle(
          color: _text.isEmpty && !_focused
              ? const Color(0xFF666666)
              : const Color(0xFFFFFFFF),
          fontSize: 18,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: size.x - 24);
    textPainter.paint(canvas, const Offset(12, 28));
  }

  @override
  void onTapUp(TapUpEvent event) {
    _focused = true;
    onFocused?.call();
  }

  void unfocus() {
    _focused = false;
  }

  @override
  bool onKeyEvent(KeyEvent event, Set<LogicalKeyboardKey> keysPressed) {
    if (!_focused) return false;
    if (event is! KeyDownEvent) return false;

    final key = event.logicalKey;

    // Handle Backspace
    if (key == LogicalKeyboardKey.backspace) {
      if (_text.isNotEmpty) {
        _text = _text.substring(0, _text.length - 1);
        onTextChanged(_text);
      }
      return true;
    }

    // Handle Enter/Return to submit
    if (key == LogicalKeyboardKey.enter) {
      onSubmit();
      return true;
    }

    // Handle Escape to unfocus (let parent handle the actual unfocus)
    if (key == LogicalKeyboardKey.escape) {
      _focused = false;
      return true;
    }

    // Handle regular character input
    if (event.character != null && event.character!.isNotEmpty) {
      final char = event.character!;
      // Only allow alphanumeric and common punctuation
      if (_isValidChar(char)) {
        if (_text.length < 50) {
          _text += char;
          onTextChanged(_text);
        }
        return true;
      }
    }

    return false;
  }

  bool _isValidChar(String char) {
    // Allow letters, numbers, spaces, and common punctuation
    return RegExp(r"[a-zA-Z0-9\s\-']").hasMatch(char) || char == '.';
  }
}

class _ContentModeToggle extends PositionComponent with TapCallbacks {
  _ContentModeToggle({
    required this.value,
    required this.onChanged,
  }) : super(size: Vector2(520, 70));

  final bool value;
  final Function(bool) onChanged;
  bool _sfwMode = true;

  @override
  Future<void> onLoad() async {
    _sfwMode = value;
  }

  @override
  void render(Canvas canvas) {
    // Title
    final titlePainter = TextPainter(
      text: const TextSpan(
        text: 'CONTENT MODE',
        style: TextStyle(
          color: Color(0xFFB3B3B3),
          fontSize: 14,
          fontWeight: FontWeight.bold,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    titlePainter.paint(canvas, const Offset(0, 0));

    // SFW Button
    _renderToggleButton(
      canvas,
      x: 0,
      y: 28,
      width: 250,
      label: 'SFW',
      isSelected: _sfwMode,
    );

    // NSFW Button
    _renderToggleButton(
      canvas,
      x: 270,
      y: 28,
      width: 250,
      label: 'NSFW',
      isSelected: !_sfwMode,
    );
  }

  void _renderToggleButton(
    Canvas canvas, {
    required double x,
    required double y,
    required double width,
    required String label,
    required bool isSelected,
  }) {
    final rect = RRect.fromRectAndRadius(
      Rect.fromLTWH(x, y, width, 36),
      const Radius.circular(6),
    );

    canvas.drawRRect(
      rect,
      Paint()
        ..color = isSelected
            ? const Color(0xFF00D9FF).withOpacity(0.2)
            : const Color(0x22000000),
    );
    canvas.drawRRect(
      rect,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color =
            isSelected ? const Color(0xFF00D9FF) : const Color(0xFFFFFFFF),
    );

    final textPainter = TextPainter(
      text: TextSpan(
        text: label,
        style: TextStyle(
          color: isSelected ? const Color(0xFF00D9FF) : const Color(0xFFFFFFFF),
          fontSize: 14,
          fontWeight: FontWeight.bold,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    textPainter.paint(
      canvas,
      Offset(x + (width - textPainter.width) / 2, y + 8),
    );
  }

  @override
  void onTapUp(TapUpEvent event) {
    if (event.localPosition.x < 250) {
      _sfwMode = true;
    } else {
      _sfwMode = false;
    }
    onChanged(_sfwMode);
  }
}

class _DifficultyToggle extends PositionComponent with TapCallbacks {
  _DifficultyToggle({
    required this.selected,
    required this.onSelected,
  }) : super(size: Vector2(520, 70));

  int selected;
  final Function(int) onSelected;
  final difficulties = ['FORGIVING', 'BALANCED', 'HARSH'];

  @override
  void render(Canvas canvas) {
    // Title
    final titlePainter = TextPainter(
      text: const TextSpan(
        text: 'DIFFICULTY',
        style: TextStyle(
          color: Color(0xFFB3B3B3),
          fontSize: 14,
          fontWeight: FontWeight.bold,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    titlePainter.paint(canvas, const Offset(0, 0));

    final buttonWidth = 160.0;
    for (var i = 0; i < difficulties.length; i++) {
      final x = i * (buttonWidth + 8);
      _renderToggleButton(
        canvas,
        x: x,
        y: 28,
        width: buttonWidth,
        label: difficulties[i],
        isSelected: selected == i,
      );
    }
  }

  void _renderToggleButton(
    Canvas canvas, {
    required double x,
    required double y,
    required double width,
    required String label,
    required bool isSelected,
  }) {
    final rect = RRect.fromRectAndRadius(
      Rect.fromLTWH(x, y, width, 36),
      const Radius.circular(6),
    );

    canvas.drawRRect(
      rect,
      Paint()
        ..color = isSelected
            ? const Color(0xFF00D9FF).withOpacity(0.2)
            : const Color(0x22000000),
    );
    canvas.drawRRect(
      rect,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color =
            isSelected ? const Color(0xFF00D9FF) : const Color(0xFFFFFFFF),
    );

    final textPainter = TextPainter(
      text: TextSpan(
        text: label,
        style: TextStyle(
          color: isSelected ? const Color(0xFF00D9FF) : const Color(0xFFFFFFFF),
          fontSize: 12,
          fontWeight: FontWeight.bold,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    textPainter.paint(
      canvas,
      Offset(x + (width - textPainter.width) / 2, y + 8),
    );
  }

  @override
  void onTapUp(TapUpEvent event) {
    if (event.localPosition.x < 168) {
      onSelected(0);
      selected = 0;
    } else if (event.localPosition.x < 336) {
      onSelected(1);
      selected = 1;
    } else {
      onSelected(2);
      selected = 2;
    }
  }
}

class _BeginLifeButton extends PositionComponent with TapCallbacks {
  _BeginLifeButton({required this.onPressed}) : super(size: Vector2(300, 60));

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
