import 'package:flame/game.dart';
import 'package:flame/route.dart';
import 'models/game_state.dart';
import 'ui_effect_layer.dart';
import 'widgets/particle_system_component.dart';
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

class SynGame extends FlameGame {
  late final GameState gameState;
  late UIEffectLayer uiEffectLayer;
  late ParticleSystemComponent particleSystem;

  SynGame() {
    gameState = GameState();
  }

  @override
  void onMount() {
    // Add global UI effect layer (renders on top of all game components)
    uiEffectLayer = UIEffectLayer();
    add(uiEffectLayer);

    // Add particle system for mood-driven environmental effects
    particleSystem = ParticleSystemComponent();
    add(particleSystem);

    // Add router for screen management
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
