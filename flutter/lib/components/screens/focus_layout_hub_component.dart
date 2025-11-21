import 'package:flame/components.dart';
import 'package:flame/effects.dart';
import 'package:flutter/material.dart';

import '../../models/game_state.dart';
import '../../syn_game.dart';
import '../ui/cards/event_card_component.dart';
import '../ui/panels/quick_panel_component.dart';
import '../ui/panels/relationship_panel_component.dart';
import '../ui/panels/stat_panel_component.dart';
import '../ui/panels/top_bar_component.dart';
import '../ui/system/background_layer_component.dart';

enum HubState {
  idle, // Standard gameplay: Event Card is hero
  statFocus, // Stats expanded
  socialFocus // Relationships expanded
}

class FocusLayoutHubComponent extends PositionComponent
    with HasGameReference<SynGame> {
  // -- Child Components --
  late final BackgroundLayerComponent _background;
  late final TopBarComponent _topBar;
  late final StatPanelComponent _statPanel;
  late final RelationshipPanelComponent _relationshipPanel;
  late final QuickMenuBarComponent _quickMenu;
  late final EventCardComponent _eventCard;

  // -- Layout State --
  HubState _state = HubState.idle;

  // Animation Config - "Snappy & Aggressive"
  static const double _morphDuration = 0.45;
  static const Curve _morphCurve = Curves.easeInOutQuart;

  // Safe Zone for Focused Content (90% of screen center)
  late Rect _focusRect;

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    size = game.size.clone();
    _updateFocusRect();

    // 1. Initialize Children (Using your extracted classes)
    _background = BackgroundLayerComponent()..priority = 0;

    _topBar = TopBarComponent(
      gameState: game.gameState,
      onStats: () => _switchState(HubState.statFocus), // Hook up the button!
      onSettings: game.showSettings,
      onSave: game.showSaveLoad,
      onPause: game.togglePauseOverlay,
      onNotifications: () {},
    )..priority = 20;

    // Note: Pass callbacks to handle "Back" actions from panels
    _statPanel = StatPanelComponent(
      gameState: game.gameState,
      onClose: () => _switchState(HubState.idle),
    )..priority = 15;

    _relationshipPanel = RelationshipPanelComponent(
      relationships: game.gameState.relationships,
      onOpenNetwork: () => _switchState(HubState.socialFocus),
    )..priority = 15;

    _quickMenu = QuickMenuBarComponent(
      onMemory: game.showMemoryJournal,
      onMap: game.showWorldMap,
      onPossessions: game.showPossessions,
    )..priority = 15;
    _quickMenu.size = Vector2(size.x * 0.68, 80);

    _eventCard = EventCardComponent(
      event: game.gameState.currentEvent ??
          _placeholderEvent(), // Placeholder until backend hook-up
      onChoice: (i) => {}, // TODO: wire choice handling when backend is ready
    )..priority = 10;

    addAll([
      _background,
      _topBar,
      _statPanel,
      _relationshipPanel,
      _quickMenu,
      _eventCard
    ]);

    // 2. Initial Layout (Instant)
    _applyLayoutState(_state, animated: false);
  }

  @override
  void onGameResize(Vector2 newSize) {
    super.onGameResize(newSize);
    size = newSize;
    _background.size = newSize;
    _topBar.size = Vector2(newSize.x, 92.0); // Use SynTopBar.height
    _updateFocusRect();
    // Re-apply current state without animation on resize to keep positions correct
    _applyLayoutState(_state, animated: false);
  }

  void _updateFocusRect() {
    // The "Stage" where focused panels go
    _focusRect = Rect.fromCenter(
      center:
          size.toOffset() / 2 + const Offset(0, 40), // Slight offset for TopBar
      width: size.x * 0.85,
      height: size.y * 0.75,
    );
  }

  void _switchState(HubState newState) {
    if (_state == newState) return;
    _state = newState;
    _applyLayoutState(newState, animated: true);
  }

  void _applyLayoutState(HubState state, {required bool animated}) {
    // Define specific layouts for each state
    switch (state) {
      case HubState.idle:
        _layoutIdle(animated);
        break;
      case HubState.statFocus:
        _layoutStatFocus(animated);
        break;
      case HubState.socialFocus:
        _layoutSocialFocus(animated);
        break;
    }
  }

  // --- LAYOUT DEFINITIONS ---

  void _layoutIdle(bool animated) {
    final w = size.x;
    final h = size.y;

    // 1. Event Card: Center Stage
    _morph(_eventCard,
        pos: Vector2((w - 600) / 2, h * 0.25),
        scale: 1.0,
        opacity: 1.0,
        animated: animated);

    // 2. Stat Panel: Compact, Left Edge (Peeking in)
    _morph(_statPanel,
        pos: Vector2(w * 0.05, h * 0.25),
        scale: 0.85,
        opacity: 1.0,
        animated: animated);

    // 3. Relationship Panel: Compact, Right Edge
    _morph(_relationshipPanel,
        pos: Vector2(w - (w * 0.25) - (w * 0.05), h * 0.25),
        scale: 0.85,
        opacity: 1.0,
        animated: animated);

    // 4. Quick Menu: Bottom Center
    _morph(_quickMenu,
        pos: Vector2((w - _quickMenu.width) / 2, h - 100),
        scale: 1.0,
        opacity: 1.0,
        animated: animated);
  }

  void _layoutStatFocus(bool animated) {
    // 1. Event Card: Slashes off-screen to the bottom-right
    _morph(_eventCard,
        pos: Vector2(size.x + 100, size.y),
        scale: 1.2,
        opacity: 0.0,
        animated: animated);

    // 2. Stat Panel: Takes Center Stage
    _morph(_statPanel,
        pos: Vector2(_focusRect.left, _focusRect.top),
        scale: 1.0,
        opacity: 1.0,
        animated: animated);

    // 3. Others: Fade out / retreat
    _morph(_relationshipPanel,
        pos: Vector2(size.x + 200, size.y * 0.25),
        scale: 0.8,
        opacity: 0.0,
        animated: animated);
    _morph(_quickMenu,
        pos: Vector2(size.x / 2, size.y + 200),
        scale: 1.0,
        opacity: 0.0,
        animated: animated);
  }

  void _layoutSocialFocus(bool animated) {
    // Similar logic for Social Focus...
    // Stat panel moves left/fades, Relationship panel moves to center
  }

  // --- THE TWEEN ENGINE ---

  void _morph(PositionComponent c,
      {required Vector2 pos,
      double scale = 1.0,
      double opacity = 1.0,
      required bool animated}) {
    c.removeWhere((component) => component is Effect); // Clear old animations

    if (!animated) {
      c.position = pos;
      c.scale = Vector2.all(scale);
      if (c is OpacityProvider) {
        (c as OpacityProvider).opacity = opacity;
      }
      return;
    }

    c.add(MoveEffect.to(
        pos, EffectController(duration: _morphDuration, curve: _morphCurve)));
    c.add(ScaleEffect.to(Vector2.all(scale),
        EffectController(duration: _morphDuration, curve: _morphCurve)));
    if (c is OpacityProvider) {
      c.add(OpacityEffect.to(opacity,
          EffectController(duration: _morphDuration, curve: _morphCurve)));
    }
  }

  // Placeholder helper
  GameEvent _placeholderEvent() => GameEvent(
      id: 'xx',
      title: 'LOADING',
      description: '...',
      choices: [],
      lifeStage: 'Child',
      age: 6);
}
