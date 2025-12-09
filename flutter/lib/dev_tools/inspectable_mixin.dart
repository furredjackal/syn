import 'package:flame/components.dart';

/// Mixin for Flame components that enables runtime inspection and property editing.
///
/// Any component using this mixin can be queried and modified through the
/// Inspector Panel, providing a Unity-like workflow for tweaking values at runtime.
///
/// Usage:
/// ```dart
/// class MyComponent extends PositionComponent with InspectableMixin {
///   @override
///   String get debugName => 'My Custom Component';
///
///   @override
///   Map<String, dynamic> get inspectableProperties => {
///     'speed': speed,
///     'isActive': isActive,
///     'opacity': opacity,
///   };
///
///   @override
///   void onPropertyUpdate(String key, dynamic value) {
///     switch (key) {
///       case 'speed':
///         speed = value as double;
///         break;
///       case 'isActive':
///         isActive = value as bool;
///         break;
///       case 'opacity':
///         opacity = value as double;
///         break;
///     }
///   }
/// }
/// ```
mixin InspectableMixin on Component {
  /// The display name for this component in the inspector.
  /// Should be human-readable and descriptive.
  String get debugName;

  /// A map of property names to their current values.
  ///
  /// Supported types:
  /// - bool: Will be rendered as a SwitchListTile
  /// - double: Will be rendered as a Slider (0-100 range)
  /// - int: Will be rendered as a Slider (converted to/from double)
  /// - String: Will be rendered as read-only text
  Map<String, dynamic> get inspectableProperties;

  /// Called when a property is updated through the inspector.
  ///
  /// Implement this to apply the new value to your component's state.
  /// Make sure to handle type casting appropriately.
  void onPropertyUpdate(String key, dynamic value);
}
