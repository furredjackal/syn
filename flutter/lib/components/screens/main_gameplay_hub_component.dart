import 'dart:ui';

import 'package:flame/camera.dart';
import 'package:flame/components.dart';
import 'package:flame/events.dart';
import 'package:flutter/material.dart'
    show Colors, TextStyle, FontWeight, LinearGradient, Alignment;

import '../../syn_game.dart';

/// Floating gameplay hub layout per GDD: inboard panels, clear breathing room.
class MainGameplayHubComponent extends Component with HasGameReference<SynGame> {
  late final World _world;
  late final CameraComponent _camera;

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    _world = World();
    _camera = CameraComponent(
      world: _world,
      viewport: FixedResolutionViewport(resolution: game.size.clone()),
    );

    add(_world);
    add(_camera);

    _addBackground();
    _addLayout();
  }

  void _addBackground() {
    _world.add(
      RectangleComponent(
        size: game.size.clone(),
        paint: Paint()..color = const Color(0xFF05070D),
      ),
    );
    _world.add(
      GridComponent(
        size: game.size.clone(),
        paint: Paint()
          ..color = const Color(0x2200D9FF)
          ..strokeWidth = 1.2
          ..style = PaintingStyle.stroke,
        cell: 64,
      ),
    );
  }

  void _addLayout() {
    final w = game.size.x;
    final h = game.size.y;

    // Primary panels (positions in screen space; anchor top-left).
    final topBar = TopBarComponent(
      position: Vector2(0.10 * w, 0.04 * h),
      size: Vector2(0.80 * w, 0.10 * h),
    );
    final eventCanvas = EventCanvasComponent(
      position: Vector2(0.20 * w, 0.17 * h),
      size: Vector2(0.60 * w, 0.55 * h),
    );
    final statsPanel = StatPanelComponent(
      position: Vector2(0.08 * w, 0.22 * h),
      size: Vector2(0.18 * w, 0.50 * h),
    );
    final relationshipPanel = RelationshipPanelComponent(
      position: Vector2(0.74 * w, 0.22 * h),
      size: Vector2(0.18 * w, 0.50 * h),
    );
    final quickMenu = QuickMenuBarComponent(
      position: Vector2(0.18 * w, 0.80 * h),
      size: Vector2(0.64 * w, 0.10 * h),
      onMemory: game.showMemoryJournal,
      onMap: game.showWorldMap,
      onPossessions: game.showPossessions,
      onSaveLoad: game.showSaveLoad,
      onSettings: game.showSettings,
      onPause: game.togglePauseOverlay,
    );
    final notifications = NotificationStackComponent(
      position: Vector2(0.70 * w, 0.06 * h),
    );

    // Z-order: add in back-to-front order.
    _world.addAll([
      topBar,
      statsPanel,
      relationshipPanel,
      quickMenu,
      eventCanvas,
      notifications,
    ]);
  }
}

class GridComponent extends PositionComponent {
  GridComponent({required this.cell, required this.paint, super.size});
  final double cell;
  final Paint paint;

  @override
  void render(Canvas canvas) {
    final cols = (size.x / cell).ceil();
    final rows = (size.y / cell).ceil();
    for (var i = 0; i <= cols; i++) {
      final x = i * cell;
      canvas.drawLine(Offset(x, 0), Offset(x, size.y), paint);
    }
    for (var j = 0; j <= rows; j++) {
      final y = j * cell;
      canvas.drawLine(Offset(0, y), Offset(size.x, y), paint);
    }
  }
}

class TopBarComponent extends PositionComponent {
  TopBarComponent({super.position, super.size});

  @override
  void render(Canvas canvas) {
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    final path = Path()
      ..moveTo(18, 0)
      ..lineTo(size.x - 18, 0)
      ..lineTo(size.x, size.y)
      ..lineTo(18, size.y)
      ..close();
    canvas.drawShadow(path, const Color(0xAA000000), 12, false);
    final gradient = LinearGradient(
      colors: const [Color(0xFF0F1624), Color(0xFF0C0F18)],
      begin: Alignment.topLeft,
      end: Alignment.bottomRight,
    );
    canvas.drawPath(
      path,
      Paint()
        ..shader = gradient.createShader(rect)
        ..style = PaintingStyle.fill,
    );
    canvas.drawPath(
      path,
      Paint()
        ..color = const Color(0xFF00D9FF)
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2,
    );

    // Chips: life stage, age, mood, menu icon.
    final chipPaint = Paint()..color = const Color(0xFF121A2C);
    _drawChip(canvas, Offset(24, size.y / 2 - 18), const Size(140, 36), chipPaint,
        label: 'STAGE: RISE');
    _drawChip(canvas, Offset(176, size.y / 2 - 18), const Size(120, 36),
        chipPaint,
        label: 'AGE: 27');
    _drawCircleIndicator(
      canvas,
      center: Offset(size.x - 120, size.y / 2),
      radius: 24,
      color: const Color(0xFF00E6FF),
    );
    _drawMenuIcon(canvas, Offset(size.x - 52, size.y / 2 - 14));
  }

  void _drawChip(Canvas canvas, Offset origin, Size s, Paint paint,
      {required String label}) {
    final r = RRect.fromRectAndRadius(
      Rect.fromLTWH(origin.dx, origin.dy, s.width, s.height),
      const Radius.circular(14),
    );
    canvas.drawRRect(r, paint);
    final textPainter = TextPaint(
      style: const TextStyle(
        color: Colors.white,
        fontSize: 14,
        fontWeight: FontWeight.w700,
      ),
    ).toTextPainter(label);
    textPainter.paint(
      canvas,
      Offset(origin.dx + 12, origin.dy + (s.height - textPainter.height) / 2),
    );
  }

  void _drawCircleIndicator(Canvas canvas,
      {required Offset center, required double radius, required Color color}) {
    canvas.drawCircle(center, radius, Paint()..color = const Color(0x4414FFD2));
    canvas.drawCircle(center, radius - 4, Paint()..color = color);
    canvas.drawCircle(center, radius - 4, Paint()
      ..style = PaintingStyle.stroke
      ..color = Colors.black
      ..strokeWidth = 2);
  }

  void _drawMenuIcon(Canvas canvas, Offset origin) {
    final paint = Paint()
      ..color = Colors.white
      ..strokeWidth = 3
      ..strokeCap = StrokeCap.round;
    canvas.drawLine(origin, origin + const Offset(28, 0), paint);
    canvas.drawLine(origin + const Offset(0, 10),
        origin + const Offset(28, 10), paint);
    canvas.drawLine(origin + const Offset(0, 20),
        origin + const Offset(28, 20), paint);
  }
}

class EventCanvasComponent extends PositionComponent {
  EventCanvasComponent({super.position, super.size});

  @override
  void render(Canvas canvas) {
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(20)),
      Paint()..color = const Color(0xFF0E1524),
    );
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(20)),
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 3
        ..color = const Color(0xFF00D9FF),
    );

    final title = TextPaint(
      style: const TextStyle(
        color: Colors.white,
        fontSize: 26,
        fontWeight: FontWeight.w800,
        letterSpacing: 1.2,
      ),
    ).toTextPainter('CURRENT EVENT: SIGNAL IN THE FOG');
    title.paint(canvas, const Offset(24, 20));

    final body = TextPaint(
      style: const TextStyle(
        color: Color(0xFFB8C2D6),
        fontSize: 16,
        height: 1.4,
      ),
    ).toTextPainter(
      'A flickering broadcast spills through the neon mist. Do you trace the source, '
      'boost the signal, or jam it? Choices here should show tiny impact previews.',
    );
    body.paint(canvas, const Offset(24, 60));

    // Choice stubs with impact icons.
    final choices = [
      'TRACE THE SOURCE  [+Trust, +Risk]',
      'BOOST THE SIGNAL  [+Influence, -Stealth]',
      'JAM IT  [+Security, -Info]',
    ];
    for (var i = 0; i < choices.length; i++) {
      final y = 160 + i * 56;
      canvas.drawRRect(
        RRect.fromRectAndRadius(
          Rect.fromLTWH(24, y.toDouble(), size.x - 48, 44),
          const Radius.circular(12),
        ),
        Paint()..color = const Color(0xFF121A2C),
      );
      final choiceText = TextPaint(
        style: const TextStyle(
          color: Colors.white,
          fontSize: 16,
          fontWeight: FontWeight.w700,
          letterSpacing: 1.1,
        ),
      ).toTextPainter(choices[i]);
      choiceText.paint(canvas, Offset(36, y + 12));
      _drawImpactDots(canvas, Offset(size.x - 120, y + 22));
    }
  }

  void _drawImpactDots(Canvas canvas, Offset origin) {
    final paints = [
      Paint()..color = const Color(0xFF00D9FF),
      Paint()..color = const Color(0xFFFF8F5B),
      Paint()..color = const Color(0xFFFF4C4C),
    ];
    for (var i = 0; i < paints.length; i++) {
      canvas.drawCircle(origin + Offset(i * 14, 0), 5, paints[i]);
    }
  }
}

class StatPanelComponent extends PositionComponent {
  StatPanelComponent({super.position, super.size});

  final _stats = const [
    ('HEALTH', 0.76),
    ('WEALTH', 0.42),
    ('CHARISMA', 0.61),
    ('INFLUENCE', 0.35),
    ('SECURITY', 0.54),
    ('STEALTH', 0.48),
  ];

  @override
  void render(Canvas canvas) {
    _drawPanelShell(canvas);
    for (var i = 0; i < _stats.length; i++) {
      final y = 32 + i * 56;
      _drawStat(
        canvas,
        origin: Offset(16, y.toDouble()),
        label: _stats[i].$1,
        value: _stats[i].$2,
      );
    }
  }

  void _drawPanelShell(Canvas canvas) {
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(16)),
      Paint()..color = const Color(0xFF0B101C),
    );
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(16)),
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = const Color(0xFF7B5CFF),
    );
  }

  void _drawStat(Canvas canvas,
      {required Offset origin,
      required String label,
      required double value}) {
    final labelText = TextPaint(
      style: const TextStyle(
        color: Colors.white,
        fontSize: 14,
        fontWeight: FontWeight.w700,
      ),
    ).toTextPainter(label);
    labelText.paint(canvas, origin);

    final barWidth = size.x - 32;
    final y = origin.dy + 22;
    canvas.drawRRect(
      RRect.fromRectAndRadius(
        Rect.fromLTWH(origin.dx, y, barWidth, 10),
        const Radius.circular(6),
      ),
      Paint()..color = const Color(0xFF131A28),
    );
    canvas.drawRRect(
      RRect.fromRectAndRadius(
        Rect.fromLTWH(origin.dx, y, barWidth * value, 10),
        const Radius.circular(6),
      ),
      Paint()..color = const Color(0xFF00D9FF),
    );

    final valText = TextPaint(
      style: const TextStyle(
        color: Color(0xFFB8C2D6),
        fontSize: 12,
        fontWeight: FontWeight.w600,
      ),
    ).toTextPainter('${(value * 100).round()}');
    valText.paint(
      canvas,
      Offset(origin.dx, y + 12),
    );
  }
}

class RelationshipPanelComponent extends PositionComponent {
  RelationshipPanelComponent({super.position, super.size});

  final _people = const [
    ('KAZ', 0.72, 0.18),
    ('ILA', 0.55, 0.32),
    ('MOTHER', 0.63, 0.42),
    ('FIXER', 0.44, 0.58),
  ];

  @override
  void render(Canvas canvas) {
    _drawPanelShell(canvas);
    for (var i = 0; i < _people.length; i++) {
      final y = 32 + i * 72;
      _drawCard(
        canvas,
        origin: Offset(12, y.toDouble()),
        name: _people[i].$1,
        trust: _people[i].$2,
        tension: _people[i].$3,
      );
    }
  }

  void _drawPanelShell(Canvas canvas) {
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(16)),
      Paint()..color = const Color(0xFF0B101C),
    );
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(16)),
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = const Color(0xFFFF3C8F),
    );
  }

  void _drawCard(Canvas canvas,
      {required Offset origin,
      required String name,
      required double trust,
      required double tension}) {
    final rrect = RRect.fromRectAndRadius(
      Rect.fromLTWH(origin.dx, origin.dy, size.x - 24, 60),
      const Radius.circular(12),
    );
    canvas.drawRRect(rrect, Paint()..color = const Color(0xFF111828));
    canvas.drawRRect(
      rrect,
      Paint()
        ..style = PaintingStyle.stroke
        ..color = const Color(0x335CFF90),
    );

    final nameText = TextPaint(
      style: const TextStyle(
        color: Colors.white,
        fontSize: 14,
        fontWeight: FontWeight.w800,
      ),
    ).toTextPainter(name);
    nameText.paint(canvas, Offset(origin.dx + 12, origin.dy + 10));

    _drawBar(canvas,
        origin: Offset(origin.dx + 12, origin.dy + 32),
        label: 'Trust',
        value: trust,
        color: const Color(0xFF5CFF90));
    _drawBar(canvas,
        origin: Offset(origin.dx + 12, origin.dy + 46),
        label: 'Conflict',
        value: tension,
        color: const Color(0xFFFF4C4C));
  }

  void _drawBar(Canvas canvas,
      {required Offset origin,
      required String label,
      required double value,
      required Color color}) {
    canvas.drawRect(
      Rect.fromLTWH(origin.dx, origin.dy, size.x - 48, 8),
      Paint()..color = const Color(0xFF1A2234),
    );
    canvas.drawRect(
      Rect.fromLTWH(origin.dx, origin.dy, (size.x - 48) * value, 8),
      Paint()..color = color,
    );
  }
}

class QuickMenuBarComponent extends PositionComponent {
  QuickMenuBarComponent({
    super.position,
    super.size,
    required this.onMemory,
    required this.onMap,
    required this.onPossessions,
    required this.onSaveLoad,
    required this.onSettings,
    required this.onPause,
  });

  final VoidCallback onMemory;
  final VoidCallback onMap;
  final VoidCallback onPossessions;
  final VoidCallback onSaveLoad;
  final VoidCallback onSettings;
  final VoidCallback onPause;

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    final entries = [
      ('MEMORY', onMemory),
      ('MAP', onMap),
      ('POSSESSIONS', onPossessions),
      ('SAVE/LOAD', onSaveLoad),
      ('SETTINGS', onSettings),
      ('PAUSE', onPause),
    ];
    final buttonWidth = size.x / entries.length;
    for (var i = 0; i < entries.length; i++) {
      add(
        _QuickButton(
          label: entries[i].$1,
          onTap: entries[i].$2,
          position: Vector2(i * buttonWidth, 0),
          size: Vector2(buttonWidth - 8, size.y),
        ),
      );
    }
  }

  @override
  void render(Canvas canvas) {
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(18)),
      Paint()..color = const Color(0xFF0D131F),
    );
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(18)),
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = const Color(0xFF00D9FF),
    );
  }
}

class _QuickButton extends PositionComponent with TapCallbacks {
  _QuickButton({
    required this.label,
    required this.onTap,
    super.position,
    super.size,
  }) : super(anchor: Anchor.topLeft);

  final String label;
  final VoidCallback onTap;
  bool _hovered = false;

  @override
  void render(Canvas canvas) {
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(12)),
      Paint()
        ..color = _hovered
            ? const Color(0xFF131C2E)
            : const Color(0xFF0D131F),
    );
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(12)),
      Paint()
        ..style = PaintingStyle.stroke
        ..color = const Color(0x3300D9FF),
    );
    final text = TextPaint(
      style: const TextStyle(
        color: Colors.white,
        fontSize: 14,
        fontWeight: FontWeight.w800,
        letterSpacing: 1.2,
      ),
    ).toTextPainter(label);
    text.paint(
      canvas,
      Offset(
        (size.x - text.width) / 2,
        (size.y - text.height) / 2,
      ),
    );
  }

  @override
  void onTapDown(TapDownEvent event) {
    _hovered = true;
  }

  @override
  void onTapUp(TapUpEvent event) {
    _hovered = false;
    onTap();
  }

  @override
  void onTapCancel(TapCancelEvent event) {
    _hovered = false;
  }
}

class NotificationStackComponent extends PositionComponent {
  NotificationStackComponent({super.position});

  final _items = const [
    '+10 TRUST (KAZ) — Backed their play',
    '-5 STEALTH — Signal boosted loudly',
    '+1 MEMORY — “Signal in the Fog”',
  ];

  @override
  void render(Canvas canvas) {
    for (var i = 0; i < _items.length; i++) {
      final dy = i * 48;
      _drawToast(
        canvas,
        Offset(0, dy.toDouble()),
        _items[i],
      );
    }
  }

  void _drawToast(Canvas canvas, Offset origin, String text) {
    const width = 280.0;
    const height = 40.0;
    final rrect = RRect.fromRectAndRadius(
      Rect.fromLTWH(origin.dx, origin.dy, width, height),
      const Radius.circular(10),
    );
    canvas.drawShadow(Path()..addRRect(rrect), const Color(0xAA000000), 8, false);
    canvas.drawRRect(
      rrect,
      Paint()..color = const Color(0xFF101828),
    );
    canvas.drawRRect(
      rrect,
      Paint()
        ..style = PaintingStyle.stroke
        ..color = const Color(0xFF00D9FF),
    );
    final textPainter = TextPaint(
      style: const TextStyle(
        color: Colors.white,
        fontSize: 12,
        fontWeight: FontWeight.w700,
      ),
    ).toTextPainter(text);
    textPainter.paint(
      canvas,
      Offset(origin.dx + 12, origin.dy + (height - textPainter.height) / 2),
    );
  }
}
