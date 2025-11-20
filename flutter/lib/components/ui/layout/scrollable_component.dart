import 'package:flame/components.dart';
import 'package:flame/events.dart';

/// Scrollable container stub with drag-based offset.
class ScrollableComponent extends PositionComponent with DragCallbacks {
  Vector2 scrollOffset = Vector2.zero();

  @override
  void onDragUpdate(DragUpdateEvent event) {
    scrollOffset += event.localDelta;
    super.onDragUpdate(event);
  }
}
