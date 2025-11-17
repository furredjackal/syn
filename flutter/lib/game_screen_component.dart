import 'package:flame/components.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'models/game_state.dart';
import 'syn_game.dart';
import 'widgets/event_card_component.dart';
import 'widgets/persona_background.dart';
import 'widgets/quick_menu_bar_component.dart';
import 'widgets/relationship_panel_component.dart';
import 'widgets/stat_panel_component.dart';
import 'widgets/top_bar_component.dart';

/// Main game screen component using Flame components as floating canvas.
///
/// Floating Persona UI model:
/// - EventCard: Centered focal point (max 60% width, angled Persona style)
/// - StatPanel: Floats left (compact ~250-300px, minimal footprint)
/// - RelationshipPanel: Floats right (compact ~250-300px, minimal footprint)
/// - TopBar: Thin strip at top
/// - QuickMenuBar: Thin strip at bottom
///
/// NO rigid layout frames. Each component defines its own position and shape.
/// Follows floating canvas model from design docs.
class GameScreenComponent extends PositionComponent
    with HasGameReference<SynGame>, KeyboardHandler {
  static final Map<int, Set<LogicalKeyboardKey>> _shortcutKeyMap = {
    0: {LogicalKeyboardKey.digit0, LogicalKeyboardKey.numpad0},
    1: {LogicalKeyboardKey.digit1, LogicalKeyboardKey.numpad1},
    2: {LogicalKeyboardKey.digit2, LogicalKeyboardKey.numpad2},
    3: {LogicalKeyboardKey.digit3, LogicalKeyboardKey.numpad3},
    4: {LogicalKeyboardKey.digit4, LogicalKeyboardKey.numpad4},
    5: {LogicalKeyboardKey.digit5, LogicalKeyboardKey.numpad5},
    6: {LogicalKeyboardKey.digit6, LogicalKeyboardKey.numpad6},
    7: {LogicalKeyboardKey.digit7, LogicalKeyboardKey.numpad7},
    8: {LogicalKeyboardKey.digit8, LogicalKeyboardKey.numpad8},
    9: {LogicalKeyboardKey.digit9, LogicalKeyboardKey.numpad9},
  };

  late final PersonaBackground _background;
  late final PositionComponent _componentLayer;

  late TopBarComponent topBar;
  late StatPanelComponent statPanel;
  late RelationshipPanelComponent relationshipPanel;
  late QuickMenuBarComponent quickMenuBar;

  EventCardComponent? currentEventCard;

  @override
  Future<void> onLoad() async {
    size = game.size;

    // Background (Persona-style canvas)
    _background = PersonaBackground()..size = size;
    add(_background);

    // Single component layer (no rigid frame structure)
    _componentLayer = PositionComponent()..size = size;
    add(_componentLayer);

    const margin = _margin;
    topBar = TopBarComponent()
      ..position = Vector2.zero()
      ..size = Vector2(size.x, _topBarHeight);
    _componentLayer.add(topBar);

    final leftWidth = _leftWidth;
    final centerWidth = _centerWidth;
    final rightWidth = _rightWidth;

    statPanel = StatPanelComponent()
      ..position = Vector2(margin, _topBarHeight + margin)
      ..size = Vector2(leftWidth - margin * 1.5, _contentHeight);
    _componentLayer.add(statPanel);

    relationshipPanel = RelationshipPanelComponent()
      ..position = Vector2(
        leftWidth + centerWidth + margin * 0.5,
        _topBarHeight + margin,
      )
      ..size = Vector2(rightWidth - margin * 1.5, _contentHeight);
    _componentLayer.add(relationshipPanel);

    quickMenuBar = QuickMenuBarComponent()
      ..position = Vector2(margin, size.y - _quickMenuHeight - margin)
      ..size = Vector2(size.x - margin * 2, _quickMenuHeight);
    _componentLayer.add(quickMenuBar);

    final initialCardSize = _eventCardSize();
    final eventCardPosition = _eventCardPosition();

    // Create a placeholder event for initial display
    final initialEvent = GameEvent(
      id: 'placeholder',
      title: 'Loading...',
      description: '',
      choices: [],
      lifeStage: 'Loading',
      age: 0,
    );

    final eventCard = EventCardComponent(
      event: initialEvent,
      onChoice: _handleChoice,
      position: eventCardPosition,
      size: initialCardSize,
    );
    _componentLayer.add(eventCard);
    currentEventCard = eventCard;

    // Load and display initial event
    _loadNextEvent();
  }

  /// Load the next event and display it in the center panel
  void _loadNextEvent() {
    // Demo event - in production this would come from Rust backend
    game.gameState.setCurrentEvent(
      GameEvent(
        id: 'demo_001',
        title: 'A New Beginning',
        description:
            'You wake up on your first day of school. Your parents have prepared your lunch.',
        choices: [
          GameChoice(
            text: 'Eat breakfast',
            statChanges: {'health': 10},
            keyboardShortcut: 1,
          ),
          GameChoice(
            text: 'Skip breakfast',
            statChanges: {'health': -5},
            keyboardShortcut: 2,
          ),
        ],
        lifeStage: 'Child',
        age: 6,
      ),
    );
    _showEvent();
  }

  /// Display the current event card in the center of the screen
  void _showEvent() {
    currentEventCard?.removeFromParent();

    final event = game.gameState.currentEvent;
    if (event != null) {
      final cardSize = _eventCardSize();
      currentEventCard = EventCardComponent(
        event: event,
        onChoice: _handleChoice,
        position: _eventCardPosition(),
        size: cardSize,
      );
      _componentLayer.add(currentEventCard!);
      _layoutFloatingComponents();
    }
  }

  /// Handle player choice selection
  void _handleChoice(int index) {
    final choice = game.gameState.currentEvent!.choices[index];

    // Trigger particle burst for stat changes
    if (choice.statChanges.isNotEmpty) {
      final totalChange = choice.statChanges.values.fold<int>(
        0,
        (sum, val) => sum + val.abs(),
      );
      final color =
          choice.statChanges.values.first > 0 ? Colors.green : Colors.red;

      game.particleSystem.burstParticles(
        count: totalChange,
        color: color,
      );
    }

    // Apply choice to game state
    game.gameState.applyChoice(choice);

    // Load next event after a brief delay
    Future.delayed(const Duration(milliseconds: 500), () {
      _loadNextEvent();
    });
  }

  @override
  bool onKeyEvent(KeyEvent event, Set<LogicalKeyboardKey> keysPressed) {
    if (event is! KeyDownEvent) {
      return false;
    }

    final currentEvent = game.gameState.currentEvent;
    if (currentEvent == null) {
      return false;
    }

    final pressedKey = event.logicalKey;
    for (var i = 0; i < currentEvent.choices.length; i++) {
      final shortcut = currentEvent.choices[i].keyboardShortcut;
      final keySet = _shortcutKeyMap[shortcut];
      if (keySet != null && keySet.contains(pressedKey)) {
        _handleChoice(i);
        return true;
      }
    }
    return false;
  }

  void _updateFloatingLayout() {
    _layoutFloatingComponents();
  }

  @override
  void onGameResize(Vector2 newSize) {
    super.onGameResize(newSize);
    size = newSize;
    if (!isLoaded) {
      return;
    }
    _background.size = newSize;
    _componentLayer.size = newSize;
    _layoutFloatingComponents();
  }

  void _layoutFloatingComponents() {
    if (!isLoaded) {
      return;
    }

    const margin = _margin;
    final leftWidth = _leftWidth;
    final centerWidth = _centerWidth;
    final rightWidth = _rightWidth;

    topBar
      ..position = Vector2.zero()
      ..size = Vector2(size.x, _topBarHeight);

    final cardSize = _eventCardSize();
    final cardPosition = _eventCardPosition();
    currentEventCard
      ?..size = cardSize
      ..position = cardPosition;

    statPanel
      ..size = Vector2(leftWidth - margin * 1.5, _contentHeight)
      ..position = Vector2(margin, _topBarHeight + margin);

    relationshipPanel
      ..size = Vector2(rightWidth - margin * 1.5, _contentHeight)
      ..position = Vector2(
        leftWidth + centerWidth + margin * 0.5,
        _topBarHeight + margin,
      );

    quickMenuBar
      ..size = Vector2(size.x - margin * 2, _quickMenuHeight)
      ..position = Vector2(
        margin,
        size.y - _quickMenuHeight - margin,
      );
  }

  static const double _margin = 24.0;

  double get _topBarHeight => size.y * 0.12;
  double get _quickMenuHeight => size.y * 0.10;
  double get _leftWidth => size.x * 0.22;
  double get _centerWidth => size.x * 0.56;
  double get _rightWidth => size.x * 0.22;
  double get _contentHeight =>
      size.y - _topBarHeight - _quickMenuHeight - _margin * 2;

  Vector2 _eventCardSize() => Vector2(_centerWidth - _margin * 2, _contentHeight);

  Vector2 _eventCardPosition() =>
      Vector2(_leftWidth + _margin, _topBarHeight + _margin);

  @override
  void update(double dt) {
    super.update(dt);
    // Update particle system mood based on game state
    game.particleSystem.updateMood(game.gameState.mood);
  }
}
