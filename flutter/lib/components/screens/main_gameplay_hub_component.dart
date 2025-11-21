import 'package:flame/components.dart';
import 'package:flame/effects.dart';
import 'package:flutter/material.dart';
import 'dart:math';

import '../../models/game_state.dart';
import '../../syn_game.dart';
import '../ui/buttons/icon_button_component.dart';
import '../ui/buttons/text_button_component.dart';
import '../ui/cards/event_card_component.dart';
import '../ui/cards/npc_card_component.dart';
import '../ui/display/age_counter_component.dart';
import '../ui/display/life_stage_badge_component.dart';
import '../ui/display/mood_pulse_component.dart';
import '../ui/display/stat_bar_component.dart';
import '../ui/syn_theme.dart';
import '../ui/system/background_layer_component.dart';
import '../ui/system/notification_queue_component.dart';

/// Main gameplay hub: a layout compositor that positions existing UI components.
class MainGameplayHubComponent extends PositionComponent
    with HasGameReference<SynGame> {
  late final BackgroundLayerComponent _background;
  late final TopBarComponent _topBar;
  late final StatPanelComponent _statPanel;
  late final RelationshipPanelComponent _relationshipPanel;
  late final QuickMenuBarComponent _quickMenu;
  late final EventCardComponent _eventCard;
  late final NotificationQueueComponent _notificationQueue;

  GameEvent get _activeEvent =>
      game.gameState.currentEvent ?? _placeholderEvent;

  GameEvent _buildEventPlaceholder(GameState state) {
    final tags = ['NIGHT', 'RADIO', 'FOG'];
    return GameEvent(
      id: 'signal-in-the-fog',
      title: 'Signal in the Fog',
      description:
          'A flickering broadcast spills through the neon mist. Trace it, boost it, or jam it — each move shapes the mood of the city.',
      lifeStage: state.lifeStage.toUpperCase(),
      age: state.age,
      tags: tags,
      deltas: const {'health': 2, 'stability': -1},
      choices: [
        GameChoice(
          text: 'TRACE THE SOURCE',
          statChanges: const {'health': -2, 'stability': 2},
          keyboardShortcut: 1,
        ),
        GameChoice(
          text: 'BOOST THE SIGNAL',
          statChanges: const {'wealth': 1, 'charisma': 1, 'stability': -1},
          keyboardShortcut: 2,
        ),
        GameChoice(
          text: 'JAM THE FREQUENCY',
          statChanges: const {'stability': 2, 'intelligence': -1},
          keyboardShortcut: 3,
        ),
      ],
    );
  }

  GameEvent get _placeholderEvent => _buildEventPlaceholder(game.gameState);

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    size = game.size.clone();

    _background = BackgroundLayerComponent()
      ..position = Vector2.zero()
      ..size = size.clone()
      ..priority = 0;

    _notificationQueue = NotificationQueueComponent(
      initialMessages: const [
        '+10 TRUST — Kaz backs your move',
        'HEALTH +2 — Efficient recovery',
        'STABILITY -1 — Static lingers',
      ],
    )..priority = 30;

    _topBar = TopBarComponent(
      gameState: game.gameState,
      onStats: game.showDetailedStats,
      onSettings: game.showSettings,
      onSave: game.showSaveLoad,
      onPause: game.togglePauseOverlay,
      onNotifications: _notificationQueue.toggleHistory,
    )..priority = 10;

    _statPanel = StatPanelComponent(gameState: game.gameState)..priority = 10;

    _relationshipPanel = RelationshipPanelComponent(
      relationships: game.gameState.relationships,
      onOpenNetwork: game.showRelationshipNetwork,
    )..priority = 10;

    _quickMenu = QuickMenuBarComponent(
      onMemory: game.showMemoryJournal,
      onMap: game.showWorldMap,
      onPossessions: game.showPossessions,
    )..priority = 12;

    _eventCard = EventCardComponent(
      event: _activeEvent,
      onChoice: _handleChoice,
    )..priority = 20;

    _applyLayout();

    addAll([
      _background,
      _topBar,
      _statPanel,
      _relationshipPanel,
      _quickMenu,
      _eventCard,
      _notificationQueue,
    ]);
  }

  @override
  void onGameResize(Vector2 newSize) {
    super.onGameResize(newSize);
    size = newSize;
    _applyLayout();
  }

  void _applyLayout() {
    final w = size.x;
    final h = size.y;
    final topBarHeight = (h * SynTopBar.heightFraction)
        .clamp(SynTopBar.height * 0.85, SynTopBar.height * 1.15);
    final marginX = w * 0.06;
    final marginBottom = h * 0.06;

    final statPanelWidth = w * 0.20;
    final sidePanelHeight = h * 0.52;
    final eventWidth = w * 0.56;
    final eventHeight = h * 0.58;
    final quickBarHeight = h * 0.10;
    final quickWidth = eventWidth + w * 0.06;
    final sidePanelY = topBarHeight + h * 0.08;
    final eventY = topBarHeight + h * 0.05;
    final quickY = h - quickBarHeight - marginBottom;

    _background.size = size.clone();

    _topBar
      ..position = Vector2.zero()
      ..size = Vector2(w, topBarHeight);

    _statPanel
      ..position = Vector2(marginX, sidePanelY)
      ..size = Vector2(statPanelWidth, sidePanelHeight);

    _relationshipPanel
      ..position = Vector2(w - marginX - statPanelWidth, sidePanelY)
      ..size = Vector2(statPanelWidth, sidePanelHeight);

    _eventCard
      ..position = Vector2((w - eventWidth) / 2, eventY)
      ..size = Vector2(eventWidth, eventHeight);

    _quickMenu
      ..position = Vector2((w - quickWidth) / 2, quickY)
      ..size = Vector2(quickWidth, quickBarHeight);

    _notificationQueue.position = Vector2(
      w - marginX - 260,
      topBarHeight + h * 0.02,
    );
  }

  void _handleChoice(int index) {
    final choice = _activeEvent.choices[index];
    choice.statChanges.forEach(game.gameState.updateStat);
    _notificationQueue.addMessage(
      'CHOICE ${index + 1}: ${choice.text.toUpperCase()}',
    );
  }
}

class GameplayHub extends MainGameplayHubComponent {}

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

class StatPanelComponent extends PositionComponent
    with HasGameReference<SynGame> {
  StatPanelComponent({required this.gameState, super.position, super.size});

  final GameState gameState;

  // Visual polish constants, derived from syn_theme.dart
  static final _backgroundGradient = LinearGradient(
    colors: [SynColors.bgDark, SynColors.bgPanel],
    begin: Alignment.topCenter,
    end: Alignment.bottomCenter,
  );
  static const _borderColor = SynHudChrome.topBarBorderColorPrimary;
  static const _slashColor = SynHudChrome.topBarBorderColorPrimary;
  static const _borderWidth = SynLayout.borderWidthHeavy;

  @override
  Future<void> onLoad() async {
    await super.onLoad();

    final stats = _buildStats();
    var y = 24.0; // Increased top padding
    for (final stat in stats) {
      final row = _StatRow(
        label: stat.$1,
        value: stat.$2,
        rawValue: stat.$3,
      )
        ..position = Vector2(20, y) // Increased horizontal padding
        ..size = Vector2(size.x - 40, 52); // Adjusted height for spacing
      add(row);
      y += 58; // Adjusted vertical stride
    }
  }

  List<(String, double, int)> _buildStats() {
    return [
      ('HEALTH', gameState.health / 100, gameState.health),
      ('WEALTH', gameState.wealth / 100, gameState.wealth),
      ('CHARISMA', gameState.charisma / 100, gameState.charisma),
      ('INTELLECT', gameState.intelligence / 100, gameState.intelligence),
      ('STABILITY', gameState.stability / 100, gameState.stability),
      ('CREATIVITY', gameState.wisdom / 100, gameState.wisdom),
    ];
  }

  @override
  void render(Canvas canvas) {
    final path = Path()
      ..moveTo(16, 0)
      ..lineTo(size.x - 10, 6)
      ..lineTo(size.x, size.y * 0.25)
      ..lineTo(size.x, size.y - 10)
      ..lineTo(size.x - 16, size.y)
      ..lineTo(10, size.y - 6)
      ..lineTo(0, size.y * 0.15)
      ..close();

    canvas.drawShadow(path, SynHudChrome.topBarShadowColor, 12, false);
    canvas.drawPath(
      path,
      Paint()..shader = _backgroundGradient.createShader(size.toRect()),
    );
    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = _borderWidth
        ..color = _borderColor,
    );

    // Subtler interior slashes
    final slash1 = Path()
      ..moveTo(size.x * 0.70, -10)
      ..lineTo(size.x * 0.9, size.y * 0.3)
      ..lineTo(size.x * 0.76, size.y * 0.3 + 18)
      ..close();
    canvas.drawPath(
      slash1,
      Paint()
        ..color = _slashColor.withOpacity(0.14)
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 3),
    );

    final slash2 = Path()
      ..moveTo(size.x * 0.12, size.y * 0.12)
      ..lineTo(size.x * 0.3, size.y * 0.45)
      ..lineTo(size.x * 0.18, size.y * 0.45 + 16)
      ..close();
    canvas.drawPath(
      slash2,
      Paint()
        ..color = _slashColor.withOpacity(0.08)
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 5),
    );
  }
}

class _StatRow extends PositionComponent {
  _StatRow({
    required this.label,
    required this.value,
    required this.rawValue,
  });

  final String label;
  final double value;
  final int rawValue;

  @override
  Future<void> onLoad() async {
    await super.onLoad();

    // Polished stat label
    final labelText = TextComponent(
      text: label,
      textRenderer: TextPaint(
        style: SynTextStyles.h2Strip.copyWith(
          fontSize: 13,
          color: const Color(0xFFE5ECF5),
          letterSpacing: 1.8,
        ),
      ),
      position: Vector2(0, 0),
    );
    add(labelText);

    final bar = StatBarComponent(
      value: value,
      position: Vector2(0, 24),
      size: Vector2(size.x, 10),
    );
    add(bar);

    // Right-aligned numeric value
    final numeric = TextComponent(
      text: rawValue.toString(),
      textRenderer: TextPaint(
        style: SynTextStyles.body.copyWith(
          color: SynColors.textSubtle,
          fontSize: 14,
          fontWeight: FontWeight.w600,
        ),
      ),
      anchor: Anchor.topRight,
      position: Vector2(size.x, 0),
    );
    add(numeric);
  }
}

class RelationshipPanelComponent extends PositionComponent
    with HasGameReference<SynGame> {
  RelationshipPanelComponent({
    required this.relationships,
    required this.onOpenNetwork,
    super.position,
    super.size,
  });

  final List<RelationshipData> relationships;
  final VoidCallback onOpenNetwork;

  // Visual polish constants
  static final _backgroundGradient = LinearGradient(
    colors: [
      const Color(0xFF0A081A), // Dark blue/purple
      const Color(0xFF180F2A), // Dark magenta/indigo
    ],
    begin: Alignment.topCenter,
    end: Alignment.bottomCenter,
  );
  static const _borderColor = Color(0xFFFF4A9B); // Bright magenta/pink
  static const _slashColor = Color(0xFFFF4A9B);
  static const _borderWidth = SynLayout.borderWidthHeavy;

  // Layout constants
  static const double _topMargin = 24.0;
  static const double _bottomMargin = 24.0;
  static const double _horizontalPadding = 16.0;
  static const double _cardHeight = 78.0;
  static const double _minSpacing = 12.0;
  static const int _maxCards = 4;

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    _updateCardLayout();
  }

  @override
  void onGameResize(Vector2 newSize) {
    super.onGameResize(newSize);
    _updateCardLayout();
  }

  void _updateCardLayout() {
    // Clear previous cards to prevent duplicates on resize
    removeAll(children.whereType<NPCCardComponent>());

    final entries =
        relationships.isNotEmpty ? relationships : _placeholderRelationships();
    final cardCount = min(entries.length, _maxCards);
    if (cardCount == 0) {
      return;
    }

    final availableHeight = size.y - _topMargin - _bottomMargin;
    final cardWidth = size.x - 2 * _horizontalPadding;

    double y;
    final double spacing;

    if (cardCount > 1) {
      final totalCardHeight = cardCount * _cardHeight;
      spacing = max(
        _minSpacing,
        (availableHeight - totalCardHeight) / (cardCount - 1),
      );
      y = _topMargin;
    } else {
      // Vertically center a single card
      spacing = 0;
      y = _topMargin + (availableHeight - _cardHeight) / 2;
    }

    // Add the cards with the new layout
    for (var i = 0; i < cardCount; i++) {
      final rel = entries[i];
      final card = NPCCardComponent(
        relationship: rel,
        onTap: onOpenNetwork,
        position: Vector2(_horizontalPadding, y),
        size: Vector2(cardWidth, _cardHeight),
      );
      add(card);
      y += _cardHeight + spacing;
    }
  }

  List<RelationshipData> _placeholderRelationships() {
    return [
      RelationshipData(
        npcId: 'kaz',
        npcName: 'Kaz',
        affection: 4,
        trust: 6,
        attraction: 3,
        familiarity: 6,
        resentment: 2,
        state: 'Ally',
      ),
      RelationshipData(
        npcId: 'ila',
        npcName: 'Ila',
        affection: 2,
        trust: 5,
        attraction: 1,
        familiarity: 4,
        resentment: 3,
        state: 'Confidant',
      ),
      RelationshipData(
        npcId: 'fixer',
        npcName: 'Fixer',
        affection: 1,
        trust: 4,
        attraction: 0,
        familiarity: 5,
        resentment: 5,
        state: 'Contact',
      ),
    ];
  }

  @override
  void render(Canvas canvas) {
    // Angled silhouette, matching StatPanel's general shape but mirrored
    final path = Path()
      ..moveTo(10, 6)
      ..lineTo(size.x, 0)
      ..lineTo(size.x, size.y - 10)
      ..lineTo(size.x - 10, size.y - 6)
      ..lineTo(0, size.y)
      ..lineTo(0, size.y * 0.25)
      ..close();

    canvas.drawShadow(path, SynHudChrome.topBarShadowColor, 12, false);
    canvas.drawPath(
      path,
      Paint()..shader = _backgroundGradient.createShader(size.toRect()),
    );
    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = _borderWidth
        ..color = _borderColor,
    );

    // Mirrored interior slash
    final slash = Path()
      ..moveTo(size.x * 0.18, -8)
      ..lineTo(size.x * 0.32, size.y * 0.45)
      ..lineTo(size.x * 0.2, size.y * 0.45 + 14)
      ..close();
    canvas.drawPath(
      slash,
      Paint()
        ..color = _slashColor.withOpacity(0.12)
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 4),
    );
  }
}

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
