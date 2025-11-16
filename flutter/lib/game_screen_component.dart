import 'package:flame/components.dart';
import 'package:syn/flutter/lib/models/game_state.dart';
import 'package:syn/flutter/lib/syn_game.dart';
import 'package:syn/flutter/lib/widgets/event_card_component.dart';
import 'package:syn/flutter/lib/widgets/quick_menu_bar_component.dart';
import 'package:syn/flutter/lib/widgets/relationship_panel_component.dart';
import 'package:syn/flutter/lib/widgets/stat_panel_component.dart';
import 'package:syn/flutter/lib/widgets/top_bar_component.dart';

class GameScreenComponent extends Component with HasGameRef<SynGame> {
  @override
  Future<void> onLoad() async {
    final screenSize = game.size;

    add(TopBarComponent()
      ..position = Vector2(0, 0)
      ..size = Vector2(screenSize.x, 80));

    add(StatPanelComponent()
      ..position = Vector2(0, 96)
      ..size = Vector2(screenSize.x * 0.2, screenSize.y - 176));

    add(RelationshipPanelComponent()
      ..position = Vector2(screenSize.x * 0.8, 96)
      ..size = Vector2(screenSize.x * 0.2, screenSize.y - 176));

    add(QuickMenuBarComponent()
      ..position = Vector2(0, screenSize.y - 80)
      ..size = Vector2(screenSize.x, 80));

    _loadNextEvent();
  }

  void _loadNextEvent() {
    game.gameState.setCurrentEvent(
      GameEvent(
        id: 'demo_001',
        title: 'A New Beginning',
        description:
            'You wake up on your first day of school. Your parents have prepared your lunch.',
        choices: [
          GameChoice(
              text: 'Eat breakfast',
              statChanges: {'health': 10},
              keyboardShortcut: 1),
          GameChoice(
              text: 'Skip breakfast',
              statChanges: {'health': -5},
              keyboardShortcut: 2),
        ],
        lifeStage: 'Child',
        age: 6,
      ),
    );
    _showEvent();
  }

  void _showEvent() {
    final event = game.gameState.currentEvent;
    if (event != null) {
      add(EventCardComponent(
        event: event,
        onChoice: _handleChoice,
        position: Vector2(game.size.x * 0.25, 96),
        size: Vector2(game.size.x * 0.5, game.size.y - 176),
      ));
    }
  }

  void _handleChoice(int index) {
    final choice = game.gameState.currentEvent!.choices[index];
    game.gameState.applyChoice(choice);
    // TODO: Show stat changes
    _loadNextEvent();
  }
}
