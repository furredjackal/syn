import 'package:flame/game.dart';
import 'models/game_state.dart';
import 'ui_effect_layer.dart';
import 'widgets/particle_system_component.dart' as custom;
import 'game_screen_component.dart';

class SynGame extends FlameGame {
  late final GameState gameState;
  late UIEffectLayer uiEffectLayer;
  late custom.ParticleSystemComponent particleSystem;

  SynGame() {
    gameState = GameState();
  }

  @override
  void onMount() {
    // Add global UI effect layer (renders on top of all game components)
    uiEffectLayer = UIEffectLayer();
    add(uiEffectLayer);

    // Add particle system for mood-driven environmental effects
    particleSystem = custom.ParticleSystemComponent();
    add(particleSystem);

    // Note: RouterComponent routing is handled via Flutter Material navigation
    // Game components are added directly in onMount
    add(GameScreenComponent());
    
    super.onMount();
  }
}
