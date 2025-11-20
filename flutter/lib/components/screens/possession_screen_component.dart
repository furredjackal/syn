import 'package:flame/components.dart';

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

    const columns = 3;
    const rows = 2;
    const spacing = 20.0;
    final itemSize = Vector2(150, 150);

    final gridWidth = columns * itemSize.x + (columns - 1) * spacing;
    final gridHeight = rows * itemSize.y + (rows - 1) * spacing;

    final gridContainer = PositionComponent(
      size: Vector2(gridWidth, gridHeight),
      anchor: Anchor.center,
      position: Vector2(game.size.x / 2, game.size.y / 2),
    );
    add(gridContainer);

    for (var i = 0; i < columns * rows; i++) {
      final col = i % columns;
      final row = i ~/ columns;
      final button = BaseButtonComponent(
        onTap: () {
          // TODO: Implement possession logic
          print('Possessing item ${i + 1}');
        },
        size: itemSize,
        position: Vector2(
          col * (itemSize.x + spacing),
          row * (itemSize.y + spacing),
        ),
      );
      gridContainer.add(button);
    }
  }
}
