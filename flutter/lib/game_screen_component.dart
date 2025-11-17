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

    // Position components according to floating canvas model
    // Top bar: thin strip at top
    topBar = TopBarComponent()
      ..position = Vector2(40, 25)
      ..size = Vector2(size.x - 80, 60);
    _componentLayer.add(topBar);

    // Event card: centered focal point (max 60% of screen width)
    final eventWidth = (size.x * 0.6).clamp(400.0, size.x - 200);
    final eventHeight = (size.y * 0.65).clamp(200.0, size.y - 200);

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
      position: Vector2(
        (size.x - eventWidth) / 2,
        (size.y - eventHeight) / 2 - 40,
      ),
      size: Vector2(eventWidth, eventHeight),
    );
    _componentLayer.add(eventCard);
    currentEventCard = eventCard;

    // Stat panel: floats left (compact ~250px width, 250-300px height)
    const statPanelWidth = 280.0;
    const statPanelHeight = 280.0;
    statPanel = StatPanelComponent()
      ..position = Vector2(55, size.y * 0.30)
      ..size = Vector2(statPanelWidth, statPanelHeight);
    _componentLayer.add(statPanel);

    // Relationship panel: floats right (compact ~250px width, 250-300px height)
    const relPanelWidth = 280.0;
    const relPanelHeight = 280.0;
    relationshipPanel = RelationshipPanelComponent()
      ..position = Vector2(size.x - relPanelWidth - 55, size.y * 0.30)
      ..size = Vector2(relPanelWidth, relPanelHeight);
    _componentLayer.add(relationshipPanel);

    // Quick menu bar: thin strip at bottom
    quickMenuBar = QuickMenuBarComponent()
      ..position = Vector2(40, size.y - 120)
      ..size = Vector2(size.x - 80, 100);
    _componentLayer.add(quickMenuBar);

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
      final eventWidth = (size.x * 0.6).clamp(400.0, size.x - 200);
      final eventHeight = (size.y * 0.65).clamp(200.0, size.y - 200);
      currentEventCard = EventCardComponent(
        event: event,
        onChoice: _handleChoice,
        position: Vector2(
          (size.x - eventWidth) / 2,
          (size.y - eventHeight) / 2 - 40,
        ),
        size: Vector2(eventWidth, eventHeight),
      );
      _componentLayer.add(currentEventCard!);
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
    // Recalculate positions for floating components (called on resize)
    const statPanelWidth = 280.0;
    const statPanelHeight = 280.0;
    const relPanelWidth = 280.0;
    const relPanelHeight = 280.0;

    topBar
      ..position = Vector2(40, 25)
      ..size = Vector2(size.x - 80, 60);

    final eventWidth = (size.x * 0.6).clamp(400.0, size.x - 200);
    final eventHeight = (size.y * 0.65).clamp(200.0, size.y - 200);
    currentEventCard
      ?..position = Vector2(
        (size.x - eventWidth) / 2,
        (size.y - eventHeight) / 2 - 40,
      )
      ..size = Vector2(eventWidth, eventHeight);

    statPanel
      ..position = Vector2(55, size.y * 0.30)
      ..size = Vector2(statPanelWidth, statPanelHeight);

    relationshipPanel
      ..position = Vector2(size.x - relPanelWidth - 55, size.y * 0.30)
      ..size = Vector2(relPanelWidth, relPanelHeight);

    quickMenuBar
      ..position = Vector2(40, size.y - 120)
      ..size = Vector2(size.x - 80, 100);
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

    _updateFloatingLayout();
    if (game.gameState.currentEvent != null) {
      _showEvent();
    }
  }

  @override
  void update(double dt) {
    super.update(dt);
    // Update particle system mood based on game state
    game.particleSystem.updateMood(game.gameState.mood);
  }
}
