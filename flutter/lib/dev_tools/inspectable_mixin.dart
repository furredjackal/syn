import 'package:flame/components.dart';
import 'package:flutter/material.dart';

// ============================================================================
// INSPECTOR OVERRIDES - Global runtime property override system
// ============================================================================

/// Global property override system for live widget editing.
///
/// Widgets can register themselves and their editable properties.
/// The inspector modifies values here, widgets rebuild automatically.
///
/// Usage in widgets:
/// ```dart
/// @override
/// void initState() {
///   super.initState();
///   InspectorOverrides.instance.register(
///     'DetailedStatsPanel',
///     {
///       'padding': 16.0,
///       'fontSize': 14.0,
///       'headerHeight': 60.0,
///     },
///     onUpdate: () => setState(() {}),
///   );
/// }
///
/// @override
/// Widget build(BuildContext context) {
///   final o = InspectorOverrides.instance;
///   return Container(
///     padding: EdgeInsets.all(o.get('DetailedStatsPanel.padding', 16.0)),
///     ...
///   );
/// }
/// ```
class InspectorOverrides extends ChangeNotifier {
  static final InspectorOverrides instance = InspectorOverrides._();
  InspectorOverrides._();

  // Registered widget schemas: widgetName -> {propName: defaultValue}
  final Map<String, Map<String, dynamic>> _schemas = {};
  
  // Property callbacks: widgetName -> callback
  final Map<String, VoidCallback> _callbacks = {};
  
  // Active overrides: "widgetName.propName" -> value
  final Map<String, dynamic> _overrides = {};

  /// Register a widget with its editable properties
  void register(String widgetName, Map<String, dynamic> defaults, {VoidCallback? onUpdate}) {
    _schemas[widgetName] = Map.from(defaults);
    if (onUpdate != null) {
      _callbacks[widgetName] = onUpdate;
    }
    notifyListeners();
  }

  /// Unregister a widget
  void unregister(String widgetName) {
    _schemas.remove(widgetName);
    _callbacks.remove(widgetName);
    _overrides.removeWhere((key, _) => key.startsWith('$widgetName.'));
  }

  /// Get a property value (override or default)
  T get<T>(String fullKey, T defaultValue) {
    if (_overrides.containsKey(fullKey)) {
      return _overrides[fullKey] as T;
    }
    // Try to find in schema
    final parts = fullKey.split('.');
    if (parts.length >= 2) {
      final widget = parts[0];
      final prop = parts.sublist(1).join('.');
      if (_schemas.containsKey(widget) && _schemas[widget]!.containsKey(prop)) {
        return _schemas[widget]![prop] as T;
      }
    }
    return defaultValue;
  }

  /// Set an override value
  void set(String fullKey, dynamic value) {
    _overrides[fullKey] = value;
    // Trigger widget callback
    final parts = fullKey.split('.');
    if (parts.isNotEmpty && _callbacks.containsKey(parts[0])) {
      _callbacks[parts[0]]!();
    }
    notifyListeners();
  }

  /// Reset a property to default
  void reset(String fullKey) {
    _overrides.remove(fullKey);
    final parts = fullKey.split('.');
    if (parts.isNotEmpty && _callbacks.containsKey(parts[0])) {
      _callbacks[parts[0]]!();
    }
    notifyListeners();
  }

  /// Reset all overrides for a widget
  void resetWidget(String widgetName) {
    _overrides.removeWhere((key, _) => key.startsWith('$widgetName.'));
    if (_callbacks.containsKey(widgetName)) {
      _callbacks[widgetName]!();
    }
    notifyListeners();
  }

  /// Reset all overrides
  void resetAll() {
    _overrides.clear();
    for (final callback in _callbacks.values) {
      callback();
    }
    notifyListeners();
  }

  /// Check if a property has an override
  bool hasOverride(String fullKey) => _overrides.containsKey(fullKey);

  /// Get all registered widgets
  List<String> get registeredWidgets => _schemas.keys.toList();

  /// Get schema for a widget
  Map<String, dynamic>? getSchema(String widgetName) => _schemas[widgetName];

  /// Get all properties for a widget (with current values)
  Map<String, dynamic> getWidgetProperties(String widgetName) {
    final schema = _schemas[widgetName];
    if (schema == null) return {};
    
    final result = <String, dynamic>{};
    for (final entry in schema.entries) {
      final fullKey = '$widgetName.${entry.key}';
      result[entry.key] = _overrides[fullKey] ?? entry.value;
    }
    return result;
  }

  /// Export all current overrides
  Map<String, dynamic> exportOverrides() => Map.from(_overrides);

  /// Import overrides
  void importOverrides(Map<String, dynamic> overrides) {
    _overrides.addAll(overrides);
    for (final callback in _callbacks.values) {
      callback();
    }
    notifyListeners();
  }
}

// ============================================================================
// INSPECTABLE INTERFACES - For both Flame and Flutter
// ============================================================================

/// Property metadata for the inspector
class InspectableProperty {
  final String name;
  final dynamic value;
  final Type type;
  final bool editable;
  final double? min;
  final double? max;
  final List<String>? enumValues;
  final String? category;
  final String? description;

  const InspectableProperty({
    required this.name,
    required this.value,
    required this.type,
    this.editable = true,
    this.min,
    this.max,
    this.enumValues,
    this.category,
    this.description,
  });

  /// Quick constructors for common types
  factory InspectableProperty.boolean(String name, bool value, {bool editable = true, String? category}) =>
      InspectableProperty(name: name, value: value, type: bool, editable: editable, category: category);

  factory InspectableProperty.number(String name, num value, {double min = 0, double max = 100, bool editable = true, String? category}) =>
      InspectableProperty(name: name, value: value, type: value.runtimeType, editable: editable, min: min, max: max, category: category);

  factory InspectableProperty.text(String name, String value, {bool editable = true, String? category}) =>
      InspectableProperty(name: name, value: value, type: String, editable: editable, category: category);

  factory InspectableProperty.color(String name, Color value, {bool editable = true, String? category}) =>
      InspectableProperty(name: name, value: value, type: Color, editable: editable, category: category);

  factory InspectableProperty.offset(String name, Offset value, {bool editable = true, String? category}) =>
      InspectableProperty(name: name, value: value, type: Offset, editable: editable, category: category);

  factory InspectableProperty.size(String name, Size value, {bool editable = true, String? category}) =>
      InspectableProperty(name: name, value: value, type: Size, editable: editable, category: category);

  factory InspectableProperty.enumValue(String name, dynamic value, List<dynamic> values, {bool editable = true, String? category}) =>
      InspectableProperty(
        name: name,
        value: value,
        type: value.runtimeType,
        editable: editable,
        enumValues: values.map((e) => e.toString().split('.').last).toList(),
        category: category,
      );

  factory InspectableProperty.readOnly(String name, dynamic value, {String? category}) =>
      InspectableProperty(name: name, value: value, type: value.runtimeType, editable: false, category: category);
}

/// Base interface for anything inspectable
abstract class Inspectable {
  /// The display name for this item in the inspector
  String get inspectorName;

  /// Icon to display in the inspector tree
  IconData get inspectorIcon => Icons.widgets;

  /// Category/group for organization
  String get inspectorCategory => 'General';

  /// List of inspectable properties with metadata
  List<InspectableProperty> get inspectorProperties;

  /// Called when a property is updated through the inspector
  void onInspectorPropertyUpdate(String key, dynamic value);

  /// Child inspectables (for tree view)
  List<Inspectable> get inspectorChildren => [];
}

// ============================================================================
// FLAME COMPONENT MIXIN
// ============================================================================

/// Mixin for Flame components that enables runtime inspection and property editing.
///
/// Any component using this mixin can be queried and modified through the
/// Inspector Panel, providing a Unity-like workflow for tweaking values at runtime.
///
/// Usage (Simple - override interface directly):
/// ```dart
/// class MyComponent extends PositionComponent with InspectableMixin {
///   @override
///   String get inspectorName => 'My Component';
///
///   @override
///   List<InspectableProperty> get inspectorProperties => [
///     InspectableProperty.number('Speed', speed),
///     InspectableProperty.boolean('Active', isActive),
///   ];
/// }
/// ```
///
/// Usage (Legacy - use Map-based properties):
/// ```dart
/// class MyComponent extends PositionComponent with InspectableMixin {
///   @override
///   String get debugName => 'My Custom Component';
///
///   @override
///   Map<String, dynamic> get inspectableProperties => {
///     'speed': speed,
///     'isActive': isActive,
///   };
///
///   @override
///   void onPropertyUpdate(String key, dynamic value) {
///     switch (key) {
///       case 'speed': speed = value as double; break;
///       case 'isActive': isActive = value as bool; break;
///     }
///   }
/// }
/// ```
mixin InspectableMixin on Component implements Inspectable {
  /// Legacy: The display name for this component in the inspector.
  /// Override inspectorName instead for the new API.
  String get debugName => runtimeType.toString();

  /// Legacy: A map of property names to their current values.
  /// Override inspectorProperties instead for the new API.
  Map<String, dynamic> get legacyProperties => {};

  /// Legacy: Called when a property is updated through the inspector.
  /// Override onInspectorPropertyUpdate instead for the new API.
  void onPropertyUpdate(String key, dynamic value) {}

  // Inspectable interface implementation
  @override
  String get inspectorName => debugName;

  @override
  IconData get inspectorIcon => Icons.gamepad;

  @override
  String get inspectorCategory => 'Flame Components';

  @override
  List<InspectableProperty> get inspectorProperties {
    // Convert legacy Map properties to List<InspectableProperty>
    return legacyProperties.entries.map((e) {
      final value = e.value;
      if (value is bool) {
        return InspectableProperty.boolean(e.key, value);
      } else if (value is double) {
        return InspectableProperty.number(e.key, value);
      } else if (value is int) {
        return InspectableProperty.number(e.key, value);
      } else if (value is Color) {
        return InspectableProperty.color(e.key, value);
      } else if (value is Offset) {
        return InspectableProperty.offset(e.key, value);
      } else if (value is Size) {
        return InspectableProperty.size(e.key, value);
      } else {
        return InspectableProperty.text(e.key, value.toString(), editable: false);
      }
    }).toList();
  }

  @override
  void onInspectorPropertyUpdate(String key, dynamic value) => onPropertyUpdate(key, value);

  @override
  List<Inspectable> get inspectorChildren {
    return children.whereType<InspectableMixin>().cast<Inspectable>().toList();
  }
}

// ============================================================================
// FLUTTER WIDGET MIXIN
// ============================================================================

/// Mixin for Flutter StatefulWidgets that enables runtime inspection.
///
/// Usage:
/// ```dart
/// class MyWidgetState extends State<MyWidget> with FlutterInspectableMixin {
///   double _opacity = 1.0;
///   bool _visible = true;
///
///   @override
///   String get inspectorName => 'My Widget';
///
///   @override
///   List<InspectableProperty> get inspectorProperties => [
///     InspectableProperty.number('opacity', _opacity, min: 0, max: 1),
///     InspectableProperty.boolean('visible', _visible),
///   ];
///
///   @override
///   void onInspectorPropertyUpdate(String key, dynamic value) {
///     setState(() {
///       switch (key) {
///         case 'opacity': _opacity = value as double; break;
///         case 'visible': _visible = value as bool; break;
///       }
///     });
///   }
/// }
/// ```
mixin FlutterInspectableMixin<T extends StatefulWidget> on State<T> implements Inspectable {
  @override
  IconData get inspectorIcon => Icons.widgets_outlined;

  @override
  String get inspectorCategory => 'Flutter Widgets';

  @override
  List<Inspectable> get inspectorChildren => [];

  /// Register this inspectable with the global registry
  @override
  void initState() {
    super.initState();
    InspectableRegistry.instance.register(this);
  }

  @override
  void dispose() {
    InspectableRegistry.instance.unregister(this);
    super.dispose();
  }
}

// ============================================================================
// GLOBAL REGISTRY
// ============================================================================

/// Global registry for all inspectable items (Flame and Flutter)
class InspectableRegistry extends ChangeNotifier {
  InspectableRegistry._();
  static final InspectableRegistry instance = InspectableRegistry._();

  final Set<Inspectable> _items = {};

  /// All registered inspectable items
  List<Inspectable> get items => _items.toList();

  /// Items grouped by category
  Map<String, List<Inspectable>> get itemsByCategory {
    final map = <String, List<Inspectable>>{};
    for (final item in _items) {
      map.putIfAbsent(item.inspectorCategory, () => []).add(item);
    }
    return map;
  }

  /// Register an inspectable item
  void register(Inspectable item) {
    _items.add(item);
    notifyListeners();
  }

  /// Unregister an inspectable item
  void unregister(Inspectable item) {
    _items.remove(item);
    notifyListeners();
  }

  /// Clear all items
  void clear() {
    _items.clear();
    notifyListeners();
  }
}

// ============================================================================
// WIDGET TREE INSPECTOR UTILITIES
// ============================================================================

/// Represents a node in the widget tree for inspection
class WidgetTreeNode {
  final String name;
  final String type;
  final Key? key;
  final Map<String, dynamic> properties;
  final List<WidgetTreeNode> children;
  final Rect? renderBounds;
  final Size? size;
  final BoxConstraints? constraints;

  WidgetTreeNode({
    required this.name,
    required this.type,
    this.key,
    required this.properties,
    required this.children,
    this.renderBounds,
    this.size,
    this.constraints,
  });

  /// Check if this widget is app-level (not internal Flutter framework)
  bool get isAppWidget {
    // Include all project-specific widgets (Panel, Overlay, Card, Screen, etc.)
    if (type.contains('Panel')) return true;
    if (type.contains('Overlay')) return true;
    if (type.contains('Card')) return true;
    if (type.contains('Screen')) return true;
    if (type.contains('Button')) return true;
    if (type.contains('Bar')) return true;
    if (type.contains('Menu')) return true;
    if (type.contains('Dialog')) return true;
    if (type.contains('Modal')) return true;
    if (type.contains('Drawer')) return true;
    if (type.contains('List')) return true;
    if (type.contains('Item')) return true;
    if (type.contains('Tile')) return true;
    if (type.contains('Badge')) return true;
    if (type.contains('Avatar')) return true;
    if (type.contains('Header')) return true;
    if (type.contains('Footer')) return true;
    if (type.contains('Nav')) return true;
    if (type.contains('Tab')) return true;
    if (type.contains('Chip')) return true;
    if (type.startsWith('Syn')) return true; // All Syn* widgets
    if (type.startsWith('Event')) return true;
    if (type.startsWith('Choice')) return true;
    if (type.startsWith('Stat')) return true;
    if (type.startsWith('Relationship')) return true;
    if (type.startsWith('Memory')) return true;
    if (type.startsWith('Inventory')) return true;
    if (type.startsWith('World')) return true;
    if (type.startsWith('Possession')) return true;
    if (type.startsWith('Detailed')) return true;
    return false;
  }

  /// Check if this is a "meaningful" widget worth showing in inspector
  bool get isInteresting {
    // Always include app-level widgets
    if (isAppWidget) return true;
    
    // Include common layout/visual widgets with properties
    if (type == 'Container' && properties.isNotEmpty) return true;
    if (type == 'Text') return true;
    if (type == 'RichText') return true;
    if (type == 'Icon') return true;
    if (type == 'Image') return true;
    if (type == 'SizedBox' && (properties['width'] != null || properties['height'] != null)) return true;
    if (type == 'Padding') return true;
    if (type == 'Opacity') return true;
    if (type == 'Transform') return true;
    if (type == 'DecoratedBox') return true;
    if (type == 'ClipRRect') return true;
    if (type == 'Material') return true;
    if (type == 'Scaffold') return true;
    if (type == 'AppBar') return true;
    if (type == 'Row' || type == 'Column') return true;
    if (type == 'Stack') return true;
    if (type == 'Positioned') return true;
    if (type == 'Expanded' || type == 'Flexible') return true;
    if (type == 'AnimatedContainer') return true;
    if (type == 'AnimatedOpacity') return true;
    if (type == 'AnimatedPositioned') return true;
    if (type == 'GestureDetector' && properties['hasOnTap'] == true) return true;
    if (type == 'InkWell' && properties['hasOnTap'] == true) return true;
    
    // Include any widget with size info
    if (size != null && size!.width > 0 && size!.height > 0) {
      // But skip very small internal widgets
      if (size!.width < 5 || size!.height < 5) return false;
      return true;
    }
    
    return false;
  }

  /// Get category for grouping in inspector
  String get category {
    if (isAppWidget) return 'App Widgets';
    if (type.contains('Text') || type.contains('Rich')) return 'Text';
    if (type.contains('Container') || type.contains('Box') || type.contains('Padding')) return 'Layout';
    if (type.contains('Row') || type.contains('Column') || type.contains('Stack') || type.contains('Flex')) return 'Flex';
    if (type.contains('Button') || type.contains('Gesture') || type.contains('Ink')) return 'Interactive';
    if (type.contains('Animated') || type.contains('Transition')) return 'Animation';
    if (type.contains('Scroll') || type.contains('List')) return 'Scrolling';
    if (type.contains('Image') || type.contains('Icon')) return 'Media';
    return 'Other';
  }

  /// Create from an Element with comprehensive property extraction
  factory WidgetTreeNode.fromElement(Element element, {int maxDepth = 15}) {
    if (maxDepth <= 0) {
      return WidgetTreeNode(
        name: '...',
        type: 'truncated',
        properties: {},
        children: [],
      );
    }

    final widget = element.widget;
    final properties = <String, dynamic>{};

    // Extract common properties
    if (widget.key != null) {
      properties['key'] = widget.key.toString();
    }

    // Extract widget-specific properties (comprehensive list)
    _extractWidgetProperties(widget, properties);

    // Get render info if available
    Rect? bounds;
    Size? widgetSize;
    BoxConstraints? widgetConstraints;
    
    if (element is RenderObjectElement) {
      final renderObject = element.renderObject;
      if (renderObject is RenderBox) {
        if (renderObject.hasSize) {
          widgetSize = renderObject.size;
          try {
            final topLeft = renderObject.localToGlobal(Offset.zero);
            bounds = topLeft & renderObject.size;
          } catch (_) {}
        }
        widgetConstraints = renderObject.constraints;
      }
    }

    // Collect children
    final children = <WidgetTreeNode>[];
    element.visitChildren((child) {
      children.add(WidgetTreeNode.fromElement(child, maxDepth: maxDepth - 1));
    });

    return WidgetTreeNode(
      name: widget.runtimeType.toString(),
      type: widget.runtimeType.toString(),
      key: widget.key,
      properties: properties,
      children: children,
      renderBounds: bounds,
      size: widgetSize,
      constraints: widgetConstraints,
    );
  }

  /// Recursively find all interesting widgets
  List<WidgetTreeNode> get allInterestingWidgets {
    final results = <WidgetTreeNode>[];
    _collectInteresting(this, results);
    return results;
  }

  static void _collectInteresting(WidgetTreeNode node, List<WidgetTreeNode> results) {
    if (node.isInteresting) {
      results.add(node);
    }
    for (final child in node.children) {
      _collectInteresting(child, results);
    }
  }
}

/// Extract properties from common widget types
void _extractWidgetProperties(Widget widget, Map<String, dynamic> properties) {
  // Layout widgets
  if (widget is Container) {
    if (widget.color != null) properties['color'] = _colorToHex(widget.color!);
    if (widget.padding != null) properties['padding'] = widget.padding.toString();
    if (widget.margin != null) properties['margin'] = widget.margin.toString();
    if (widget.alignment != null) properties['alignment'] = widget.alignment.toString();
    if (widget.constraints != null) properties['constraints'] = widget.constraints.toString();
  } else if (widget is Padding) {
    properties['padding'] = widget.padding.toString();
  } else if (widget is SizedBox) {
    if (widget.width != null) properties['width'] = widget.width;
    if (widget.height != null) properties['height'] = widget.height;
  } else if (widget is ConstrainedBox) {
    properties['constraints'] = widget.constraints.toString();
  } else if (widget is Align) {
    properties['alignment'] = widget.alignment.toString();
    if (widget.widthFactor != null) properties['widthFactor'] = widget.widthFactor;
    if (widget.heightFactor != null) properties['heightFactor'] = widget.heightFactor;
  } else if (widget is Center) {
    if (widget.widthFactor != null) properties['widthFactor'] = widget.widthFactor;
    if (widget.heightFactor != null) properties['heightFactor'] = widget.heightFactor;
  } else if (widget is Positioned) {
    if (widget.left != null) properties['left'] = widget.left;
    if (widget.top != null) properties['top'] = widget.top;
    if (widget.right != null) properties['right'] = widget.right;
    if (widget.bottom != null) properties['bottom'] = widget.bottom;
    if (widget.width != null) properties['width'] = widget.width;
    if (widget.height != null) properties['height'] = widget.height;
  } else if (widget is Expanded) {
    properties['flex'] = widget.flex;
  } else if (widget is Flexible) {
    properties['flex'] = widget.flex;
    properties['fit'] = widget.fit.toString();
  } else if (widget is AspectRatio) {
    properties['aspectRatio'] = widget.aspectRatio;
  } else if (widget is FractionallySizedBox) {
    if (widget.widthFactor != null) properties['widthFactor'] = widget.widthFactor;
    if (widget.heightFactor != null) properties['heightFactor'] = widget.heightFactor;
  }
  // Flex widgets
  else if (widget is Row) {
    properties['mainAxisAlignment'] = widget.mainAxisAlignment.toString().split('.').last;
    properties['crossAxisAlignment'] = widget.crossAxisAlignment.toString().split('.').last;
    properties['mainAxisSize'] = widget.mainAxisSize.toString().split('.').last;
  } else if (widget is Column) {
    properties['mainAxisAlignment'] = widget.mainAxisAlignment.toString().split('.').last;
    properties['crossAxisAlignment'] = widget.crossAxisAlignment.toString().split('.').last;
    properties['mainAxisSize'] = widget.mainAxisSize.toString().split('.').last;
  } else if (widget is Wrap) {
    properties['direction'] = widget.direction.toString().split('.').last;
    properties['alignment'] = widget.alignment.toString().split('.').last;
    properties['spacing'] = widget.spacing;
    properties['runSpacing'] = widget.runSpacing;
  } else if (widget is Stack) {
    properties['alignment'] = widget.alignment.toString();
    properties['fit'] = widget.fit.toString().split('.').last;
    properties['clipBehavior'] = widget.clipBehavior.toString().split('.').last;
  }
  // Text widgets
  else if (widget is Text) {
    properties['data'] = widget.data ?? '(TextSpan)';
    if (widget.style != null) {
      if (widget.style!.fontSize != null) properties['fontSize'] = widget.style!.fontSize;
      if (widget.style!.color != null) properties['color'] = _colorToHex(widget.style!.color!);
      if (widget.style!.fontWeight != null) properties['fontWeight'] = widget.style!.fontWeight.toString().split('.').last;
    }
    if (widget.textAlign != null) properties['textAlign'] = widget.textAlign.toString().split('.').last;
    if (widget.maxLines != null) properties['maxLines'] = widget.maxLines;
    if (widget.overflow != null) properties['overflow'] = widget.overflow.toString().split('.').last;
  } else if (widget is RichText) {
    properties['textAlign'] = widget.textAlign.toString().split('.').last;
    properties['maxLines'] = widget.maxLines;
  } else if (widget is DefaultTextStyle) {
    properties['fontSize'] = widget.style.fontSize;
    if (widget.style.color != null) properties['color'] = _colorToHex(widget.style.color!);
  }
  // Visual widgets
  else if (widget is Opacity) {
    properties['opacity'] = widget.opacity;
  } else if (widget is DecoratedBox) {
    properties['decoration'] = widget.decoration.runtimeType.toString();
  } else if (widget is ClipRRect) {
    properties['borderRadius'] = widget.borderRadius.toString();
    properties['clipBehavior'] = widget.clipBehavior.toString().split('.').last;
  } else if (widget is ClipOval) {
    properties['clipBehavior'] = widget.clipBehavior.toString().split('.').last;
  } else if (widget is Transform) {
    properties['transform'] = 'Matrix4';
    properties['alignment'] = widget.alignment?.toString();
  } else if (widget is RotatedBox) {
    properties['quarterTurns'] = widget.quarterTurns;
  }
  // Interactive widgets  
  else if (widget is GestureDetector) {
    properties['hasOnTap'] = widget.onTap != null;
    properties['hasOnDoubleTap'] = widget.onDoubleTap != null;
    properties['hasOnLongPress'] = widget.onLongPress != null;
    properties['behavior'] = widget.behavior?.toString().split('.').last ?? 'deferToChild';
  } else if (widget is InkWell) {
    properties['hasOnTap'] = widget.onTap != null;
    properties['hasOnDoubleTap'] = widget.onDoubleTap != null;
    properties['hasOnLongPress'] = widget.onLongPress != null;
  } else if (widget is ElevatedButton) {
    properties['enabled'] = widget.enabled;
  } else if (widget is TextButton) {
    properties['enabled'] = widget.enabled;
  } else if (widget is IconButton) {
    properties['iconSize'] = widget.iconSize;
    properties['isSelected'] = widget.isSelected;
  }
  // Scrolling widgets
  else if (widget is ListView) {
    properties['scrollDirection'] = widget.scrollDirection.toString().split('.').last;
    properties['shrinkWrap'] = widget.shrinkWrap;
    properties['reverse'] = widget.reverse;
  } else if (widget is SingleChildScrollView) {
    properties['scrollDirection'] = widget.scrollDirection.toString().split('.').last;
    properties['reverse'] = widget.reverse;
  } else if (widget is CustomScrollView) {
    properties['scrollDirection'] = widget.scrollDirection.toString().split('.').last;
    properties['shrinkWrap'] = widget.shrinkWrap;
  }
  // Image widgets
  else if (widget is Image) {
    properties['width'] = widget.width;
    properties['height'] = widget.height;
    properties['fit'] = widget.fit?.toString().split('.').last;
  } else if (widget is Icon) {
    properties['icon'] = widget.icon?.codePoint.toRadixString(16);
    properties['size'] = widget.size;
    if (widget.color != null) properties['color'] = _colorToHex(widget.color!);
  }
  // Material widgets
  else if (widget is Material) {
    properties['type'] = widget.type.toString().split('.').last;
    properties['elevation'] = widget.elevation;
    if (widget.color != null) properties['color'] = _colorToHex(widget.color!);
    properties['borderRadius'] = widget.borderRadius?.toString();
  } else if (widget is Card) {
    properties['elevation'] = widget.elevation;
    if (widget.color != null) properties['color'] = _colorToHex(widget.color!);
    properties['margin'] = widget.margin?.toString();
  } else if (widget is Scaffold) {
    properties['hasAppBar'] = widget.appBar != null;
    properties['hasDrawer'] = widget.drawer != null;
    properties['hasBottomNav'] = widget.bottomNavigationBar != null;
    properties['hasFAB'] = widget.floatingActionButton != null;
  } else if (widget is AppBar) {
    properties['elevation'] = widget.elevation;
    properties['centerTitle'] = widget.centerTitle;
    if (widget.backgroundColor != null) properties['backgroundColor'] = _colorToHex(widget.backgroundColor!);
  }
  // Animation widgets
  else if (widget is AnimatedContainer) {
    properties['duration'] = widget.duration.inMilliseconds;
    properties['curve'] = widget.curve.toString();
  } else if (widget is AnimatedOpacity) {
    properties['opacity'] = widget.opacity;
    properties['duration'] = widget.duration.inMilliseconds;
  } else if (widget is AnimatedPositioned) {
    properties['duration'] = widget.duration.inMilliseconds;
    if (widget.left != null) properties['left'] = widget.left;
    if (widget.top != null) properties['top'] = widget.top;
  } else if (widget is FadeTransition) {
    properties['opacity'] = 'Animation<double>';
  } else if (widget is SlideTransition) {
    properties['position'] = 'Animation<Offset>';
  }
}

String _colorToHex(Color color) {
  return '#${color.value.toRadixString(16).padLeft(8, '0').toUpperCase()}';
}

/// Utility to capture the widget tree from a BuildContext
class WidgetTreeCapture {
  /// Capture the widget tree starting from the given context
  static WidgetTreeNode? capture(BuildContext context, {int maxDepth = 20}) {
    if (context is Element) {
      return WidgetTreeNode.fromElement(context, maxDepth: maxDepth);
    }
    return null;
  }

  /// Find all widgets of a specific type in the tree
  static List<WidgetTreeNode> findByType(WidgetTreeNode root, String typeName) {
    final results = <WidgetTreeNode>[];
    
    void search(WidgetTreeNode node) {
      if (node.type == typeName) {
        results.add(node);
      }
      for (final child in node.children) {
        search(child);
      }
    }
    
    search(root);
    return results;
  }

  /// Get a flat list of all nodes in the tree
  static List<WidgetTreeNode> flatten(WidgetTreeNode root) {
    final results = <WidgetTreeNode>[];
    
    void collect(WidgetTreeNode node) {
      results.add(node);
      for (final child in node.children) {
        collect(child);
      }
    }
    
    collect(root);
    return results;
  }
}

// ============================================================================
// RENDER OBJECT INSPECTOR
// ============================================================================

/// Information about a RenderObject for inspection
class RenderObjectInfo {
  final String type;
  final Rect? paintBounds;
  final Rect? semanticBounds;
  final bool needsPaint;
  final bool needsLayout;
  final bool needsCompositing;
  final Map<String, dynamic> properties;

  RenderObjectInfo({
    required this.type,
    this.paintBounds,
    this.semanticBounds,
    required this.needsPaint,
    required this.needsLayout,
    required this.needsCompositing,
    required this.properties,
  });

  factory RenderObjectInfo.from(RenderObject renderObject) {
    final properties = <String, dynamic>{};

    if (renderObject is RenderBox) {
      if (renderObject.hasSize) {
        properties['size'] = renderObject.size.toString();
      }
      properties['constraints'] = renderObject.constraints.toString();
    }

    return RenderObjectInfo(
      type: renderObject.runtimeType.toString(),
      paintBounds: renderObject.paintBounds,
      semanticBounds: renderObject.semanticBounds,
      needsPaint: renderObject.debugNeedsPaint,
      needsLayout: renderObject.debugNeedsLayout,
      needsCompositing: false,
      properties: properties,
    );
  }
}

// ============================================================================
// PERFORMANCE METRICS
// ============================================================================

/// Performance metrics for inspection
class InspectorMetrics {
  final int widgetCount;
  final int elementCount;
  final int renderObjectCount;
  final int dirtyElements;
  final Duration? lastBuildTime;
  final Duration? lastPaintTime;

  InspectorMetrics({
    required this.widgetCount,
    required this.elementCount,
    required this.renderObjectCount,
    required this.dirtyElements,
    this.lastBuildTime,
    this.lastPaintTime,
  });

  /// Capture metrics from a context
  factory InspectorMetrics.capture(BuildContext context) {
    int widgetCount = 0;
    int elementCount = 0;
    int renderObjectCount = 0;
    int dirtyElements = 0;

    void visit(Element element) {
      elementCount++;
      widgetCount++;
      if (element is RenderObjectElement) {
        renderObjectCount++;
      }
      if (element.dirty) {
        dirtyElements++;
      }
      element.visitChildren(visit);
    }

    if (context is Element) {
      visit(context);
    }

    return InspectorMetrics(
      widgetCount: widgetCount,
      elementCount: elementCount,
      renderObjectCount: renderObjectCount,
      dirtyElements: dirtyElements,
    );
  }

  @override
  String toString() => 'Widgets: $widgetCount, Elements: $elementCount, RenderObjects: $renderObjectCount, Dirty: $dirtyElements';
}

