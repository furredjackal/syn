import 'package:flame/components.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'models/game_state.dart';
import 'syn_game.dart';
import 'widgets/event_card_component.dart';
import 'widgets/quick_menu_bar_component.dart';
import 'widgets/relationship_panel_component.dart';
import 'widgets/stat_panel_component.dart';
import 'widgets/top_bar_component.dart';

/// Main game screen component using Flame components instead of Flutter widgets.
///
/// Follows the Flame Engine UI architecture with a layout of:
/// - TopBar (age, mood, life stage display)
/// - StatPanel (left side: health, wealth, charisma, etc.)
/// - EventCard (center: current event with choices)
/// - RelationshipPanel (right side: active relationships)
/// - QuickMenuBar (bottom: memory, save, settings, menu buttons)
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
  late TopBarComponent topBar;
  late StatPanelComponent statPanel;
  late RelationshipPanelComponent relationshipPanel;
  late QuickMenuBarComponent quickMenuBar;

  EventCardComponent? currentEventCard;

  @override
  Future<void> onLoad() async {
    // Set component to fill entire screen
    size = game.size;

    // Background gradient (dark theme)
    add(RectangleComponent(
      paint: Paint()..color = const Color(0xFF0A0E27),
      size: size,
    ));

    // Calculate layout dimensions
    const topBarHeight = 80.0;
    const bottomBarHeight = 80.0;
    final panelWidth = size.x * 0.2;
    final contentHeight = size.y - topBarHeight - bottomBarHeight;

    // Add top bar
    topBar = TopBarComponent()
      ..position = Vector2(0, 0)
      ..size = Vector2(size.x, topBarHeight);
    add(topBar);

    // Add left stat panel
    statPanel = StatPanelComponent()
      ..position = Vector2(0, topBarHeight)
      ..size = Vector2(panelWidth, contentHeight);
    add(statPanel);

    // Add right relationship panel
    relationshipPanel = RelationshipPanelComponent()
      ..position = Vector2(size.x - panelWidth, topBarHeight)
      ..size = Vector2(panelWidth, contentHeight);
    add(relationshipPanel);

    // Add bottom quick menu bar
    quickMenuBar = QuickMenuBarComponent()
      ..position = Vector2(0, size.y - bottomBarHeight)
      ..size = Vector2(size.x, bottomBarHeight);
    add(quickMenuBar);

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
    // Remove previous event card if it exists
    currentEventCard?.removeFromParent();

    final event = game.gameState.currentEvent;
    if (event != null) {
      const topBarHeight = 80.0;
      const bottomBarHeight = 80.0;
      final panelWidth = size.x * 0.2;
      final contentHeight = size.y - topBarHeight - bottomBarHeight;
      final centerWidth = size.x - (panelWidth * 2);

      // Create and position event card in center panel
      currentEventCard = EventCardComponent(
        event: event,
        onChoice: _handleChoice,
        position: Vector2(panelWidth, topBarHeight),
        size: Vector2(centerWidth, contentHeight),
      );
      add(currentEventCard!);
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

  @override
  void update(double dt) {
    super.update(dt);
    // Update particle system mood based on game state
    game.particleSystem.updateMood(game.gameState.mood);
  }
}
