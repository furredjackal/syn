import 'package:flame/camera.dart';
import 'package:flame/components.dart';

import '../../syn_game.dart';

/// Main Gameplay Hub stub with world + camera setup.
class MainGameplayHubComponent extends Component with HasGameReference<SynGame> {
  late final World _world;
  late final CameraComponent _camera;

  @override
  Future<void> onLoad() async {
    _world = World();
    _camera = CameraComponent(world: _world);
    add(_world);
    add(_camera);
  }
}
