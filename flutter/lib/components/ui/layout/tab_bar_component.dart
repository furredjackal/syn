import 'package:flame/components.dart';

import 'tab_button_component.dart';

/// Tab bar stub managing TabButtonComponents.
class TabBarComponent extends PositionComponent {
  TabBarComponent({this.tabs = const []});

  final List<TabButtonComponent> tabs;
}
