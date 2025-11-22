import 'package:flame/components.dart';

import '../../../models/game_state.dart';
import '../../../syn_game.dart';
import 'event_card_component.dart';

class EventCardStackComponent extends PositionComponent
    with HasGameReference<SynGame> {
  late final EventCardComponent _card;
  late GameEvent _event;

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    _event = game.gameState.currentEvent ?? _placeholderEvent();
    _layout();

    _card = EventCardComponent(
      event: _event,
      onChoice: (index) {},
      position: Vector2.zero(),
      size: size,
    )..anchor = Anchor.topLeft;

    add(_card);
  }

  @override
  void onGameResize(Vector2 newSize) {
    super.onGameResize(newSize);
    _layout();
    _card
      ..size = size
      ..onGameResize(size);
  }

  void _layout() {
    final viewport = game.size;
    final cardWidth = viewport.x * 0.48;
    final cardHeight = viewport.y * 0.7;

    size = Vector2(cardWidth, cardHeight);
    position = Vector2(
      viewport.x / 2 - cardWidth / 2,
      viewport.y / 2 - cardHeight / 2,
    );
  }

  GameEvent _placeholderEvent() => GameEvent(
        id: 'placeholder',
        title: 'LOADING...',
        description: '...',
        choices: const [],
        lifeStage: 'Child',
        age: 6,
      );
}
