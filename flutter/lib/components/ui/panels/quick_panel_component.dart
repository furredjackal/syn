import 'package:flame/components.dart';
import 'package:flutter/material.dart';

import '../../../syn_game.dart';
import '../buttons/text_button_component.dart';
import '../syn_theme.dart';

class QuickMenuBarComponent extends PositionComponent
    with HasGameReference<SynGame> {
  QuickMenuBarComponent({
    required this.onMemory,
    required this.onMap,
    required this.onPossessions,
    super.position,
    super.size,
  });

  final VoidCallback onMemory;
  final VoidCallback onMap;
  final VoidCallback onPossessions;
  final List<TextButtonComponent> _buttons = [];
  final List<Rect> _buttonBounds = [];
  int? _hoveredIndex;

  @override
  Future<void> onLoad() async {
    await super.onLoad();

    final entries = [
      ('MEMORY', onMemory),
      ('MAP', onMap),
      ('POSSESSIONS', onPossessions),
    ];

    final buttonWidth = size.x / entries.length;
    for (var i = 0; i < entries.length; i++) {
      final button = TextButtonComponent(
        label: entries[i].$1,
        onTap: entries[i].$2,
        size: Vector2(buttonWidth - 18, size.y - 16),
        position: Vector2(i * buttonWidth + 9, 8),
      );
      _buttons.add(button);
      _buttonBounds.add(
        Rect.fromLTWH(
          button.position.x,
          button.position.y,
          button.size.x,
          button.size.y,
        ),
      );
      add(button);
    }

    _styleLabels();
  }

  void _styleLabels() {
    for (final button in _buttons) {
      final labels = button.children.whereType<TextComponent>();
      if (labels.isEmpty) continue;
      final label = labels.first;
      label.textRenderer = TextPaint(
        style: SynTopBar.textPrimaryStyle.copyWith(
          fontSize: 14,
          letterSpacing: 1.4,
          shadows: const [
            Shadow(
                color: Color(0xAA000000), blurRadius: 4, offset: Offset(0, 1)),
          ],
        ),
      );
      label.position = button.size / 2 + Vector2(0, 1);
    }
  }

  @override
  void onGameResize(Vector2 size) {
    super.onGameResize(size);
    if (_buttons.isEmpty) return;
    final buttonWidth = this.size.x / _buttons.length;
    _buttonBounds.clear();
    for (var i = 0; i < _buttons.length; i++) {
      final button = _buttons[i];
      button
        ..size = Vector2(buttonWidth - 18, this.size.y - 16)
        ..position = Vector2(i * buttonWidth + 9, 8);
      _buttonBounds.add(
        Rect.fromLTWH(
          button.position.x,
          button.position.y,
          button.size.x,
          button.size.y,
        ),
      );
    }
    _styleLabels();
  }

  @override
  void update(double dt) {
    super.update(dt);
    final mouse = game.mousePosition;
    if (mouse == null) {
      _hoveredIndex = null;
      return;
    }
    final local = mouse - position;
    _updateHover(local);
  }

  void _updateHover(Vector2 local) {
    int? index;
    for (var i = 0; i < _buttonBounds.length; i++) {
      if (_buttonBounds[i].contains(Offset(local.x, local.y))) {
        index = i;
        break;
      }
    }
    if (index != _hoveredIndex) {
      _hoveredIndex = index;
    }
  }

  @override
  void render(Canvas canvas) {
    final buttonCount = _buttonBounds.isEmpty ? 1 : _buttonBounds.length;
    final path = Path()
      ..moveTo(18, 0)
      ..lineTo(size.x - 18, 0)
      ..lineTo(size.x, size.y * 0.65)
      ..lineTo(size.x - 18, size.y)
      ..lineTo(18, size.y)
      ..lineTo(0, size.y * 0.35)
      ..close();

    canvas.drawShadow(path, const Color(0x99000000), 14, false);
    canvas.drawPath(
      path,
      Paint()
        ..shader = const LinearGradient(
          colors: [Color(0xFF070A12), Color(0xFF0E1524)],
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
        ).createShader(Rect.fromLTWH(0, 0, size.x, size.y)),
    );

    final slash1 = Path()
      ..moveTo(size.x * 0.18, -6)
      ..lineTo(size.x * 0.34, -6)
      ..lineTo(size.x * 0.26, size.y * 0.72)
      ..lineTo(size.x * 0.10, size.y * 0.72)
      ..close();
    canvas.drawPath(
      slash1,
      Paint()
        ..color = const Color(0xFF00E6FF).withValues(alpha: 0.08)
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 4),
    );

    final slash2 = Path()
      ..moveTo(size.x * 0.62, -8)
      ..lineTo(size.x * 0.82, -8)
      ..lineTo(size.x * 0.70, size.y * 0.78)
      ..lineTo(size.x * 0.52, size.y * 0.78)
      ..close();
    canvas.drawPath(
      slash2,
      Paint()
        ..color = const Color(0xFF9A27FF).withValues(alpha: 0.05)
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 5),
    );

    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 3
        ..color = const Color(0xFF00E6FF),
    );

    if (_hoveredIndex != null && _hoveredIndex! < _buttonBounds.length) {
      final hoverRect = _buttonBounds[_hoveredIndex!];
      final highlightPath = Path()
        ..addRRect(
            RRect.fromRectAndRadius(hoverRect, const Radius.circular(6)));
      canvas.drawPath(
        highlightPath,
        Paint()
          ..color = const Color(0xFF00E6FF).withValues(alpha: 0.12)
          ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 4),
      );
      canvas.drawLine(
        Offset(hoverRect.left + 8, hoverRect.bottom - 5),
        Offset(hoverRect.right - 8, hoverRect.bottom - 1),
        Paint()
          ..color = const Color(0xFF00E6FF).withValues(alpha: 0.7)
          ..strokeWidth = 2
          ..strokeCap = StrokeCap.round,
      );
    }

    for (var i = 1; i < buttonCount; i++) {
      final x = size.x * i / buttonCount;
      final separator = Path()
        ..moveTo(x - 6, 4)
        ..lineTo(x + 6, size.y - 4);
      canvas.drawPath(
        separator,
        Paint()
          ..color = const Color(0xFF00E6FF).withValues(alpha: 0.22)
          ..strokeWidth = 1.2
          ..style = PaintingStyle.stroke,
      );
    }
  }
}
