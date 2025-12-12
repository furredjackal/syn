import 'package:flame/components.dart';
import 'package:flame/game.dart';
import 'package:flutter/material.dart';

import 'inspectable_mixin.dart';

/// Runtime inspector panel for Flame components.
///
/// Provides a Unity-like inspector that displays all components using the
/// InspectableMixin and allows live editing of their properties.
///
/// Features:
/// - Automatically discovers inspectable components in the game
/// - Renders property editors based on type (switches, sliders, text)
/// - Updates component properties in real-time
/// - Cyberpunk aesthetic with cyan accents
class InspectorPanel extends StatefulWidget {
  final FlameGame game;

  const InspectorPanel({
    super.key,
    required this.game,
  });

  @override
  State<InspectorPanel> createState() => _InspectorPanelState();
}

class _InspectorPanelState extends State<InspectorPanel> {
  List<InspectableMixin> _inspectableComponents = [];
  int? _selectedIndex;

  @override
  void initState() {
    super.initState();
    _discoverInspectableComponents();
    // Periodically refresh to catch newly added components
    Future.delayed(const Duration(seconds: 1), _schedulePeriodicRefresh);
  }

  /// Schedule periodic component discovery to catch runtime additions
  void _schedulePeriodicRefresh() {
    if (mounted) {
      _discoverInspectableComponents();
      Future.delayed(const Duration(seconds: 2), _schedulePeriodicRefresh);
    }
  }

  /// Query the game world for all components that use InspectableMixin
  void _discoverInspectableComponents() {
    final components = <InspectableMixin>[];
    
    // Recursively search through the component tree
    void _searchComponents(Component component) {
      // Check if this component is inspectable
      if (component is InspectableMixin) {
        components.add(component);
      }
      
      // Recursively search children
      for (final child in component.children) {
        _searchComponents(child);
      }
    }

    // Start from game.world if it exists (HasWorldChildren games)
    try {
      final dynamic game = widget.game;
      if (game is Component) {
        // Try to access world property if it exists
        try {
          final dynamic world = (game as dynamic).world;
          if (world is Component) {
            _searchComponents(world);
          }
        } catch (_) {
          // No world property, that's okay
        }
        
        // Also search direct children of game
        for (final child in game.children) {
          _searchComponents(child);
        }
      }
    } catch (e) {
      debugPrint('[Inspector] Error discovering components: $e');
    }
    
    if (mounted) {
      setState(() {
        _inspectableComponents = components;
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 320,
      decoration: BoxDecoration(
        color: Colors.black.withValues(alpha: 0.9),
        border: Border(
          left: BorderSide(color: Colors.cyanAccent, width: 2),
        ),
        boxShadow: [
          BoxShadow(
            color: Colors.cyanAccent.withValues(alpha: 0.3),
            blurRadius: 20,
            spreadRadius: 0,
          ),
        ],
      ),
      child: Column(
        children: [
          // Header
          Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: Colors.cyanAccent.withValues(alpha: 0.2),
              border: Border(
                bottom: BorderSide(color: Colors.cyanAccent, width: 2),
              ),
            ),
            child: Row(
              children: [
                Icon(Icons.tune, color: Colors.cyanAccent, size: 24),
                const SizedBox(width: 12),
                Text(
                  'INSPECTOR',
                  style: TextStyle(
                    color: Colors.cyanAccent,
                    fontSize: 18,
                    fontWeight: FontWeight.bold,
                    letterSpacing: 2,
                  ),
                ),
                const Spacer(),
                IconButton(
                  icon: Icon(Icons.refresh, color: Colors.cyanAccent),
                  onPressed: _discoverInspectableComponents,
                  tooltip: 'Refresh Components',
                ),
              ],
            ),
          ),

          // Component List
          Expanded(
            child: _inspectableComponents.isEmpty
                ? _buildEmptyState()
                : ListView.builder(
                    itemCount: _inspectableComponents.length,
                    itemBuilder: (context, index) {
                      return _buildComponentTile(_inspectableComponents[index]);
                    },
                  ),
          ),
        ],
      ),
    );
  }

  Widget _buildEmptyState() {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(24.0),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(
              Icons.search_off,
              color: Colors.grey,
              size: 48,
            ),
            const SizedBox(height: 16),
            Text(
              'No Inspectable Components',
              style: TextStyle(
                color: Colors.grey,
                fontSize: 16,
              ),
              textAlign: TextAlign.center,
            ),
            const SizedBox(height: 8),
            Text(
              'Add InspectableMixin to your\nFlame components',
              style: TextStyle(
                color: Colors.grey.withValues(alpha: 0.7),
                fontSize: 12,
              ),
              textAlign: TextAlign.center,
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildComponentTile(InspectableMixin component) {
    final properties = component.inspectableProperties;
    final componentIndex = _inspectableComponents.indexOf(component);
    final isSelected = _selectedIndex == componentIndex;

    return Container(
      margin: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
      decoration: BoxDecoration(
        border: Border.all(
          color: isSelected 
              ? Colors.cyanAccent 
              : Colors.cyanAccent.withValues(alpha: 0.2),
          width: isSelected ? 2 : 1,
        ),
        borderRadius: BorderRadius.circular(4),
      ),
      child: ExpansionTile(
        title: Row(
          children: [
            Icon(
              Icons.widgets,
              size: 16,
              color: Colors.cyanAccent.withValues(alpha: 0.8),
            ),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                component.debugName,
                style: TextStyle(
                  color: Colors.cyanAccent,
                  fontSize: 14,
                  fontWeight: FontWeight.bold,
                ),
              ),
            ),
            Container(
              padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
              decoration: BoxDecoration(
                color: Colors.cyanAccent.withValues(alpha: 0.2),
                borderRadius: BorderRadius.circular(8),
              ),
              child: Text(
                '${properties.length}',
                style: TextStyle(
                  color: Colors.cyanAccent,
                  fontSize: 11,
                  fontWeight: FontWeight.bold,
                ),
              ),
            ),
          ],
        ),
        iconColor: Colors.cyanAccent,
        collapsedIconColor: Colors.cyanAccent.withValues(alpha: 0.6),
        tilePadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
        childrenPadding: const EdgeInsets.only(left: 16, right: 16, bottom: 12),
        onExpansionChanged: (expanded) {
          setState(() {
            _selectedIndex = expanded ? componentIndex : null;
          });
        },
        children: properties.isEmpty
            ? [
                Padding(
                  padding: const EdgeInsets.all(16),
                  child: Text(
                    'No inspectable properties',
                    style: TextStyle(
                      color: Colors.grey,
                      fontSize: 12,
                      fontStyle: FontStyle.italic,
                    ),
                  ),
                ),
              ]
            : [
                ...properties.entries.map((entry) {
                  return _buildPropertyEditor(component, entry.key, entry.value);
                }),
              ],
      ),
    );
  }

  Widget _buildPropertyEditor(
    InspectableMixin component,
    String key,
    dynamic value,
  ) {
    if (value is bool) {
      return SwitchListTile(
        title: Text(
          key,
          style: TextStyle(color: Colors.white, fontSize: 13),
        ),
        value: value,
        activeThumbColor: Colors.cyanAccent,
        onChanged: (newValue) {
          setState(() {
            component.onPropertyUpdate(key, newValue);
          });
        },
        dense: true,
      );
    } else if (value is double) {
      return Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  key,
                  style: TextStyle(color: Colors.white, fontSize: 13),
                ),
                Text(
                  value.toStringAsFixed(2),
                  style: TextStyle(
                    color: Colors.cyanAccent,
                    fontSize: 13,
                    fontFamily: 'monospace',
                  ),
                ),
              ],
            ),
          ),
          Slider(
            value: value.clamp(0.0, 100.0),
            min: 0,
            max: 100,
            divisions: 100,
            activeColor: Colors.cyanAccent,
            inactiveColor: Colors.cyanAccent.withValues(alpha: 0.3),
            onChanged: (newValue) {
              setState(() {
                component.onPropertyUpdate(key, newValue);
              });
            },
          ),
        ],
      );
    } else if (value is int) {
      return Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(
                  key,
                  style: TextStyle(color: Colors.white, fontSize: 13),
                ),
                Text(
                  value.toString(),
                  style: TextStyle(
                    color: Colors.cyanAccent,
                    fontSize: 13,
                    fontFamily: 'monospace',
                  ),
                ),
              ],
            ),
          ),
          Slider(
            value: value.toDouble().clamp(0.0, 100.0),
            min: 0,
            max: 100,
            divisions: 100,
            activeColor: Colors.cyanAccent,
            inactiveColor: Colors.cyanAccent.withValues(alpha: 0.3),
            onChanged: (newValue) {
              setState(() {
                component.onPropertyUpdate(key, newValue.round());
              });
            },
          ),
        ],
      );
    } else {
      // String or other type - display as read-only
      return ListTile(
        title: Text(
          key,
          style: TextStyle(color: Colors.white, fontSize: 13),
        ),
        subtitle: Text(
          value.toString(),
          style: TextStyle(
            color: Colors.cyanAccent.withValues(alpha: 0.7),
            fontSize: 12,
            fontFamily: 'monospace',
          ),
        ),
        dense: true,
      );
    }
  }
}
