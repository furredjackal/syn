import 'package:flame/components.dart';
import 'package:flame/events.dart';

import '../syn_game.dart';

/// Stub for save/load screen with tappable slots.
class SaveLoadComponent extends Component
    with HasGameReference<SynGame>, HasTappables {}
