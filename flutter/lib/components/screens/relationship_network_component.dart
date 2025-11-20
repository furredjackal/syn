import 'dart:ui';

import 'package:flame/components.dart';
import 'package:flame/palette.dart';

import '../../syn_game.dart';
import '../ui/charts/network_node_component.dart';

/// A draggable graph that displays the relationships between characters.
class RelationshipNetworkComponent extends Component
    with HasGameReference<SynGame> {
  late final List<NetworkNodeComponent> _nodes;
  late final Path _connections;

  @override
  Future<void> onLoad() async {
    await super.onLoad();

    // Create some placeholder nodes
    _nodes = [
      NetworkNodeComponent(
        position: Vector2(100, 100),
        label: 'Player',
      ),
      NetworkNodeComponent(
        position: Vector2(300, 150),
        label: 'Friend',
      ),
      NetworkNodeComponent(
        position: Vector2(200, 300),
        label: 'Enemy',
      ),
    ];

    // Add the nodes to the component
    for (final node in _nodes) {
      add(node);
    }

    // Create the connections between the nodes
    _connections = Path()
      ..moveTo(_nodes[0].position.x, _nodes[0].position.y)
      ..lineTo(_nodes[1].position.x, _nodes[1].position.y)
      ..moveTo(_nodes[0].position.x, _nodes[0].position.y)
      ..lineTo(_nodes[2].position.x, _nodes[2].position.y);
  }

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    final paint = BasicPalette.white.withAlpha(100).paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2.0;
    canvas.drawPath(_connections, paint);
  }
}
