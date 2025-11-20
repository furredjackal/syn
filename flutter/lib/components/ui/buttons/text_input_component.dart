import 'package:flame/components.dart';
import 'package:flame/events.dart';

import '../../../syn_game.dart';

/// Text input stub that triggers the Flutter text_input overlay.
class TextInputComponent extends PositionComponent
    with HasGameReference<SynGame>, TapCallbacks {
  TextInputComponent({this.placeholder = 'Enter text'});

  final String placeholder;
  String value = '';

  @override
  void onTapUp(TapUpEvent event) {
    game.overlays.add('text_input');
    super.onTapUp(event);
  }
}
