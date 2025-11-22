import 'package:flame/components.dart';
import 'package:flame/effects.dart';
import 'package:flutter/material.dart';

import '../../../models/game_state.dart';
import '../../../syn_game.dart';
import '../buttons/icon_button_component.dart';
import '../display/age_counter_component.dart';
import '../display/life_stage_badge_component.dart';
import '../display/mood_pulse_component.dart';
import '../syn_theme.dart';

const double _kTopBarRiseFactor = 0.52;
const double _kTopBarBottomBendFactor = 0.74;
const double _kTopBarNotchDepth = 24.0;
const double _kTopBarSlashDepthFactor = 0.58;
const double _kTopBarSlashPrimaryStart = 0.035;
const double _kTopBarSlashPrimaryEnd = 0.20;
const double _kTopBarSlashPrimaryTaper = 0.05;
const double _kTopBarSlashPrimaryOverhang = 0.018;
const double _kTopBarSlashSecondaryStart = 0.14;
const double _kTopBarSlashSecondaryEnd = 0.32;
const double _kTopBarSlashSecondaryTaper = 0.055;
const double _kTopBarSlashSecondaryOverhang = 0.020;

const double _kTopBarClusterGap = 16.0;
const double _kTopBarLeftPaddingFactor = 0.04;
const double _kTopBarCenterYFactor = 0.55;
const double _kLifeStageWidthFactor = 0.15;
const double _kLifeStageHeightFactor = 0.62;
const double _kAgeWidthFactor = 0.105;
const double _kAgeHeightFactor = 0.52;
const double _kMoodSizeFactor = 0.50;
const double _kActionHitSizeFactor = 0.62;
const double _kActionSpacing = 20.0;
const double _kActionRightPaddingFactor = 0.04;
const double _kFlashFadePerSecond = 2.8;

enum _ActionVisualState { idle, hover, active }

class TopBarComponent extends PositionComponent with HasGameReference<SynGame> {
  TopBarComponent({
    required this.gameState,
    required this.onStats,
    required this.onSettings,
    required this.onSave,
    required this.onPause,
    required this.onNotifications,
    super.position,
    super.size,
  });

  final GameState gameState;
  final VoidCallback onStats;
  final VoidCallback onSettings;
  final VoidCallback onSave;
  final VoidCallback onPause;
  final VoidCallback onNotifications;

  late final LifeStageBadgeComponent _lifeStageBadge;
  late final AgeCounterComponent _ageCounter;
  late final MoodPulseComponent _moodPulse;
  late final List<(IconButtonComponent, TextComponent)> _actionButtons;
  late final TextPaint _primaryActionPaint;
  late final TextPaint _hoverActionPaint;
  late final TextPaint _activeActionPaint;
  double _flashAlpha = 0;
  String _lastStage = '';
  double _lastMood = 0;

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    _lastStage = gameState.lifeStage;
    _lastMood = gameState.mood.toDouble();
    _primaryActionPaint = TextPaint(style: SynTopBar.textPrimaryStyle);
    _hoverActionPaint = TextPaint(
      style: SynTopBar.textPrimaryStyle.copyWith(
        color: SynTopBar.textHoverColor,
        shadows: [
          Shadow(
            color: SynTopBar.textHoverColor.withValues(alpha: 0.25),
            blurRadius: SynTopBar.textGlowBlur,
          ),
        ],
      ),
    );
    _activeActionPaint = TextPaint(
      style: SynTopBar.textPrimaryStyle.copyWith(
        color: SynTopBar.textActiveColor,
        shadows: [
          Shadow(
            color: SynTopBar.textActiveColor.withValues(alpha: 0.25),
            blurRadius: SynTopBar.textGlowBlur,
          ),
        ],
      ),
    );

    _lifeStageBadge = LifeStageBadgeComponent()..anchor = Anchor.center;
    _ageCounter = AgeCounterComponent()..anchor = Anchor.center;
    _moodPulse = MoodPulseComponent()..anchor = Anchor.center;

    addAll([
      _lifeStageBadge,
      _ageCounter,
      _moodPulse,
    ]);

    final actions = [
      ('STATS', onStats),
      ('SAVE', onSave),
      ('CFG', onSettings),
      ('LOG', onNotifications),
      ('PAUSE', onPause),
    ];
    _actionButtons = [];
    for (final entry in actions) {
      final label = TextComponent(
        text: entry.$1,
        anchor: Anchor.center,
        textRenderer: _primaryActionPaint,
      );
      late IconButtonComponent button;
      button = IconButtonComponent(
        onTap: entry.$2,
        onHoverChanged: (hovering) => _updateActionLabelState(
          button,
          label,
          isHovered: hovering,
          isPressed: false,
        ),
        onPressChanged: (pressed) => _updateActionLabelState(
          button,
          label,
          isHovered: button.isHovered,
          isPressed: pressed,
        ),
        hoverScale: SynTopBar.actionHoverScale,
        hoverDuration: SynTopBar.actionHoverDuration,
        baseColor: const Color(0x00000000),
        hoverColor: const Color(0x00000000),
        pressedColor: const Color(0x00000000),
      )..anchor = Anchor.center;
      button.add(label);
      _setActionState(label, _ActionVisualState.idle);
      _actionButtons.add((button, label));
      add(button);
    }

    _layoutChildren();
  }

  @override
  void render(Canvas canvas) {
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    final slashDepth = size.y * _kTopBarSlashDepthFactor;
    final path = Path()
      ..moveTo(0, 0)
      ..lineTo(size.x, 0)
      ..lineTo(size.x, size.y * _kTopBarBottomBendFactor)
      ..lineTo(size.x - _kTopBarNotchDepth, size.y)
      ..lineTo(_kTopBarNotchDepth, size.y)
      ..lineTo(0, size.y * _kTopBarRiseFactor)
      ..close();

    canvas.drawShadow(
      path,
      SynTopBar.shadowColor,
      SynTopBar.shadowBlur,
      false,
    );
    canvas.drawPath(
      path,
      Paint()
        ..shader = LinearGradient(
          colors: [
            SynTopBar.backgroundColor,
            SynTopBar.backgroundSheenColor,
          ],
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
        ).createShader(rect),
    );

    final slash1 = Path()
      ..moveTo(size.x * _kTopBarSlashPrimaryStart, 0)
      ..lineTo(size.x * _kTopBarSlashPrimaryEnd, 0)
      ..lineTo(
        size.x * (_kTopBarSlashPrimaryEnd - _kTopBarSlashPrimaryTaper),
        slashDepth,
      )
      ..lineTo(
        size.x * (_kTopBarSlashPrimaryStart - _kTopBarSlashPrimaryOverhang),
        slashDepth,
      )
      ..close();
    canvas.drawPath(
      slash1,
      Paint()
        ..color = SynTopBar.slashOverlayColor.withValues(
          alpha: SynTopBar.slashOverlayOpacity,
        ),
    );

    final slash2 = Path()
      ..moveTo(size.x * _kTopBarSlashSecondaryStart, 0)
      ..lineTo(size.x * _kTopBarSlashSecondaryEnd, 0)
      ..lineTo(
        size.x * (_kTopBarSlashSecondaryEnd - _kTopBarSlashSecondaryTaper),
        slashDepth,
      )
      ..lineTo(
        size.x * (_kTopBarSlashSecondaryStart - _kTopBarSlashSecondaryOverhang),
        slashDepth,
      )
      ..close();
    canvas.drawPath(
      slash2,
      Paint()
        ..color = SynTopBar.slashOverlayColor.withValues(
          alpha: SynTopBar.slashOverlayOpacity * 0.8,
        ),
    );

    if (_flashAlpha > 0) {
      canvas.drawPath(
        path,
        Paint()
          ..color = SynTopBar.ambientGlowColor.withValues(alpha: _flashAlpha)
          ..maskFilter = MaskFilter.blur(
            BlurStyle.normal,
            SynTopBar.stageFlashBlur,
          ),
      );
    }
  }

  @override
  void onGameResize(Vector2 size) {
    super.onGameResize(size);
    _layoutChildren();
  }

  @override
  void update(double dt) {
    super.update(dt);
    if (_ageCounter.displayedAge.toDouble() != gameState.age.toDouble()) {
      _ageCounter.displayedAge = gameState.age;
      _ageCounter.add(
        ScaleEffect.to(
          Vector2.all(1.08),
          EffectController(duration: 0.15, reverseDuration: 0.15),
        ),
      );
    }

    if (_lastStage != gameState.lifeStage) {
      _lastStage = gameState.lifeStage;
      _flashBorder();
    }

    if (_lastMood != gameState.mood.toDouble()) {
      _lastMood = gameState.mood.toDouble();
      _moodPulse.mood = _normalizeMood(gameState.mood);
      _moodPulse.add(
        ScaleEffect.to(
          Vector2.all(1.05),
          EffectController(duration: 0.12, reverseDuration: 0.12),
        ),
      );
    }

    if (_flashAlpha > 0) {
      _flashAlpha =
          (_flashAlpha - dt * _kFlashFadePerSecond).clamp(0.0, 1.0);
    }
  }

  void _layoutChildren() {
    final w = size.x;
    final h = size.y;
    final centerY = h * _kTopBarCenterYFactor;
    const gap = _kTopBarClusterGap;

    _lifeStageBadge
      ..size = Vector2(w * _kLifeStageWidthFactor, h * _kLifeStageHeightFactor)
      ..position = Vector2(
        w * _kTopBarLeftPaddingFactor + _lifeStageBadge.size.x / 2,
        centerY,
      );

    _ageCounter
      ..size = Vector2(w * _kAgeWidthFactor, h * _kAgeHeightFactor)
      ..position = _lifeStageBadge.position +
          Vector2(_lifeStageBadge.size.x / 2 + gap + _ageCounter.size.x / 2, 0);

    _moodPulse
      ..size = Vector2(h * _kMoodSizeFactor, h * _kMoodSizeFactor)
      ..position = _ageCounter.position +
          Vector2(_ageCounter.size.x / 2 + gap + _moodPulse.size.x / 2, 0);

    final buttonSize =
        Vector2(h * _kActionHitSizeFactor, h * _kActionHitSizeFactor);
    final spacing = _kActionSpacing;
    final rightPadding = w * _kActionRightPaddingFactor;
    double cursorX = w - buttonSize.x / 2 - rightPadding;
    for (final tuple in _actionButtons) {
      final button = tuple.$1;
      final label = tuple.$2;
      button
        ..size = buttonSize
        ..position = Vector2(cursorX, centerY);
      label.position = button.size / 2;
      cursorX -= (buttonSize.x + spacing);
    }
  }

  void _updateActionLabelState(
    IconButtonComponent button,
    TextComponent label, {
    required bool isHovered,
    required bool isPressed,
  }) {
    var state = _ActionVisualState.idle;
    if (isPressed) {
      state = _ActionVisualState.active;
    } else if (isHovered) {
      state = _ActionVisualState.hover;
    }
    _setActionState(label, state);
  }

  void _setActionState(TextComponent label, _ActionVisualState state) {
    switch (state) {
      case _ActionVisualState.idle:
        label.textRenderer = _primaryActionPaint;
        label.scale = Vector2.all(1);
        break;
      case _ActionVisualState.hover:
        label.textRenderer = _hoverActionPaint;
        label.scale = Vector2.all(1);
        break;
      case _ActionVisualState.active:
        label.textRenderer = _activeActionPaint;
        label.scale = Vector2.all(SynTopBar.actionPressScale);
        break;
    }
  }

  void _flashBorder() {
    _flashAlpha = SynTopBar.stageFlashOpacity;
  }

  double _normalizeMood(num mood) => mood.clamp(-10, 10) / 20 + 0.5;
}
