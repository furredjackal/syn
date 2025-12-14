import 'package:flame/components.dart';
import 'package:flame/game.dart';
import 'package:flutter/material.dart';

import 'inspectable_mixin.dart';

/// Comprehensive runtime inspector panel for both Flame and Flutter.
///
/// Provides a Unity-like inspector that displays:
/// - Flame components using InspectableMixin
/// - Flutter widgets using FlutterInspectableMixin
/// - Widget tree visualization
/// - Render object inspection
/// - Performance metrics
///
/// Features:
/// - Tab-based organization (Components, Widgets, Tree, Metrics)
/// - Live property editing with type-specific editors
/// - Hierarchical tree view
/// - Search/filter functionality
/// - Cyberpunk aesthetic with cyan accents
class InspectorPanel extends StatefulWidget {
  final FlameGame? game;
  final BuildContext? appContext;

  const InspectorPanel({
    super.key,
    this.game,
    this.appContext,
  });

  @override
  State<InspectorPanel> createState() => _InspectorPanelState();
}

class _InspectorPanelState extends State<InspectorPanel> with SingleTickerProviderStateMixin {
  late TabController _tabController;
  List<InspectableMixin> _flameComponents = [];
  String _searchQuery = '';
  int? _selectedIndex;
  WidgetTreeNode? _widgetTree;
  InspectorMetrics? _metrics;

  final List<String> _tabs = ['Overrides', 'Components', 'Widgets', 'Tree'];

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: _tabs.length, vsync: this);
    _discoverFlameComponents();
    _listenToRegistry();
    _listenToOverrides();
    Future.delayed(const Duration(seconds: 1), _schedulePeriodicRefresh);
  }

  @override
  void dispose() {
    _tabController.dispose();
    InspectableRegistry.instance.removeListener(_onRegistryChanged);
    InspectorOverrides.instance.removeListener(_onOverridesChanged);
    super.dispose();
  }

  void _listenToRegistry() {
    InspectableRegistry.instance.addListener(_onRegistryChanged);
  }

  void _listenToOverrides() {
    InspectorOverrides.instance.addListener(_onOverridesChanged);
  }

  void _onRegistryChanged() {
    if (mounted) setState(() {});
  }

  void _onOverridesChanged() {
    if (mounted) setState(() {});
  }

  void _schedulePeriodicRefresh() {
    if (mounted) {
      _discoverFlameComponents();
      _captureWidgetTree();
      _captureMetrics();
      Future.delayed(const Duration(seconds: 2), _schedulePeriodicRefresh);
    }
  }

  void _discoverFlameComponents() {
    if (widget.game == null) {
      debugPrint('[Inspector] No game provided');
      return;
    }
    
    final components = <InspectableMixin>[];

    void searchComponents(Component component) {
      if (component is InspectableMixin) {
        components.add(component);
      }
      for (final child in component.children) {
        searchComponents(child);
      }
    }

    try {
      final game = widget.game!;
      debugPrint('[Inspector] Searching game with ${game.children.length} children');
      
      // FlameGame extends Component, so we can safely iterate
      try {
        final dynamic world = (game as dynamic).world;
        if (world is Component) {
          debugPrint('[Inspector] Found world with ${world.children.length} children');
          searchComponents(world);
        }
      } catch (_) {}

      for (final child in game.children) {
        debugPrint('[Inspector] Checking child: ${child.runtimeType}');
        searchComponents(child);
      }
      
      debugPrint('[Inspector] Found ${components.length} inspectable components');
    } catch (e) {
      debugPrint('[Inspector] Error discovering components: $e');
    }

    if (mounted) {
      setState(() {
        _flameComponents = components;
      });
    }
  }

  void _captureWidgetTree() {
    if (widget.appContext != null) {
      _widgetTree = WidgetTreeCapture.capture(widget.appContext!, maxDepth: 15);
    }
  }

  void _captureMetrics() {
    if (widget.appContext != null) {
      _metrics = InspectorMetrics.capture(widget.appContext!);
    }
  }

  List<Inspectable> get _allInspectables {
    final items = <Inspectable>[...InspectableRegistry.instance.items];
    for (final c in _flameComponents) {
      if (!items.contains(c)) items.add(c);
    }
    return items;
  }

  List<Inspectable> get _filteredItems {
    if (_searchQuery.isEmpty) return _allInspectables;
    return _allInspectables.where((item) {
      return item.inspectorName.toLowerCase().contains(_searchQuery.toLowerCase()) ||
          item.inspectorCategory.toLowerCase().contains(_searchQuery.toLowerCase());
    }).toList();
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 360,
      decoration: BoxDecoration(
        color: Colors.black.withOpacity(0.95),
        border: const Border(
          left: BorderSide(color: Colors.cyanAccent, width: 2),
        ),
        boxShadow: [
          BoxShadow(
            color: Colors.cyanAccent.withOpacity(0.3),
            blurRadius: 20,
          ),
        ],
      ),
      child: Column(
        children: [
          _buildHeader(),
          _buildSearchBar(),
          _buildTabs(),
          Expanded(
            child: TabBarView(
              controller: _tabController,
              children: [
                _buildOverridesTab(),
                _buildComponentsTab(),
                _buildWidgetsTab(),
                _buildTreeTab(),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildHeader() {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        color: Colors.cyanAccent.withOpacity(0.15),
        border: const Border(
          bottom: BorderSide(color: Colors.cyanAccent, width: 1),
        ),
      ),
      child: Row(
        children: [
          const Icon(Icons.developer_mode, color: Colors.cyanAccent, size: 24),
          const SizedBox(width: 12),
          const Expanded(
            child: Text(
              'INSPECTOR',
              style: TextStyle(
                color: Colors.cyanAccent,
                fontSize: 18,
                fontWeight: FontWeight.bold,
                letterSpacing: 3,
              ),
            ),
          ),
          IconButton(
            icon: const Icon(Icons.refresh, color: Colors.cyanAccent, size: 20),
            onPressed: () {
              _discoverFlameComponents();
              _captureWidgetTree();
              _captureMetrics();
            },
            tooltip: 'Refresh',
          ),
        ],
      ),
    );
  }

  Widget _buildSearchBar() {
    return Padding(
      padding: const EdgeInsets.all(12),
      child: TextField(
        onChanged: (v) => setState(() => _searchQuery = v),
        style: const TextStyle(color: Colors.white, fontSize: 14),
        decoration: InputDecoration(
          hintText: 'Search...',
          hintStyle: TextStyle(color: Colors.grey.shade600),
          prefixIcon: Icon(Icons.search, color: Colors.cyanAccent.withOpacity(0.7), size: 20),
          filled: true,
          fillColor: Colors.white.withOpacity(0.05),
          contentPadding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
          border: OutlineInputBorder(
            borderRadius: BorderRadius.circular(8),
            borderSide: BorderSide(color: Colors.cyanAccent.withOpacity(0.3)),
          ),
          enabledBorder: OutlineInputBorder(
            borderRadius: BorderRadius.circular(8),
            borderSide: BorderSide(color: Colors.cyanAccent.withOpacity(0.3)),
          ),
          focusedBorder: OutlineInputBorder(
            borderRadius: BorderRadius.circular(8),
            borderSide: const BorderSide(color: Colors.cyanAccent),
          ),
        ),
      ),
    );
  }

  Widget _buildTabs() {
    return Container(
      decoration: BoxDecoration(
        border: Border(
          bottom: BorderSide(color: Colors.cyanAccent.withOpacity(0.3)),
        ),
      ),
      child: TabBar(
        controller: _tabController,
        indicatorColor: Colors.cyanAccent,
        labelColor: Colors.cyanAccent,
        unselectedLabelColor: Colors.grey,
        labelStyle: const TextStyle(fontSize: 11, fontWeight: FontWeight.bold, letterSpacing: 1),
        tabs: _tabs.map((t) => Tab(text: t.toUpperCase())).toList(),
      ),
    );
  }

  // ==================== OVERRIDES TAB (Live Editing) ====================

  Widget _buildOverridesTab() {
    final overrides = InspectorOverrides.instance;
    final registeredWidgets = overrides.registeredWidgets;
    
    if (registeredWidgets.isEmpty) {
      return _buildEmptyState(
        'No Editable Widgets',
        'Widgets register via InspectorOverrides.instance.register()',
      );
    }

    return ListView(
      padding: const EdgeInsets.all(8),
      children: [
        // Reset all button
        Padding(
          padding: const EdgeInsets.all(8),
          child: OutlinedButton.icon(
            onPressed: () => overrides.resetAll(),
            icon: const Icon(Icons.refresh, size: 16),
            label: const Text('RESET ALL'),
            style: OutlinedButton.styleFrom(
              foregroundColor: Colors.cyanAccent,
              side: BorderSide(color: Colors.cyanAccent.withOpacity(0.5)),
            ),
          ),
        ),
        const SizedBox(height: 8),
        ...registeredWidgets.map((widgetName) => _buildOverrideWidget(widgetName, overrides)),
      ],
    );
  }

  Widget _buildOverrideWidget(String widgetName, InspectorOverrides overrides) {
    final props = overrides.getWidgetProperties(widgetName);
    
    return Container(
      margin: const EdgeInsets.symmetric(horizontal: 4, vertical: 4),
      decoration: BoxDecoration(
        border: Border.all(color: Colors.cyanAccent.withOpacity(0.4)),
        borderRadius: BorderRadius.circular(4),
        color: Colors.cyanAccent.withOpacity(0.05),
      ),
      child: ExpansionTile(
        leading: const Icon(Icons.tune, size: 18, color: Colors.cyanAccent),
        title: Text(
          widgetName,
          style: const TextStyle(color: Colors.cyanAccent, fontSize: 13, fontWeight: FontWeight.bold),
        ),
        subtitle: Text(
          '${props.length} editable properties',
          style: TextStyle(color: Colors.grey.shade500, fontSize: 10),
        ),
        trailing: IconButton(
          icon: Icon(Icons.refresh, size: 16, color: Colors.cyanAccent.withOpacity(0.6)),
          onPressed: () => overrides.resetWidget(widgetName),
          tooltip: 'Reset',
        ),
        iconColor: Colors.cyanAccent,
        collapsedIconColor: Colors.cyanAccent.withOpacity(0.5),
        tilePadding: const EdgeInsets.symmetric(horizontal: 12),
        childrenPadding: const EdgeInsets.only(left: 12, right: 12, bottom: 12),
        initiallyExpanded: true,
        children: props.entries.map((entry) {
          return _buildOverrideEditor(widgetName, entry.key, entry.value, overrides);
        }).toList(),
      ),
    );
  }

  Widget _buildOverrideEditor(String widgetName, String propName, dynamic value, InspectorOverrides overrides) {
    final fullKey = '$widgetName.$propName';
    final hasOverride = overrides.hasOverride(fullKey);
    
    // Numeric values (double/int)
    if (value is num) {
      final isDouble = value is double;
      final displayValue = isDouble ? value : (value as int).toDouble();
      final min = _inferMin(propName, displayValue);
      final max = _inferMax(propName, displayValue);
      
      return Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 4),
            child: Row(
              children: [
                Expanded(
                  child: Text(
                    propName,
                    style: TextStyle(
                      color: hasOverride ? Colors.orangeAccent : Colors.white,
                      fontSize: 12,
                      fontWeight: hasOverride ? FontWeight.bold : FontWeight.normal,
                    ),
                  ),
                ),
                Text(
                  isDouble ? displayValue.toStringAsFixed(1) : displayValue.toInt().toString(),
                  style: const TextStyle(color: Colors.cyanAccent, fontSize: 12, fontFamily: 'monospace'),
                ),
                if (hasOverride)
                  IconButton(
                    icon: const Icon(Icons.undo, size: 14),
                    onPressed: () => overrides.reset(fullKey),
                    padding: EdgeInsets.zero,
                    constraints: const BoxConstraints(minWidth: 24, minHeight: 24),
                    color: Colors.orangeAccent,
                    tooltip: 'Reset to default',
                  ),
              ],
            ),
          ),
          SliderTheme(
            data: SliderThemeData(
              trackHeight: 3,
              thumbShape: const RoundSliderThumbShape(enabledThumbRadius: 7),
              overlayShape: const RoundSliderOverlayShape(overlayRadius: 14),
              activeTrackColor: hasOverride ? Colors.orangeAccent : Colors.cyanAccent,
              inactiveTrackColor: Colors.cyanAccent.withOpacity(0.2),
              thumbColor: hasOverride ? Colors.orangeAccent : Colors.cyanAccent,
            ),
            child: Slider(
              value: displayValue.clamp(min, max),
              min: min,
              max: max,
              onChanged: (v) {
                overrides.set(fullKey, isDouble ? v : v.round());
              },
            ),
          ),
        ],
      );
    }
    
    // Boolean values
    if (value is bool) {
      return SwitchListTile(
        title: Text(
          propName,
          style: TextStyle(
            color: hasOverride ? Colors.orangeAccent : Colors.white,
            fontSize: 12,
          ),
        ),
        value: value,
        activeColor: hasOverride ? Colors.orangeAccent : Colors.cyanAccent,
        dense: true,
        contentPadding: EdgeInsets.zero,
        onChanged: (v) => overrides.set(fullKey, v),
        secondary: hasOverride
            ? IconButton(
                icon: const Icon(Icons.undo, size: 14),
                onPressed: () => overrides.reset(fullKey),
                color: Colors.orangeAccent,
              )
            : null,
      );
    }
    
    // Color values
    if (value is Color) {
      return ListTile(
        contentPadding: EdgeInsets.zero,
        dense: true,
        title: Text(propName, style: const TextStyle(color: Colors.white, fontSize: 12)),
        trailing: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            GestureDetector(
              onTap: () => _showColorPicker(fullKey, value, overrides),
              child: Container(
                width: 28,
                height: 28,
                decoration: BoxDecoration(
                  color: value,
                  border: Border.all(color: Colors.white24),
                  borderRadius: BorderRadius.circular(4),
                ),
              ),
            ),
            if (hasOverride)
              IconButton(
                icon: const Icon(Icons.undo, size: 14),
                onPressed: () => overrides.reset(fullKey),
                color: Colors.orangeAccent,
              ),
          ],
        ),
      );
    }
    
    // String values (editable text)
    if (value is String) {
      return ListTile(
        contentPadding: EdgeInsets.zero,
        dense: true,
        title: Text(propName, style: TextStyle(color: hasOverride ? Colors.orangeAccent : Colors.white, fontSize: 12)),
        subtitle: Text(
          value,
          style: TextStyle(color: Colors.cyanAccent.withOpacity(0.8), fontSize: 11, fontFamily: 'monospace'),
          maxLines: 1,
          overflow: TextOverflow.ellipsis,
        ),
      );
    }
    
    // Default: read-only
    return ListTile(
      contentPadding: EdgeInsets.zero,
      dense: true,
      title: Text(propName, style: const TextStyle(color: Colors.white, fontSize: 12)),
      subtitle: Text(
        value.toString(),
        style: TextStyle(color: Colors.grey.shade500, fontSize: 11),
      ),
    );
  }

  double _inferMin(String propName, double currentValue) {
    final lowerName = propName.toLowerCase();
    if (lowerName.contains('opacity') || lowerName.contains('alpha')) return 0.0;
    if (lowerName.contains('scale')) return 0.1;
    if (lowerName.contains('rotation') || lowerName.contains('angle')) return -360.0;
    if (currentValue < 0) return currentValue * 2;
    return 0.0;
  }

  double _inferMax(String propName, double currentValue) {
    final lowerName = propName.toLowerCase();
    if (lowerName.contains('opacity') || lowerName.contains('alpha')) return 1.0;
    if (lowerName.contains('scale')) return 3.0;
    if (lowerName.contains('rotation') || lowerName.contains('angle')) return 360.0;
    if (lowerName.contains('size') || lowerName.contains('width') || lowerName.contains('height')) {
      return (currentValue * 3).clamp(100.0, 2000.0);
    }
    if (lowerName.contains('padding') || lowerName.contains('margin') || lowerName.contains('spacing')) {
      return (currentValue * 4).clamp(50.0, 200.0);
    }
    if (lowerName.contains('font')) return (currentValue * 3).clamp(8.0, 72.0);
    if (lowerName.contains('radius')) return (currentValue * 3).clamp(0.0, 100.0);
    return (currentValue * 2).clamp(currentValue + 10, 1000.0);
  }

  void _showColorPicker(String fullKey, Color current, InspectorOverrides overrides) {
    // Simple color preset picker
    final colors = [
      Colors.cyanAccent, Colors.cyan, Colors.teal,
      Colors.greenAccent, Colors.green, Colors.lightGreen,
      Colors.yellowAccent, Colors.yellow, Colors.orange,
      Colors.deepOrange, Colors.red, Colors.pink,
      Colors.purple, Colors.deepPurple, Colors.indigo,
      Colors.blue, Colors.lightBlue, Colors.blueAccent,
      Colors.white, Colors.grey, Colors.black,
    ];
    
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: Colors.grey.shade900,
        title: const Text('Pick Color', style: TextStyle(color: Colors.cyanAccent)),
        content: Wrap(
          spacing: 8,
          runSpacing: 8,
          children: colors.map((c) => GestureDetector(
            onTap: () {
              overrides.set(fullKey, c);
              Navigator.pop(ctx);
            },
            child: Container(
              width: 36,
              height: 36,
              decoration: BoxDecoration(
                color: c,
                border: Border.all(color: c == current ? Colors.white : Colors.white24, width: c == current ? 3 : 1),
                borderRadius: BorderRadius.circular(4),
              ),
            ),
          )).toList(),
        ),
      ),
    );
  }

  // ==================== COMPONENTS TAB ====================

  Widget _buildComponentsTab() {
    final items = _filteredItems;
    if (items.isEmpty) return _buildEmptyState('No Inspectable Items', 'Use InspectableMixin or FlutterInspectableMixin');

    final grouped = <String, List<Inspectable>>{};
    for (final item in items) {
      grouped.putIfAbsent(item.inspectorCategory, () => []).add(item);
    }

    return ListView(
      padding: const EdgeInsets.all(8),
      children: grouped.entries.map((entry) {
        return _buildCategorySection(entry.key, entry.value);
      }).toList(),
    );
  }

  Widget _buildCategorySection(String category, List<Inspectable> items) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 8),
          child: Row(
            children: [
              Container(
                width: 4,
                height: 16,
                color: Colors.cyanAccent,
              ),
              const SizedBox(width: 8),
              Text(
                category.toUpperCase(),
                style: TextStyle(
                  color: Colors.cyanAccent.withOpacity(0.8),
                  fontSize: 11,
                  fontWeight: FontWeight.bold,
                  letterSpacing: 2,
                ),
              ),
              const Spacer(),
              Text(
                '${items.length}',
                style: TextStyle(
                  color: Colors.cyanAccent.withOpacity(0.5),
                  fontSize: 11,
                ),
              ),
            ],
          ),
        ),
        ...items.map((item) => _buildInspectableTile(item)),
        const SizedBox(height: 8),
      ],
    );
  }

  Widget _buildInspectableTile(Inspectable item) {
    final properties = item.inspectorProperties;
    final index = _allInspectables.indexOf(item);
    final isSelected = _selectedIndex == index;

    return Container(
      margin: const EdgeInsets.symmetric(horizontal: 4, vertical: 2),
      decoration: BoxDecoration(
        border: Border.all(
          color: isSelected ? Colors.cyanAccent : Colors.cyanAccent.withOpacity(0.2),
          width: isSelected ? 2 : 1,
        ),
        borderRadius: BorderRadius.circular(4),
        color: isSelected ? Colors.cyanAccent.withOpacity(0.1) : null,
      ),
      child: ExpansionTile(
        leading: Icon(item.inspectorIcon, size: 18, color: Colors.cyanAccent.withOpacity(0.8)),
        title: Text(
          item.inspectorName,
          style: const TextStyle(color: Colors.cyanAccent, fontSize: 13, fontWeight: FontWeight.bold),
        ),
        subtitle: Text(
          '${properties.length} properties',
          style: TextStyle(color: Colors.grey.shade500, fontSize: 10),
        ),
        iconColor: Colors.cyanAccent,
        collapsedIconColor: Colors.cyanAccent.withOpacity(0.5),
        tilePadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 0),
        childrenPadding: const EdgeInsets.only(left: 12, right: 12, bottom: 12),
        onExpansionChanged: (expanded) => setState(() => _selectedIndex = expanded ? index : null),
        children: properties.isEmpty
            ? [const Text('No properties', style: TextStyle(color: Colors.grey, fontSize: 11))]
            : properties.map((prop) => _buildPropertyEditor(item, prop)).toList(),
      ),
    );
  }

  Widget _buildPropertyEditor(Inspectable item, InspectableProperty prop) {
    final value = prop.value;

    // Boolean
    if (value is bool) {
      return SwitchListTile(
        title: Text(prop.name, style: const TextStyle(color: Colors.white, fontSize: 12)),
        value: value,
        activeColor: Colors.cyanAccent,
        dense: true,
        contentPadding: EdgeInsets.zero,
        onChanged: prop.editable
            ? (v) => setState(() => item.onInspectorPropertyUpdate(prop.name, v))
            : null,
      );
    }

    // Number (double/int)
    if (value is num) {
      final min = prop.min ?? 0.0;
      final max = prop.max ?? 100.0;
      return Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.symmetric(vertical: 4),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(prop.name, style: const TextStyle(color: Colors.white, fontSize: 12)),
                Text(
                  value is int ? value.toString() : (value as double).toStringAsFixed(2),
                  style: const TextStyle(color: Colors.cyanAccent, fontSize: 12, fontFamily: 'monospace'),
                ),
              ],
            ),
          ),
          if (prop.editable)
            SliderTheme(
              data: SliderThemeData(
                trackHeight: 2,
                thumbShape: const RoundSliderThumbShape(enabledThumbRadius: 6),
                overlayShape: const RoundSliderOverlayShape(overlayRadius: 12),
                activeTrackColor: Colors.cyanAccent,
                inactiveTrackColor: Colors.cyanAccent.withOpacity(0.2),
                thumbColor: Colors.cyanAccent,
              ),
              child: Slider(
                value: value.toDouble().clamp(min, max),
                min: min,
                max: max,
                onChanged: (v) => setState(() {
                  item.onInspectorPropertyUpdate(prop.name, value is int ? v.round() : v);
                }),
              ),
            ),
        ],
      );
    }

    // Color
    if (value is Color) {
      return ListTile(
        contentPadding: EdgeInsets.zero,
        dense: true,
        title: Text(prop.name, style: const TextStyle(color: Colors.white, fontSize: 12)),
        trailing: Container(
          width: 24,
          height: 24,
          decoration: BoxDecoration(
            color: value,
            border: Border.all(color: Colors.white24),
            borderRadius: BorderRadius.circular(4),
          ),
        ),
      );
    }

    // Offset
    if (value is Offset) {
      return ListTile(
        contentPadding: EdgeInsets.zero,
        dense: true,
        title: Text(prop.name, style: const TextStyle(color: Colors.white, fontSize: 12)),
        subtitle: Text(
          'x: ${value.dx.toStringAsFixed(1)}, y: ${value.dy.toStringAsFixed(1)}',
          style: const TextStyle(color: Colors.cyanAccent, fontSize: 11, fontFamily: 'monospace'),
        ),
      );
    }

    // Size
    if (value is Size) {
      return ListTile(
        contentPadding: EdgeInsets.zero,
        dense: true,
        title: Text(prop.name, style: const TextStyle(color: Colors.white, fontSize: 12)),
        subtitle: Text(
          'w: ${value.width.toStringAsFixed(1)}, h: ${value.height.toStringAsFixed(1)}',
          style: const TextStyle(color: Colors.cyanAccent, fontSize: 11, fontFamily: 'monospace'),
        ),
      );
    }

    // Default: read-only text
    return ListTile(
      contentPadding: EdgeInsets.zero,
      dense: true,
      title: Text(prop.name, style: const TextStyle(color: Colors.white, fontSize: 12)),
      subtitle: Text(
        value.toString(),
        style: TextStyle(color: Colors.cyanAccent.withOpacity(0.7), fontSize: 11, fontFamily: 'monospace'),
        maxLines: 2,
        overflow: TextOverflow.ellipsis,
      ),
    );
  }

  // ==================== WIDGETS TAB ====================

  Widget _buildWidgetsTab() {
    // First show manually registered inspectables
    final manualWidgets = _allInspectables.where((i) => i.inspectorCategory == 'Flutter Widgets').toList();
    
    // Then show auto-discovered interesting widgets from tree, grouped by category
    final autoWidgets = _widgetTree?.allInterestingWidgets ?? [];
    
    // Group auto-discovered by category
    final grouped = <String, List<WidgetTreeNode>>{};
    for (final node in autoWidgets) {
      grouped.putIfAbsent(node.category, () => []).add(node);
    }
    
    // Prioritize App Widgets first
    final sortedCategories = grouped.keys.toList()
      ..sort((a, b) {
        if (a == 'App Widgets') return -1;
        if (b == 'App Widgets') return 1;
        return a.compareTo(b);
      });
    
    if (manualWidgets.isEmpty && autoWidgets.isEmpty) {
      return _buildEmptyState('No Widgets Found', 'Widgets will appear automatically');
    }
    
    return ListView(
      padding: const EdgeInsets.all(8),
      children: [
        if (manualWidgets.isNotEmpty) ...[
          _buildSectionHeader('REGISTERED WIDGETS', manualWidgets.length),
          ...manualWidgets.map((w) => _buildInspectableTile(w)),
          const SizedBox(height: 16),
        ],
        ...sortedCategories.map((category) {
          final nodes = grouped[category]!;
          return Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _buildSectionHeader(category.toUpperCase(), nodes.length),
              ...nodes.take(30).map((node) => _buildWidgetNodeTile(node)),
              if (nodes.length > 30)
                Padding(
                  padding: const EdgeInsets.all(8),
                  child: Text(
                    '... +${nodes.length - 30} more',
                    style: TextStyle(color: Colors.grey.shade600, fontSize: 11),
                  ),
                ),
              const SizedBox(height: 8),
            ],
          );
        }),
      ],
    );
  }

  Widget _buildSectionHeader(String title, int count) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 8),
      child: Row(
        children: [
          Container(width: 4, height: 16, color: Colors.cyanAccent),
          const SizedBox(width: 8),
          Text(
            title,
            style: TextStyle(
              color: Colors.cyanAccent.withOpacity(0.8),
              fontSize: 11,
              fontWeight: FontWeight.bold,
              letterSpacing: 2,
            ),
          ),
          const Spacer(),
          Text('$count', style: TextStyle(color: Colors.cyanAccent.withOpacity(0.5), fontSize: 11)),
        ],
      ),
    );
  }

  Widget _buildWidgetNodeTile(WidgetTreeNode node) {
    return Container(
      margin: const EdgeInsets.symmetric(horizontal: 4, vertical: 2),
      decoration: BoxDecoration(
        border: Border.all(color: Colors.cyanAccent.withOpacity(0.2)),
        borderRadius: BorderRadius.circular(4),
      ),
      child: ExpansionTile(
        leading: Icon(_getWidgetIcon(node.type), size: 16, color: _getTypeColor(node.type)),
        title: Text(
          node.name,
          style: TextStyle(color: _getTypeColor(node.type), fontSize: 12, fontWeight: FontWeight.bold),
        ),
        subtitle: node.size != null
            ? Text(
                '${node.size!.width.toInt()}×${node.size!.height.toInt()}',
                style: TextStyle(color: Colors.grey.shade500, fontSize: 10),
              )
            : null,
        iconColor: Colors.cyanAccent,
        collapsedIconColor: Colors.cyanAccent.withOpacity(0.5),
        tilePadding: const EdgeInsets.symmetric(horizontal: 12),
        childrenPadding: const EdgeInsets.only(left: 12, right: 12, bottom: 8),
        children: [
          if (node.size != null) _buildPropertyRow('Size', '${node.size!.width.toStringAsFixed(1)} × ${node.size!.height.toStringAsFixed(1)}'),
          if (node.renderBounds != null) _buildPropertyRow('Position', '(${node.renderBounds!.left.toInt()}, ${node.renderBounds!.top.toInt()})'),
          if (node.constraints != null) _buildPropertyRow('Constraints', node.constraints!.toString().replaceAll('BoxConstraints', '')),
          ...node.properties.entries.where((e) => e.value != null).map((e) => _buildPropertyRow(e.key, e.value.toString())),
        ],
      ),
    );
  }

  Widget _buildPropertyRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 100,
            child: Text(label, style: TextStyle(color: Colors.grey.shade400, fontSize: 11)),
          ),
          Expanded(
            child: Text(
              value,
              style: const TextStyle(color: Colors.cyanAccent, fontSize: 11, fontFamily: 'monospace'),
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
            ),
          ),
        ],
      ),
    );
  }

  IconData _getWidgetIcon(String type) {
    if (type.contains('Text')) return Icons.text_fields;
    if (type.contains('Container') || type.contains('Box')) return Icons.check_box_outline_blank;
    if (type.contains('Row')) return Icons.table_rows;
    if (type.contains('Column')) return Icons.view_column;
    if (type.contains('Stack')) return Icons.layers;
    if (type.contains('Button')) return Icons.smart_button;
    if (type.contains('Icon')) return Icons.star;
    if (type.contains('Image')) return Icons.image;
    if (type.contains('List') || type.contains('Scroll')) return Icons.view_list;
    if (type.contains('Padding')) return Icons.padding;
    if (type.contains('Card')) return Icons.credit_card;
    if (type.contains('Scaffold')) return Icons.web;
    if (type.contains('AppBar')) return Icons.web_asset;
    if (type.contains('Animated')) return Icons.animation;
    return Icons.widgets;
  }

  // ==================== TREE TAB ====================

  Widget _buildTreeTab() {
    if (_widgetTree == null) {
      return _buildEmptyState('No Widget Tree', 'Provide appContext to capture tree');
    }
    return ListView(
      padding: const EdgeInsets.all(8),
      children: [_buildTreeNode(_widgetTree!, 0)],
    );
  }

  Widget _buildTreeNode(WidgetTreeNode node, int depth) {
    if (depth > 10) return const SizedBox.shrink();

    final indent = depth * 12.0;
    final hasChildren = node.children.isNotEmpty;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: EdgeInsets.only(left: indent),
          child: Row(
            children: [
              if (hasChildren)
                Icon(Icons.arrow_right, size: 14, color: Colors.cyanAccent.withOpacity(0.5))
              else
                const SizedBox(width: 14),
              const SizedBox(width: 4),
              Expanded(
                child: Text(
                  node.name,
                  style: TextStyle(
                    color: _getTypeColor(node.type),
                    fontSize: 11,
                    fontFamily: 'monospace',
                  ),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
            ],
          ),
        ),
        ...node.children.take(20).map((c) => _buildTreeNode(c, depth + 1)),
        if (node.children.length > 20)
          Padding(
            padding: EdgeInsets.only(left: indent + 18),
            child: Text(
              '... +${node.children.length - 20} more',
              style: TextStyle(color: Colors.grey.shade600, fontSize: 10),
            ),
          ),
      ],
    );
  }

  Color _getTypeColor(String type) {
    if (type.contains('Text')) return Colors.greenAccent;
    if (type.contains('Container') || type.contains('Box')) return Colors.orangeAccent;
    if (type.contains('Row') || type.contains('Column') || type.contains('Flex')) return Colors.purpleAccent;
    if (type.contains('Button')) return Colors.redAccent;
    if (type.contains('Padding') || type.contains('Margin')) return Colors.yellowAccent;
    return Colors.cyanAccent.withOpacity(0.7);
  }

  // ==================== METRICS TAB ====================

  Widget _buildMetricsTab() {
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        _buildMetricCard('Performance Metrics', [
          _buildMetricRow('Widgets', _metrics?.widgetCount.toString() ?? '-'),
          _buildMetricRow('Elements', _metrics?.elementCount.toString() ?? '-'),
          _buildMetricRow('RenderObjects', _metrics?.renderObjectCount.toString() ?? '-'),
          _buildMetricRow('Dirty Elements', _metrics?.dirtyElements.toString() ?? '-'),
        ]),
        const SizedBox(height: 16),
        _buildMetricCard('Inspector Stats', [
          _buildMetricRow('Flame Components', _flameComponents.length.toString()),
          _buildMetricRow('Flutter Inspectables', InspectableRegistry.instance.items.length.toString()),
          _buildMetricRow('Total Items', _allInspectables.length.toString()),
        ]),
        const SizedBox(height: 16),
        _buildMetricCard('Categories', [
          ...InspectableRegistry.instance.itemsByCategory.entries.map(
            (e) => _buildMetricRow(e.key, e.value.length.toString()),
          ),
        ]),
      ],
    );
  }

  Widget _buildMetricCard(String title, List<Widget> children) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        border: Border.all(color: Colors.cyanAccent.withOpacity(0.3)),
        borderRadius: BorderRadius.circular(8),
        color: Colors.cyanAccent.withOpacity(0.05),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            title.toUpperCase(),
            style: TextStyle(
              color: Colors.cyanAccent.withOpacity(0.8),
              fontSize: 11,
              fontWeight: FontWeight.bold,
              letterSpacing: 2,
            ),
          ),
          const SizedBox(height: 12),
          ...children,
        ],
      ),
    );
  }

  Widget _buildMetricRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: TextStyle(color: Colors.grey.shade400, fontSize: 12)),
          Text(value, style: const TextStyle(color: Colors.cyanAccent, fontSize: 12, fontFamily: 'monospace')),
        ],
      ),
    );
  }

  // ==================== HELPERS ====================

  Widget _buildEmptyState(String title, String subtitle) {
    return Center(
      child: Padding(
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.search_off, color: Colors.grey.shade600, size: 48),
            const SizedBox(height: 16),
            Text(title, style: TextStyle(color: Colors.grey.shade400, fontSize: 14)),
            const SizedBox(height: 8),
            Text(
              subtitle,
              style: TextStyle(color: Colors.grey.shade600, fontSize: 11),
              textAlign: TextAlign.center,
            ),
          ],
        ),
      ),
    );
  }
}
