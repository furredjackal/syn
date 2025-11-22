import 'package:flame/components.dart';

import '../../syn_game.dart';
import '../ui/cards/event_card_stack_component.dart';
import '../ui/panels/quick_actions_dock_component.dart';
import '../ui/panels/relationships_dock_component.dart';
import '../ui/panels/stats_dock_component.dart';
import '../ui/panels/top_bar_component.dart';
import '../ui/system/background_layer_component.dart';

/// Magnetic Dock System: Edge panels that expand on hover/click
/// Event card dominates center, stats/relationships magnetically dock to edges
class MagneticDockHubComponent extends PositionComponent
    with HasGameReference<SynGame> {
  // -- Child Components --
  late final BackgroundLayerComponent _background;
  late final TopBarComponent _topBar;
  late final StatsDockComponent _leftDock; // Stats
  late final RelationshipsDockComponent _rightDock; // Relationships
  late final QuickActionsDockComponent _bottomDock; // Quick menu dock
  late final EventCardStackComponent _eventCardStack;

  // -- Layout Constants --
  static const double _topBarHeight = 92.0;

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    size = game.size.clone();
    position = Vector2.zero();

    // 1. Background
    _background = BackgroundLayerComponent()..priority = 0;

    // 2. Top Bar
    _topBar = TopBarComponent(
      gameState: game.gameState,
      onStats: () => _leftDock.toggle(),
      onSettings: game.showSettings,
      onSave: game.showSaveLoad,
      onPause: game.togglePauseOverlay,
      onNotifications: () {},
    )..priority = 30;

    // 3. Center Card Stack
    _eventCardStack = EventCardStackComponent()..priority = 10;

    // 4. Left Dock (Stats)
    _leftDock = StatsDockComponent(priority: 20);

    // 5. Right Dock (Relationships)
    _rightDock = RelationshipsDockComponent(priority: 20);

    // 6. Bottom Dock (Quick menu)
    _bottomDock = QuickActionsDockComponent(priority: 18);

    addAll([
      _background,
      _topBar,
      _eventCardStack,
      _leftDock,
      _rightDock,
      _bottomDock,
    ]);
  }

  @override
  void onGameResize(Vector2 newSize) {
    super.onGameResize(newSize);
    size = newSize;
    _background.size = newSize;
    _topBar.size = Vector2(newSize.x, _topBarHeight);
    _topBar.position = Vector2.zero();
  }
}
