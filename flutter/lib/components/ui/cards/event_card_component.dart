import 'dart:math' as math;

import 'package:flame/components.dart';
import 'package:flame/effects.dart';
import 'package:flame/events.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../../../models/game_state.dart';
import '../../../syn_game.dart';
import '../../../ui/ui_signal_bus.dart';
import '../buttons/choice_button_component.dart';
import '../paint/angled_panel.dart';
import '../syn_theme.dart';

/// EventCanvas: Centered focal point with slash transitions.
class EventCardComponent extends PositionComponent
    with HasGameReference<SynGame>, UiSignalListener, KeyboardHandler {
  final GameEvent event;
  final Function(int) onChoice;

  // Child references for layout updates
  late final _EventCanvasBackground _background;
  late final _EventBorder _border;
  late final _EventHeader _header;
  late final _SlashAccent _accent;

  // Content container - PositionComponent allows transform/scale
  final PositionComponent _contentRoot = PositionComponent();
  final List<_EventChoiceComponent> _choiceButtons = [];
  final Set<String> _highlightedStats = {};
  String? _highlightedRelationshipId;
  int _focusedChoiceIndex = 0;


  EventCardComponent({
    required this.event,
    required this.onChoice,
    super.position,
    super.size,
  });

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    game.uiSignals.register(this);

    // Ensure we have a default size if initialized with zero
    if (size.isZero()) size = Vector2(600, 800);

    // 1. Initialize Static Layers (Bottom to Top)
    _background = _EventCanvasBackground(parentSize: size);
    _border = _EventBorder(parentSize: size);
    _accent = _SlashAccent(parentSize: size);

    // Header floats above
    _header = _EventHeader(
      lifeStage: event.lifeStage,
      age: event.age,
      parentSize: size,
    );

    addAll([_background, _border, _accent, _contentRoot, _header]);

    // 2. Build Dynamic Content (Text, Buttons)
    _rebuildContent();

    // 3. Entrance Animation
    add(_SlashTransition(size: size, duration: 0.4));
  }

  @override
  void onRemove() {
    game.uiSignals.unregister(this);
    super.onRemove();
  }

  @override
  void onGameResize(Vector2 newSize) {
    super.onGameResize(newSize);
    if (!isLoaded) return;

    _background.resize(newSize);
    _border.resize(newSize);
    _accent.resize(newSize);
    _header.resize(newSize);

    _rebuildContent();
  }

  void _rebuildContent() {
    _contentRoot.removeAll(_contentRoot.children);
    _choiceButtons.clear();

    // Layout constants
    const double horizontalPadding = 42.0; 
    const double spacingAfterHeader = 32.0; 
    const double buttonHeight = 64.0;
    const double buttonGap = 16.0;

    // Header overlaps top, so start content lower
    double layoutY = 90.0 + spacingAfterHeader;

    if (size.x <= horizontalPadding * 2) return;

    final contentWidth = size.x - horizontalPadding * 2;

    // 1. Title Banner
    final titleBanner = _EventTitleBanner(
      title: event.title,
      width: contentWidth,
    )..position = Vector2(horizontalPadding, layoutY);
    _contentRoot.add(titleBanner);
    layoutY += titleBanner.height + 20.0;

    // 2. Tags (if any)
    if (event.tags.isNotEmpty) {
      final tagsRow = _EventTagChipRow(
        tags: event.tags,
        maxWidth: contentWidth,
      )..position = Vector2(horizontalPadding, layoutY);
      _contentRoot.add(tagsRow);
      layoutY += tagsRow.computedHeight + 20.0;
    }

    // 3. Description Text
    final descriptionPainter = TextPainter(
      text: TextSpan(
        text: event.description,
        style: SynTextStyles.body.copyWith(
          fontSize: 18,
          height: 1.6,
          color: SynColors.textPrimary.withValues(alpha: 0.95),
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: contentWidth);

    final description = _TextPainterComponent(painter: descriptionPainter)
      ..position = Vector2(horizontalPadding, layoutY);
    _contentRoot.add(description);
    layoutY += description.size.y + 28.0;

    // 4. Divider
    final accentDivider = _AccentDivider(
      width: contentWidth * 0.7,
    )..position = Vector2(horizontalPadding, layoutY);
    _contentRoot.add(accentDivider);
    layoutY += accentDivider.size.y + 36.0;

    // 5. Impact Summary (if any)
    if (event.deltas.isNotEmpty) {
      final impactRow = _ImpactSummaryRow(
        stats: event.deltas.keys.toList(),
        maxWidth: contentWidth,
      )..position = Vector2(horizontalPadding, layoutY);
      _contentRoot.add(impactRow);
      layoutY += impactRow.size.y + 28.0;
    }

    // 6. Choices
    for (var i = 0; i < event.choices.length; i++) {
      final choice = event.choices[i];
      final skewOffset = 8.0;
      final choiceButton = _EventChoiceComponent(
        choice: choice,
        index: i,
        onSelect: () => onChoice(i),
        position: Vector2(horizontalPadding + skewOffset, layoutY),
        size: Vector2(contentWidth - skewOffset, buttonHeight),
      );

      // Entrance Animation
      choiceButton.setOpacity(0);
      choiceButton
        ..add(
          _CustomOpacityEffect(
            target: choiceButton,
            duration: 0.4,
            startDelay: 0.1 + (0.1 * i),
          ),
        )
        ..add(
          MoveEffect.by(
            Vector2(0, -10),
            EffectController(
              duration: 0.4,
              curve: Curves.easeOut,
              startDelay: 0.1 + (0.1 * i),
            ),
          ),
        );
      choiceButton.position.y += 10;

      _contentRoot.add(choiceButton);
      _choiceButtons.add(choiceButton);
      layoutY += buttonHeight + buttonGap;
    }

    // 7. Overflow Scaling
    final maxH = size.y - 40;
    if (layoutY > maxH && maxH > 0) {
      final factor = (maxH / layoutY).clamp(0.6, 1.0);
      _contentRoot.scale = Vector2.all(factor);
      _contentRoot.position.y = (size.y - (layoutY * factor)) / 2 + 20;
    } else {
      _contentRoot.scale = Vector2.all(1.0);
      _contentRoot.position.y = 0;
    }
  }

  void _applyChoiceHighlights() {
    for (var i = 0; i < _choiceButtons.length; i++) {
      final button = _choiceButtons[i];
      final affectsHighlightedStat =
          button.choice.statChanges.keys.any(_highlightedStats.contains);
      final hasRelationshipHighlight = _highlightedRelationshipId != null;
      button.showSignalHighlight = affectsHighlightedStat || hasRelationshipHighlight;
      button.isFocused = i == _focusedChoiceIndex;
    }
  }

  @override
  void onUiSignal(UiSignal signal) {
    switch (signal.type) {
      case 'stat:updated':
        final payload = signal.payload;
        if (payload is Map && payload['id'] is String) {
          final id = payload['id'] as String;
          _highlightedStats.add(id);
          _applyChoiceHighlights();
        }
        break;
      case 'relationship:hovered':
        final payload = signal.payload;
        if (payload is Map && payload['id'] is String) {
          _highlightedRelationshipId = payload['id'] as String;
          _applyChoiceHighlights();
        }
        break;
      default:
        break;
    }
  }

  void _focusChoice(int index) {
    if (_choiceButtons.isEmpty) return;
    _focusedChoiceIndex = index.clamp(0, _choiceButtons.length - 1);
    _applyChoiceHighlights();
  }

  void _activateFocusedChoice() {
    if (_choiceButtons.isEmpty) return;
    _choiceButtons[_focusedChoiceIndex].triggerSelect();
  }

  @override
  bool onKeyEvent(KeyEvent event, Set<LogicalKeyboardKey> keysPressed) {
    if (event is! KeyDownEvent || _choiceButtons.isEmpty) return false;
    final key = event.logicalKey;
    if (key == LogicalKeyboardKey.arrowDown || key == LogicalKeyboardKey.keyS) {
      _focusChoice((_focusedChoiceIndex + 1) % _choiceButtons.length);
      return true;
    }
    if (key == LogicalKeyboardKey.arrowUp || key == LogicalKeyboardKey.keyW) {
      _focusChoice(
          (_focusedChoiceIndex - 1 + _choiceButtons.length) % _choiceButtons.length);
      return true;
    }
    if (key == LogicalKeyboardKey.enter || key == LogicalKeyboardKey.space) {
      _activateFocusedChoice();
      return true;
    }
    return false;
  }

  @override
  void update(double dt) {
    super.update(dt);
  }
}

class _EventChoiceComponent extends PositionComponent
    with HasGameReference<SynGame>, TapCallbacks, HoverCallbacks {
  _EventChoiceComponent({
    required this.choice,
    required this.index,
    required this.onSelect,
    super.position,
    required Vector2 size,
  }) : super(size: size, anchor: Anchor.topLeft);

  final GameChoice choice;
  final int index;
  final VoidCallback onSelect;
  bool showSignal = false;
  bool _hovered = false;
  bool _focused = false;
  late final ChoiceButtonComponent _button;

  set showSignalHighlight(bool value) {
    showSignal = value;
  }

  set isFocused(bool value) {
    _focused = value;
  }

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    _button = ChoiceButtonComponent(
      choice: choice,
      index: index,
      onPressed: onSelect,
      position: Vector2.zero(),
      size: size,
    );
    add(_button);
  }

  void triggerSelect() {
    onSelect();
  }

  void setOpacity(double value) {
    _button.setOpacity(value);
  }

  void _scaleTo(Vector2 target, double duration) {
    removeWhere((c) => c is ScaleEffect);
    add(
      ScaleEffect.to(
        target,
        EffectController(duration: duration, curve: Curves.easeOut),
      ),
    );
  }

  @override
  void onHoverEnter() {
    _hovered = true;
    _scaleTo(Vector2.all(1.03), 0.15);
  }

  @override
  void onHoverExit() {
    _hovered = false;
    _scaleTo(Vector2.all(1.0), 0.12);
  }

  @override
  void onTapDown(TapDownEvent event) {
    _scaleTo(Vector2.all(0.97), 0.08);
  }

  @override
  void onTapUp(TapUpEvent event) {
    _scaleTo(Vector2.all(1.0), 0.1);
    triggerSelect();
  }

  @override
  void onTapCancel(TapCancelEvent event) {
    _scaleTo(Vector2.all(1.0), 0.1);
  }

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    if (showSignal || _focused || _hovered) {
      drawAngledPanel(
        canvas,
        size.toRect(),
        fill: Colors.transparent,
        border: showSignal
            ? SynColors.primaryCyan
            : _focused
                ? SynColors.accentCyan
                : SynColors.primaryCyan.withValues(alpha: 0.5),
        borderWidth: 2.5,
        cutTopRight: true,
        cutBottomLeft: true,
      );
    }
  }
}

// Custom Effect to bridge Flame's timing with our custom setOpacity
class _CustomOpacityEffect extends Component {
  final _EventChoiceComponent target;
  final double duration;
  final double startDelay;
  double _timer = 0;

  _CustomOpacityEffect({
    required this.target,
    required this.duration,
    this.startDelay = 0,
  });

  @override
  void update(double dt) {
    _timer += dt;
    if (_timer < startDelay) return;

    final progress = ((_timer - startDelay) / duration).clamp(0.0, 1.0);
    target.setOpacity(progress);
    
    if (progress >= 1.0) removeFromParent();
  }
}

// ---------------------------------------------------------------------------
// SUB-COMPONENTS
// ---------------------------------------------------------------------------
// (Keep the _EventCanvasBackground, _EventBorder, _SlashAccent, _EventHeader, 
// _EventTitleBanner, _AccentDivider, _TextPainterComponent, _EventTagChipRow, 
// _ImpactSummaryRow, _SlashTransition classes as they were in the previous fixed version)
// Be sure to remove _TappableButtonWrapper entirely.

class _EventCanvasBackground extends PositionComponent {
  _EventCanvasBackground({required Vector2 parentSize}) {
    resize(parentSize);
  }

  final Paint _fillPaint = Paint()
    ..color = const Color(0xFF000000).withValues(alpha: 0.75)
    ..style = PaintingStyle.fill;

  final Paint _gradPaint = Paint();
  final Path _path = Path();

  void resize(Vector2 newSize) {
    size = newSize;
    const angleOffset = 16.0;
    _path.reset();
    _path
      ..moveTo(angleOffset, 0)
      ..lineTo(size.x, 0)
      ..lineTo(size.x - angleOffset, size.y)
      ..lineTo(0, size.y)
      ..close();

    _gradPaint.shader = LinearGradient(
      begin: Alignment.topCenter,
      end: Alignment.bottomCenter,
      colors: [
        const Color(0xFF1a1a1a).withValues(alpha: 0.4),
        const Color(0xFF0a0a0a).withValues(alpha: 0.2),
      ],
    ).createShader(size.toRect());
  }

  @override
  void render(Canvas canvas) {
    canvas.drawPath(_path, _fillPaint);
    canvas.drawPath(_path, _gradPaint);
  }
}

class _EventBorder extends PositionComponent {
  _EventBorder({required Vector2 parentSize}) {
    resize(parentSize);
  }

  final Paint _outerPaint = Paint()
    ..color = const Color(0xFF00D9FF)
    ..style = PaintingStyle.stroke
    ..strokeWidth = 3;

  final Paint _innerPaint = Paint()
    ..color = const Color(0xFF00D9FF).withValues(alpha: 0.3)
    ..style = PaintingStyle.stroke
    ..strokeWidth = 1;

  final Path _outerPath = Path();
  final Path _innerPath = Path();

  void resize(Vector2 newSize) {
    size = newSize;
    const angleOffset = 16.0;

    _outerPath.reset();
    _outerPath
      ..moveTo(angleOffset, 0)
      ..lineTo(size.x, 0)
      ..lineTo(size.x - angleOffset, size.y)
      ..lineTo(0, size.y)
      ..close();

    const innerOffset = 4.0;
    _innerPath.reset();
    _innerPath
      ..moveTo(angleOffset + innerOffset, innerOffset)
      ..lineTo(size.x - innerOffset, innerOffset)
      ..lineTo(size.x - angleOffset - innerOffset, size.y - innerOffset)
      ..lineTo(innerOffset, size.y - innerOffset)
      ..close();
  }

  @override
  void render(Canvas canvas) {
    canvas.drawPath(_outerPath, _outerPaint);
    canvas.drawPath(_innerPath, _innerPaint);
  }
}

class _SlashAccent extends PositionComponent {
  _SlashAccent({required Vector2 parentSize}) {
    resize(parentSize);
  }

  final Paint _glowPaint = Paint()
    ..color = const Color(0xFF00D9FF).withValues(alpha: 0.15)
    ..style = PaintingStyle.fill
    ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 12.0);

  final Paint _corePaint = Paint()
    ..color = const Color(0xFF00D9FF).withValues(alpha: 0.25)
    ..style = PaintingStyle.fill;

  final Path _slashPath = Path();
  final Path _corePath = Path();

  void resize(Vector2 newSize) {
    size = newSize;
    const slashWidth = 80.0;

    _slashPath.reset();
    _slashPath
      ..moveTo(size.x - slashWidth, -size.y * 0.1)
      ..lineTo(size.x + slashWidth, size.y * 0.8)
      ..lineTo(size.x - slashWidth + 8, size.y * 0.8)
      ..lineTo(size.x + slashWidth - 8, -size.y * 0.1)
      ..close();

    const coreW = 40.0;
    _corePath.reset();
    _corePath
      ..moveTo(size.x - coreW, -size.y * 0.1)
      ..lineTo(size.x + coreW, size.y * 0.8)
      ..lineTo(size.x - coreW + 4, size.y * 0.8)
      ..lineTo(size.x + coreW - 4, -size.y * 0.1)
      ..close();
  }

  @override
  void render(Canvas canvas) {
    drawAngledPanel(
      canvas,
      size.toRect(),
      fill: const Color(0xFF0A0A0A).withValues(alpha: 0.6),
      border: const Color(0xFF00D9FF),
      borderWidth: 3,
      cutTopRight: true,
      cutBottomLeft: true,
    );
    canvas.drawPath(_slashPath, _glowPaint);
    canvas.drawPath(_corePath, _corePaint);
  }
}

class _EventHeader extends PositionComponent {
  _EventHeader({
    required this.lifeStage,
    required this.age,
    required Vector2 parentSize,
  }) {
    _relayout(parentSize);
  }

  final String lifeStage;
  final int age;

  final Paint _fillPaint = Paint();
  final Paint _strokePaint = Paint()
    ..style = PaintingStyle.stroke
    ..strokeWidth = 2.5
    ..color = const Color(0xFF00D9FF);

  final Path _bgPath = Path();

  TextPainter? _stageLabel;
  TextPainter? _lifeStageValue;
  TextPainter? _ageLabel;
  TextPainter? _ageValue;

  void resize(Vector2 parentSize) {
    _relayout(parentSize);
  }

  void _relayout(Vector2 parentSize) {
    final headerW = math.max(parentSize.x - 32.0, parentSize.x * 0.75);
    size = Vector2(headerW, 90);

    position = Vector2((parentSize.x - headerW) / 2, -18.0);

    const stageTop = 10.0;
    const stageHeight = 60.0;
    final stageWidth = math.min(size.x * 0.45, 240.0);
    const skew = 24.0;

    _bgPath
      ..reset()
      ..moveTo(skew, stageTop)
      ..lineTo(stageWidth, stageTop)
      ..lineTo(stageWidth - skew, stageTop + stageHeight)
      ..lineTo(0, stageTop + stageHeight)
      ..close();

    _fillPaint.shader = const LinearGradient(
      colors: [Color(0xFF162037), Color(0xFF0D1426)],
      begin: Alignment.topLeft,
      end: Alignment.bottomRight,
    ).createShader(_bgPath.getBounds());

    _stageLabel = TextPainter(
      text: const TextSpan(
        text: 'STAGE',
        style: TextStyle(
          fontFamily: 'Montserrat',
          fontWeight: FontWeight.w600,
          fontSize: 13,
          color: Color(0xFF8EF9FF),
          letterSpacing: 2,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();

    _lifeStageValue = TextPainter(
      text: TextSpan(
        text: lifeStage.toUpperCase(),
        style: const TextStyle(
          fontFamily: 'Montserrat',
          fontWeight: FontWeight.w900,
          fontSize: 26,
          letterSpacing: 1.2,
          color: Colors.white,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();

    _ageLabel = TextPainter(
      text: const TextSpan(
        text: 'AGE',
        style: TextStyle(
          fontFamily: 'Montserrat',
          fontWeight: FontWeight.w600,
          fontSize: 12,
          color: Colors.black87,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();

    _ageValue = TextPainter(
      text: TextSpan(
        text: age.toString(),
        style: const TextStyle(
          fontFamily: 'Montserrat',
          fontWeight: FontWeight.w800,
          fontSize: 22,
          color: Colors.black,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
  }

  @override
  void render(Canvas canvas) {
    canvas.drawPath(_bgPath, _fillPaint);
    canvas.drawPath(_bgPath, _strokePaint);

    _stageLabel?.paint(canvas, const Offset(18, 16));
    _lifeStageValue?.paint(canvas, const Offset(18, 30));

    const hexRadius = 34.0;
    final hexCenter = Offset(size.x - 70, 40 + 4);

    final hexPath = Path();
    for (var i = 0; i < 6; i++) {
      final angle = math.pi / 3 * i - math.pi / 2;
      final x = hexCenter.dx + hexRadius * math.cos(angle);
      final y = hexCenter.dy + hexRadius * math.sin(angle);
      if (i == 0) {
        hexPath.moveTo(x, y);
      } else {
        hexPath.lineTo(x, y);
      }
    }
    hexPath.close();

    canvas.drawPath(
      hexPath,
      Paint()
        ..shader = const LinearGradient(
          colors: [Color(0xFF00D9FF), Color(0xFF7B5CFF)],
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
        ).createShader(hexPath.getBounds()),
    );

    canvas.drawPath(
      hexPath,
      Paint()
        ..style = PaintingStyle.stroke
        ..color = Colors.white
        ..strokeWidth = 2,
    );

    if (_ageLabel != null) {
      _ageLabel!.paint(
          canvas,
          Offset(hexCenter.dx - _ageLabel!.width / 2, hexCenter.dy - 22));
    }
    if (_ageValue != null) {
      _ageValue!.paint(
          canvas,
          Offset(hexCenter.dx - _ageValue!.width / 2, hexCenter.dy - 2));
    }
  }
}

class _EventTitleBanner extends PositionComponent {
  _EventTitleBanner({required this.title, required this.width})
      : super(size: Vector2(width, 82));

  final String title;
  final double width;
  final Path _path = Path();
  final Paint _fillPaint = Paint();
  final Paint _borderPaint = Paint()
    ..style = PaintingStyle.stroke
    ..strokeWidth = 3
    ..color = Colors.white;
  late final TextPainter _tp;

  @override
  Future<void> onLoad() async {
    const skew = 24.0;
    const notch = 32.0;
    final h = size.y;
    _path.reset();
    _path
      ..moveTo(0, h * 0.35)
      ..lineTo(skew, 0)
      ..lineTo(width - notch, 0)
      ..lineTo(width, h * 0.35)
      ..lineTo(width - skew, h)
      ..lineTo(notch * 0.4, h)
      ..close();

    _fillPaint.shader = const LinearGradient(
      colors: [Color(0xFF00D9FF), Color(0xFF7B5CFF)],
    ).createShader(size.toRect());

    _tp = TextPainter(
      text: TextSpan(
        text: title.toUpperCase(),
        style: SynTextStyles.h1Event.copyWith(fontSize: 30, shadows: [
          const Shadow(color: Colors.black, offset: Offset(2, 2), blurRadius: 0)
        ]),
      ),
      maxLines: 2,
      ellipsis: '…',
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: width - 40);
  }

  @override
  void render(Canvas canvas) {
    canvas.drawPath(_path, _fillPaint);
    canvas.drawPath(_path, _borderPaint);
    _tp.paint(canvas, Offset(20, size.y / 2 - _tp.height / 2));
  }
}

class _AccentDivider extends PositionComponent {
  _AccentDivider({required this.width}) : super(size: Vector2(width, 8));
  final double width;

  @override
  void render(Canvas canvas) {
    final path = Path()
      ..moveTo(0, size.y)
      ..lineTo(size.x * 0.7, 0)
      ..lineTo(size.x, 0)
      ..lineTo(size.x * 0.3, size.y)
      ..close();

    canvas.drawPath(
        path,
        Paint()
          ..shader = const LinearGradient(
            colors: [Color(0xFF00D9FF), Color(0xFF7B5CFF)],
          ).createShader(size.toRect()));
  }
}

class _TextPainterComponent extends PositionComponent {
  _TextPainterComponent({required this.painter})
      : super(size: Vector2(painter.width, painter.height));
  final TextPainter painter;
  @override
  void render(Canvas canvas) => painter.paint(canvas, Offset.zero);
}

class _EventTagChipRow extends PositionComponent {
  _EventTagChipRow({required this.tags, required this.maxWidth});
  final List<String> tags;
  final double maxWidth;

  double get computedHeight => 30.0;
}

class _ImpactSummaryRow extends PositionComponent {
  _ImpactSummaryRow({required this.stats, required this.maxWidth});
  final List<String> stats;
  final double maxWidth;
}

class _SlashTransition extends PositionComponent {
  _SlashTransition({required Vector2 size, required this.duration})
      : super(size: size);
  final double duration;
  double _t = 0;

  @override
  void update(double dt) {
    _t += dt;
    if (_t >= duration) removeFromParent();
  }

  @override
  void render(Canvas canvas) {
    final p = _t / duration;
    final x = size.x * (1.0 - p);
    canvas.drawLine(
        Offset(x, 0),
        Offset(x - 100, size.y),
        Paint()
          ..color = const Color(0xFF00D9FF).withValues(alpha: 1.0 - p)
          ..strokeWidth = 50);
  }
}
