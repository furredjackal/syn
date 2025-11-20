import 'package:flame/components.dart';
import 'package:flame/events.dart';

/// Simple toggle switch stub with TapCallbacks.
class ToggleSwitchComponent extends PositionComponent with TapCallbacks {
  ToggleSwitchComponent({this.value = false, this.onChanged});

  bool value;
  final void Function(bool)? onChanged;

  @override
  void onTapUp(TapUpEvent event) {
    value = !value;
    onChanged?.call(value);
    super.onTapUp(event);
  }
}
