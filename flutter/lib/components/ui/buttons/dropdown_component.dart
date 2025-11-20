import 'package:flame/components.dart';
import 'package:flame/events.dart';

/// Dropdown stub that can expand/collapse on tap.
class DropdownComponent extends PositionComponent with TapCallbacks {
  DropdownComponent({this.items = const [], this.onSelected});

  final List<String> items;
  final void Function(String)? onSelected;
  bool isOpen = false;

  @override
  void onTapUp(TapUpEvent event) {
    isOpen = !isOpen;
    if (isOpen && items.isNotEmpty) {
      onSelected?.call(items.first);
    }
    super.onTapUp(event);
  }
}
