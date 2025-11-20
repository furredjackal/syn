import 'package:flame/components.dart';
import 'package:flame/layout.dart';

import '../../syn_game.dart';
import '../ui/buttons/base_button_component.dart';

/// The possession screen, where the player chooses a new host to possess.
class PossessionScreenComponent extends Component
    with HasGameReference<SynGame> {
  @override
  Future<void> onLoad() async {
    await super.onLoad();

    // Add a title
    final title = TextComponent(
      text: 'Choose Your Host',
      position: Vector2(game.size.x / 2, 50),
      anchor: Anchor.topCenter,
    );
    add(title);

    // Add a grid for the possessable items
    final grid = GridLayout(
      columns: 3,
      padding: const EdgeInsets.all(20),
    );

    // Add some placeholder items to the grid
    for (var i = 0; i < 6; i++) {
      grid.add(
        BaseButtonComponent(
          onTap: () {
            // TODO: Implement possession logic
            print('Possessing item ${i + 1}');
          },
          size: Vector2(150, 150),
        ),
      );
    }

    // Center the grid on the screen
    final gridContainer = AlignComponent(
      child: grid,
      alignment: Anchor.center,
    );
    add(gridContainer);
  }
}
