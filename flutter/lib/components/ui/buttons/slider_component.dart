import 'package:flame/components.dart';
import 'package:flame/events.dart';

/// Slider stub using DragCallbacks.
class SliderComponent extends PositionComponent with DragCallbacks {
  SliderComponent({this.value = 0.0, this.onChanged});

  double value; // 0..1
  final void Function(double)? onChanged;

  @override
  void onDragUpdate(DragUpdateEvent event) {
    final local = event.localEndPosition;
    if (size.x <= 0) return;
    value = (local.x / size.x).clamp(0.0, 1.0);
    onChanged?.call(value);
    super.onDragUpdate(event);
  }
}
