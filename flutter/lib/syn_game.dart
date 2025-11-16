import 'package:flame/game.dart';
import 'package:flame/route.dart';
import 'models/game_state.dart';
import 'character_creation_component.dart';
import 'debug_console_component.dart';
import 'detailed_stat_component.dart';
import 'end_of_life_component.dart';
import 'game_screen_component.dart';
import 'inventory_screen_component.dart';
import 'main_menu_component.dart';
import 'memory_journal_component.dart';
import 'relationship_network_component.dart';
import 'save_load_component.dart';
import 'settings_screen_component.dart';
import 'splash_screen_component.dart';
import 'world_map_component.dart';

class SynGame extends FlameGame with HasRouter {
  late final GameState gameState;

  SynGame() {
    gameState = GameState();
  }

  @override
  void onMount() {
    add(
      RouterComponent(
        initialRoute: 'splash',
        routes: {
          'splash': Route(SplashScreenComponent.new),
          'menu': Route(MainMenuComponent.new),
          'character_creation': Route(CharacterCreationComponent.new),
          'game': Route(GameScreenComponent.new),
          'journal': Route(MemoryJournalComponent.new),
          'settings': Route(SettingsScreenComponent.new),
          'detailed_stats': Route(DetailedStatComponent.new),
          'relationships': Route(RelationshipNetworkComponent.new),
          'inventory': Route(InventoryScreenComponent.new),
          'map': Route(WorldMapComponent.new),
          'save_load': Route(SaveLoadComponent.new),
          'end_of_life': Route(EndOfLifeComponent.new),
          'debug': Route(DebugConsoleComponent.new),
        },
      ),
    );
    super.onMount();
  }
}
