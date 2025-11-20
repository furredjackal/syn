import 'package:flame/components.dart';
import 'package:flame/events.dart';

/// Tab button stub with TapCallbacks.
class TabButtonComponent extends PositionComponent with TapCallbacks {
  TabButtonComponent({this.label = ''});

  final String label;
}
