import 'package:flame/components.dart';

import '../../syn_game.dart';

/// The main hub for gameplay, containing the world and camera.
class MainGameplayHubComponent extends World
    with HasGameReference<SynGame> {
  MainGameplayHubComponent();

  late final CameraComponent cameraComponent;

  @override
  Future<void> onLoad() async {
    await super.onLoad();

    cameraComponent = CameraComponent(world: this)
      ..viewfinder.anchor = Anchor.center
      ..viewfinder.zoom = 1.0;

    game.add(cameraComponent);
    game.camera = cameraComponent;
  }
}
