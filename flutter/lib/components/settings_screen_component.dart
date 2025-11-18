
import 'package:flame/components.dart';
import 'package:flame/events.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../models/game_state.dart';
import '../syn_game.dart';
import '../ui/background.dart';

class SettingsScreenComponent extends PositionComponent
    with HasGameReference<SynGame>, KeyboardHandler {
  late final GameState _gameState;
  late final Background _background;
  PositionComponent? _content;
  _InfoPanel? _infoPanel;
  _BackButton? _backButton;
  double _contentWidth = 0;
  double _scaleY = 1;

  @override
  Future<void> onLoad() async {
    size = game.size;
    _gameState = game.gameState;

    _background = Background()..size = size;
    add(_background);

    _content = PositionComponent();
    add(_content!);
    _infoPanel = _InfoPanel(
      width: size.x * 0.28,
      text:
          'Options board.\n\nUse the mouse or ↑/↓ and Enter to toggle controls.\nPress ESC to return to the previous screen.',
    );
    add(_infoPanel!);
    _backButton = _BackButton(
      label: 'BACK TO MENU',
      onPressed: () => game.closeSettings(),
    );
    add(_backButton!);

    final title = TextComponent(
      text: 'SETTINGS',
      textRenderer: TextPaint(
        style: const TextStyle(
          color: Color(0xFFFFFFFF),
          fontSize: 48,
          fontWeight: FontWeight.w900,
          letterSpacing: 4,
        ),
      ),
    );
    add(title);
    final subtitle = TextComponent(
      text:
          'Control the experience. Options tuned to our neon noir.',
      textRenderer: TextPaint(
        style: const TextStyle(
          color: Color(0xFFB7B7B7),
          fontSize: 16,
          letterSpacing: 1.2,
        ),
      ),
    );
    add(subtitle);
    _titleLabel = title;
    _subtitleLabel = subtitle;

    _layout();
  }

  late final TextComponent _titleLabel;
  late final TextComponent _subtitleLabel;

  void _layout() {
    final baseHeight = 900.0;
    final scale = (size.y / baseHeight).clamp(0.7, 1.0);
    _scaleY = scale;
    final horizontalMargin = size.x * 0.08 * scale;
    final maxWidth = size.x - horizontalMargin * 2;
    _contentWidth = maxWidth.clamp(480.0 * scale, 920.0 * scale);

    _titleLabel.position = Vector2(horizontalMargin, size.y * 0.08);
    _subtitleLabel.position = Vector2(horizontalMargin, size.y * 0.13);

    final contentTop = size.y * 0.22;
    final content = _content!..position = Vector2(horizontalMargin, contentTop);
    content.removeAll(content.children.toList());
    content.size = Vector2(_contentWidth, size.y * 0.58);

    double y = 0;
    double lastSectionBottom = 0;
    final sectionSpacing = 40 * _scaleY;
    final sections = [
      _createAudioSection(_contentWidth),
      _createAccessibilitySection(_contentWidth),
    ];
    for (final section in sections) {
      section.position = Vector2(0, y);
      content.add(section);

      // track the true bottom of each section; last one will be ACCESSIBILITY
      lastSectionBottom = section.position.y + section.size.y;

      y += section.size.y + sectionSpacing;
    }
    content.size = Vector2(_contentWidth, y);

    final infoWidth = (size.x * 0.28).clamp(300.0, size.x * 0.38);
    final infoX = size.x - infoWidth - horizontalMargin;

    _infoPanel!
      ..updateWidth(infoWidth)
      ..position = Vector2(infoX, size.y * 0.24);

    // Right edge of the notes panel
    final infoRight = _infoPanel!.position.x + _infoPanel!.size.x;

    // Bottom edge of the ACCESSIBILITY panel (not including extra spacing)
    final accessibilityBottom = contentTop + lastSectionBottom;

    // Align BACK TO MENU:
    // - bottom with ACCESSIBILITY bottom
    // - right edge with NOTES right edge
    final backButtonX = infoRight - _backButton!.size.x;
    final backButtonY = accessibilityBottom - _backButton!.size.y;

    _backButton!..position = Vector2(backButtonX, backButtonY);
  }

  @override
  bool onKeyEvent(KeyEvent event, Set<LogicalKeyboardKey> keysPressed) {
    if (event is KeyDownEvent &&
        (event.logicalKey == LogicalKeyboardKey.escape ||
            event.logicalKey == LogicalKeyboardKey.gameButtonB)) {
      game.closeSettings();
      return true;
    }
    return false;
  }

  @override
  void onGameResize(Vector2 newSize) {
    super.onGameResize(newSize);
    size = newSize;
    _background.size = newSize;
    if (_content != null) {
      _layout();
    }
  }

  _SectionBlock _createAudioSection(double width) {
    return _SectionBlock(
      title: 'AUDIO',
      width: width,
      entries: [
        _ToggleSetting(
          width: width - 64,
          label: 'Sound Effects',
          description: 'Impact, UI, and ambient cues.',
          valueGetter: () => _gameState.soundEnabled,
          onChanged: (value) {
            if (value != _gameState.soundEnabled) {
              _gameState.toggleSound();
            }
          },
        ),
        _ToggleSetting(
          width: width - 64,
          label: 'Music',
          description: 'Backdrops, story cues, mood swells.',
          valueGetter: () => _gameState.musicEnabled,
          onChanged: (value) {
            if (value != _gameState.musicEnabled) {
              _gameState.toggleMusic();
            }
          },
        ),
        _SliderSetting(
          width: width - 64,
          label: 'Master Volume',
          description: 'Global mix applied to all channels.',
          min: 0,
          max: 1,
          valueGetter: () => _gameState.masterVolume,
          onChanged: (value) => _gameState.setMasterVolume(value),
        ),
      ],
    );
  }

  _SectionBlock _createAccessibilitySection(double width) {
    return _SectionBlock(
      title: 'ACCESSIBILITY',
      width: width,
      entries: [
        _ToggleSetting(
          width: width - 64,
          label: 'Color Shift',
          description: 'Adjust palette for color blind readability.',
          valueGetter: () => _gameState.colorBlindMode,
          onChanged: (value) {
            if (value != _gameState.colorBlindMode) {
              _gameState.toggleColorBlindMode();
            }
          },
        ),
        _ToggleSetting(
          width: width - 64,
          label: 'Reduced Motion',
          description: 'Simplify transitions and particle intensity.',
          valueGetter: () => _gameState.reducedMotion,
          onChanged: (value) {
            if (value != _gameState.reducedMotion) {
              _gameState.toggleReducedMotion();
            }
          },
        ),
        _SliderSetting(
          width: width - 64,
          label: 'Interface Scale',
          description: 'Tune typography and spacing.',
          min: 0.8,
          max: 1.5,
          valueGetter: () => _gameState.fontSize,
          onChanged: (value) => _gameState.setFontSize(value),
        ),
      ],
    );
  }
}

class _SectionBlock extends PositionComponent {
  final String title;
  final List<PositionComponent> entries;

  _SectionBlock({
    required this.title,
    required double width,
    required this.entries,
  }) {
    _assignWidth(width);
  }

  void _assignWidth(double width) {
    size = Vector2(width, 0);
    double y = 56;
    for (final entry in entries) {
      entry.position = Vector2(32, y);
      y += entry.size.y + 18;
    }
    size = Vector2(width, y + 16);
  }

  @override
  Future<void> onLoad() async {
    for (final entry in entries) {
      add(entry);
    }
  }

  @override
  void render(Canvas canvas) {
    final path = Path()
      ..moveTo(0, 0)
      ..lineTo(size.x - 60, 0)
      ..lineTo(size.x, size.y)
      ..lineTo(40, size.y)
      ..close();

    canvas.drawPath(
      path,
      Paint()..color = const Color(0xFF0A0A0A),
    );
    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 3
        ..color = const Color(0xFFFFFFFF),
    );

    final titleBand = Path()
      ..moveTo(0, 0)
      ..lineTo(size.x - 60, 0)
      ..lineTo(size.x - 80, 32)
      ..lineTo(0, 32)
      ..close();
    canvas.drawPath(
      titleBand,
      Paint()..color = const Color(0xFFFFFFFF),
    );
    canvas.drawRect(
      const Rect.fromLTWH(18, 8, 40, 4),
      Paint()..color = const Color(0xFF00D9FF),
    );

    final titlePainter = TextPainter(
      text: TextSpan(
        text: title,
        style: const TextStyle(
          color: Color(0xFF000000),
          fontSize: 20,
          fontWeight: FontWeight.w900,
          letterSpacing: 2,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: size.x - 120);
    titlePainter.paint(canvas, const Offset(72, 6));
  }
}

class _ToggleSetting extends PositionComponent with TapCallbacks {
  _ToggleSetting({
    required this.width,
    required this.label,
    required this.description,
    required this.valueGetter,
    required this.onChanged,
  }) : super(size: Vector2(width, 72));

  final double width;
  final String label;
  final String description;
  final bool Function() valueGetter;
  final void Function(bool) onChanged;

  bool get _isActive => valueGetter();

  @override
  void render(Canvas canvas) {
    final labelColor =
        _isActive ? const Color(0xFFFFFFFF) : const Color(0xFFDDDDDD);

    final chevron = Path()
      ..moveTo(0, 6)
      ..lineTo(18, 0)
      ..lineTo(10, 34)
      ..close();
    canvas.drawPath(
      chevron,
      Paint()..color = const Color(0xFF00D9FF),
    );

    final labelPainter = TextPainter(
      text: TextSpan(
        text: label,
        style: TextStyle(
          color: labelColor,
          fontSize: 18,
          fontWeight: FontWeight.w800,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: size.x - 160);
    labelPainter.paint(canvas, const Offset(28, 4));

    final descPainter = TextPainter(
      text: TextSpan(
        text: description,
        style: const TextStyle(
          color: Color(0xFFAAAAAA),
          fontSize: 13,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: size.x - 160);
    descPainter.paint(canvas, const Offset(28, 28));

    final toggleRect = RRect.fromRectAndRadius(
        Rect.fromLTWH(size.x - 110, 18, 70, 32), const Radius.circular(18));
    canvas.drawRRect(
      toggleRect,
      Paint()
        ..color = _isActive ? const Color(0xFF00D9FF) : const Color(0x44000000),
    );
    canvas.drawCircle(
      Offset(_isActive ? size.x - 46 : size.x - 94, 34),
      14,
      Paint()
        ..color = _isActive ? const Color(0xFF000000) : const Color(0xFFFFFFFF),
    );
    final statePainter = TextPainter(
      text: TextSpan(
        text: _isActive ? 'ON' : 'OFF',
        style: TextStyle(
          color: _isActive ? const Color(0xFF000000) : const Color(0xFFFFFFFF),
          fontSize: 12,
          fontWeight: FontWeight.bold,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    statePainter.paint(
      canvas,
      Offset(size.x - 65 - statePainter.width / 2, 50),
    );
  }

  @override
  void onTapUp(TapUpEvent event) {
    onChanged(!_isActive);
  }
}

class _SliderSetting extends PositionComponent
    with TapCallbacks, DragCallbacks {
  _SliderSetting({
    required this.width,
    required this.label,
    required this.description,
    required this.valueGetter,
    required this.onChanged,
    required this.min,
    required this.max,
  }) : super(size: Vector2(width, 90));

  final double width;
  final String label;
  final String description;
  final double Function() valueGetter;
  final void Function(double) onChanged;
  final double min;
  final double max;

  double get _ratio => ((valueGetter() - min) / (max - min)).clamp(0, 1);

  void _setFromLocal(double localX) {
    final trackWidth = size.x - 202;
    final ratio = (localX - 32).clamp(0, trackWidth) / trackWidth;
    final newValue = min + ratio * (max - min);
    onChanged(newValue);
  }

  @override
  void render(Canvas canvas) {
    final labelPainter = TextPainter(
      text: TextSpan(
        text: label,
        style: const TextStyle(
          color: Color(0xFFFFFFFF),
          fontSize: 18,
          fontWeight: FontWeight.w800,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: size.x - 160);
    labelPainter.paint(canvas, const Offset(16, 8));

    final descPainter = TextPainter(
      text: TextSpan(
        text: description,
        style: const TextStyle(
          color: Color(0xFFBBBBBB),
          fontSize: 13,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: size.x - 160);
    descPainter.paint(canvas, const Offset(16, 34));

    final trackY = 64.0;
    final trackStart = Offset(32, trackY);
    final trackEnd = Offset(size.x - 170, trackY);
    canvas.drawLine(
      trackStart,
      trackEnd,
      Paint()
        ..color = const Color(0xFF333333)
        ..strokeWidth = 6
        ..strokeCap = StrokeCap.round,
    );
    canvas.drawLine(
      trackStart,
      Offset(trackStart.dx + (trackEnd.dx - trackStart.dx) * _ratio, trackY),
      Paint()
        ..color = const Color(0xFF00D9FF)
        ..strokeWidth = 6
        ..strokeCap = StrokeCap.round,
    );

    final handleX = trackStart.dx + (trackEnd.dx - trackStart.dx) * _ratio;
    canvas.drawCircle(
      Offset(handleX, trackY),
      10,
      Paint()..color = const Color(0xFFFFFFFF),
    );

    final valuePainter = TextPainter(
      text: TextSpan(
        text: valueGetter().toStringAsFixed(2),
        style: const TextStyle(
          color: Color(0xFF00D9FF),
          fontSize: 14,
          fontWeight: FontWeight.bold,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    valuePainter.paint(canvas, Offset(size.x - 120, 52));
  }

  @override
  void onTapDown(TapDownEvent event) {
    _setFromLocal(event.localPosition.x);
  }

  @override
  void onDragUpdate(DragUpdateEvent event) {
    _setFromLocal(event.localStartPosition.x + event.localDelta.x);
  }
}

class _InfoPanel extends PositionComponent {
  final String text;
  double _width;

  _InfoPanel({required this.text, required double width}) : _width = width {
    _recalculateSize();
  }

  void updateWidth(double width) {
    _width = width;
    _recalculateSize();
  }

  void _recalculateSize() {
    final body = TextPainter(
      text: TextSpan(
        text: text,
        style: const TextStyle(
          color: Color(0xFFDDDDDD),
          fontSize: 16,
          height: 1.5,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: _width - 40);
    size = Vector2(_width, 46 + body.height + 24);
  }

  @override
  void render(Canvas canvas) {
    final path = Path()
      ..moveTo(0, 0)
      ..lineTo(_width - 40, 0)
      ..lineTo(_width, size.y)
      ..lineTo(20, size.y)
      ..close();

    canvas.drawPath(
      path,
      Paint()..color = const Color(0xFF101010),
    );
    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 3
        ..color = const Color(0xFFFFFFFF),
    );

    final headerPainter = TextPainter(
      text: const TextSpan(
        text: 'NOTES',
        style: TextStyle(
          color: Color(0xFFFFFFFF),
          fontSize: 24,
          fontWeight: FontWeight.w900,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    headerPainter.paint(canvas, const Offset(24, 12));

    final bodyPainter = TextPainter(
      text: TextSpan(
        text: text,
        style: const TextStyle(
          color: Color(0xFFDDDDDD),
          fontSize: 15,
          height: 1.5,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: _width - 40);
    bodyPainter.paint(canvas, const Offset(24, 46));
  }
}

class _BackButton extends PositionComponent with TapCallbacks {
  final String label;
  final VoidCallback onPressed;

  _BackButton({
    required this.label,
    required this.onPressed,
  }) : super(size: Vector2(220, 60));

  @override
  void render(Canvas canvas) {
    final path = Path()
      ..moveTo(0, 0)
      ..lineTo(size.x - 20, 0)
      ..lineTo(size.x, size.y)
      ..lineTo(20, size.y)
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
      text: TextSpan(
        text: label,
        style: const TextStyle(
          color: Color(0xFF000000),
          fontSize: 20,
          fontWeight: FontWeight.w900,
        ),
      ),
      textAlign: TextAlign.center,
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
