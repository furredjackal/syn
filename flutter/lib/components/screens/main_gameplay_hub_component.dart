import 'package:flame/components.dart';

import '../../models/game_state.dart';
import '../../syn_game.dart';
import '../ui/cards/event_card_component.dart';
import '../ui/panels/quick_panel_component.dart';
import '../ui/panels/relationship_panel_component.dart';
import '../ui/panels/stat_panel_component.dart';
import '../ui/panels/top_bar_component.dart';
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
        .clamp(SynTopBar.height * 0.85, SynTopBar.height * 1.15) as double;
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
