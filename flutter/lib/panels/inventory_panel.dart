import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../ui/syn_ui.dart';

/// Inventory Panel - displays character's possessions and items
/// with Persona 5 × Destiny 2 inspired styling.
///
/// Features:
/// - Staggered grid entrance animation
/// - Hover glow on items
/// - Smooth category filtering
/// - Detail panel with parallax effect
class InventoryPanel extends StatefulWidget {
  final VoidCallback onClose;
  final List<InventoryItem> items;
  final Function(InventoryItem)? onItemSelect;

  const InventoryPanel({
    super.key,
    required this.onClose,
    required this.items,
    this.onItemSelect,
  });

  @override
  State<InventoryPanel> createState() => _InventoryPanelState();
}

class _InventoryPanelState extends State<InventoryPanel>
    with SingleTickerProviderStateMixin {
  int _selectedIndex = 0;
  String _filter = 'all';
  late AnimationController _entranceController;
  late Animation<double> _backdropAnimation;
  late Animation<Offset> _panelSlideAnimation;

  List<InventoryItem> get _filteredItems {
    if (_filter == 'all') return widget.items;
    return widget.items.where((item) => item.category == _filter).toList();
  }

  @override
  void initState() {
    super.initState();
    _entranceController = AnimationController(
      duration: SynTheme.slow,
      vsync: this,
    );

    _backdropAnimation = Tween<double>(begin: 0, end: 1).animate(
      CurvedAnimation(
        parent: _entranceController,
        curve: const Interval(0, 0.5, curve: Curves.easeOut),
      ),
    );

    _panelSlideAnimation = Tween<Offset>(
      begin: const Offset(0, 1.2),
      end: Offset.zero,
    ).animate(CurvedAnimation(
      parent: _entranceController,
      curve: SynTheme.snapIn,
    ));

    _entranceController.forward();
  }

  @override
  void dispose() {
    _entranceController.dispose();
    super.dispose();
  }

  Future<void> _animateClose() async {
    await _entranceController.reverse();
    widget.onClose();
  }

  @override
  Widget build(BuildContext context) {
    return KeyboardListener(
      focusNode: FocusNode()..requestFocus(),
      onKeyEvent: _handleKeyEvent,
      child: AnimatedBuilder(
        animation: _entranceController,
        builder: (context, _) {
          return Container(
            color: Colors.black.withOpacity(0.9 * _backdropAnimation.value),
            child: Center(
              child: SlideTransition(
                position: _panelSlideAnimation,
                child: ConstrainedBox(
                  constraints:
                      const BoxConstraints(maxWidth: 950, maxHeight: 850),
                  child: SynContainer(
                    enableHover: false,
                    child: Padding(
                      padding: const EdgeInsets.all(40.0),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.stretch,
                        children: [
                          _buildHeader(),
                          const SizedBox(height: 24),
                          _buildFilterBar(),
                          const SizedBox(height: 24),
                          Expanded(
                            child: Row(
                              crossAxisAlignment: CrossAxisAlignment.start,
                              children: [
                                Expanded(flex: 2, child: _buildItemGrid()),
                                const SizedBox(width: 24),
                                Expanded(child: _buildItemDetail()),
                              ],
                            ),
                          ),
                          const SizedBox(height: 24),
                          _buildCloseButton(),
                        ],
                      ),
                    ),
                  ),
                ),
              ),
            ),
          );
        },
      ),
    );
  }

  Widget _buildHeader() {
    return SynStaggeredEntrance(
      index: 0,
      child: Row(
        children: [
          Icon(Icons.inventory_2, color: SynTheme.accent, size: 44),
          const SizedBox(width: 16),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('INVENTORY', style: SynTheme.display()),
                const SizedBox(height: 4),
                Text(
                  '${widget.items.length} items',
                  style: SynTheme.caption(color: SynTheme.textMuted),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildFilterBar() {
    final filters = ['all', 'key', 'consumable', 'equipment', 'misc'];
    final labels = ['ALL', 'KEY ITEMS', 'CONSUMABLE', 'EQUIPMENT', 'MISC'];

    return SynStaggeredEntrance(
      index: 1,
      child: SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        child: Row(
          children: filters.asMap().entries.map((entry) {
            return Padding(
              padding: const EdgeInsets.only(right: 12.0),
              child: _FilterChip(
                label: labels[entry.key],
                isActive: _filter == entry.value,
                onTap: () => setState(() {
                  _filter = entry.value;
                  _selectedIndex = 0;
                }),
              ),
            );
          }).toList(),
        ),
      ),
    );
  }

  Widget _buildItemGrid() {
    final items = _filteredItems;

    if (items.isEmpty) {
      return Center(
        child: Text(
          'No items found.',
          style: SynTheme.body(color: SynTheme.textMuted),
        ),
      );
    }

    return GridView.builder(
      gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
        crossAxisCount: 3,
        crossAxisSpacing: 16,
        mainAxisSpacing: 16,
        childAspectRatio: 1,
      ),
      itemCount: items.length,
      itemBuilder: (context, index) {
        return SynStaggeredEntrance(
          index: index + 2,
          staggerDelay: const Duration(milliseconds: 30),
          child: _ItemCell(
            item: items[index],
            isSelected: _selectedIndex == index,
            onHover: () => setState(() => _selectedIndex = index),
            onTap: () {
              setState(() => _selectedIndex = index);
              widget.onItemSelect?.call(items[index]);
            },
          ),
        );
      },
    );
  }

  Widget _buildItemDetail() {
    if (_filteredItems.isEmpty) return const SizedBox();

    final item = _filteredItems[_selectedIndex];

    return SynStaggeredEntrance(
      index: 3,
      slideFrom: const Offset(0.3, 0),
      child: SynContainer(
        enableHover: false,
        skew: -0.08,
        child: Padding(
          padding: const EdgeInsets.all(24.0),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Icon(
                _getIconForCategory(item.category),
                color: SynTheme.accent,
                size: 52,
              ),
              const SizedBox(height: 16),
              Text(item.name, style: SynTheme.headline()),
              const SizedBox(height: 8),
              Text(
                item.category.toUpperCase(),
                style: SynTheme.caption(color: SynTheme.accent),
              ),
              const SizedBox(height: 16),
              Container(
                height: 1,
                color: SynTheme.accent.withOpacity(0.3),
              ),
              const SizedBox(height: 16),
              Expanded(
                child: SingleChildScrollView(
                  child: Text(
                    item.description,
                    style: SynTheme.body(color: SynTheme.textSecondary),
                  ),
                ),
              ),
              if (item.quantity > 1) ...[
                const SizedBox(height: 12),
                Container(
                  padding:
                      const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
                  decoration: BoxDecoration(
                    border: Border.all(color: SynTheme.accent),
                    color: SynTheme.accent.withOpacity(0.1),
                  ),
                  child: Text(
                    'x${item.quantity}',
                    style: SynTheme.label(color: SynTheme.accent),
                  ),
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildCloseButton() {
    return SynStaggeredEntrance(
      index: 15,
      slideFrom: const Offset(0, 0.3),
      child: Center(
        child: SynButton(
          label: 'CLOSE',
          icon: Icons.close,
          style: SynButtonStyle.secondary,
          onPressed: _animateClose,
        ),
      ),
    );
  }

  IconData _getIconForCategory(String category) {
    switch (category) {
      case 'key':
        return Icons.vpn_key;
      case 'consumable':
        return Icons.local_drink;
      case 'equipment':
        return Icons.shield;
      case 'misc':
        return Icons.category;
      default:
        return Icons.inventory_2;
    }
  }

  void _handleKeyEvent(KeyEvent event) {
    if (event is KeyDownEvent) {
      final items = _filteredItems;
      if (items.isEmpty) return;

      const itemsPerRow = 3;
      final totalRows = (items.length / itemsPerRow).ceil();
      final currentRow = _selectedIndex ~/ itemsPerRow;
      final currentCol = _selectedIndex % itemsPerRow;

      if (event.logicalKey == LogicalKeyboardKey.arrowUp ||
          event.logicalKey == LogicalKeyboardKey.keyW) {
        final newRow = (currentRow - 1 + totalRows) % totalRows;
        final newIndex = newRow * itemsPerRow + currentCol;
        setState(() => _selectedIndex = newIndex.clamp(0, items.length - 1));
        HapticFeedback.selectionClick();
      } else if (event.logicalKey == LogicalKeyboardKey.arrowDown ||
          event.logicalKey == LogicalKeyboardKey.keyS) {
        final newRow = (currentRow + 1) % totalRows;
        final newIndex = newRow * itemsPerRow + currentCol;
        setState(() => _selectedIndex = newIndex.clamp(0, items.length - 1));
        HapticFeedback.selectionClick();
      } else if (event.logicalKey == LogicalKeyboardKey.arrowLeft ||
          event.logicalKey == LogicalKeyboardKey.keyA) {
        setState(() =>
            _selectedIndex = (_selectedIndex - 1 + items.length) % items.length);
        HapticFeedback.selectionClick();
      } else if (event.logicalKey == LogicalKeyboardKey.arrowRight ||
          event.logicalKey == LogicalKeyboardKey.keyD) {
        setState(() => _selectedIndex = (_selectedIndex + 1) % items.length);
        HapticFeedback.selectionClick();
      } else if (event.logicalKey == LogicalKeyboardKey.enter ||
          event.logicalKey == LogicalKeyboardKey.space) {
        widget.onItemSelect?.call(items[_selectedIndex]);
      } else if (event.logicalKey == LogicalKeyboardKey.escape) {
        _animateClose();
      }
    }
  }
}

/// Filter chip with hover effects
class _FilterChip extends StatefulWidget {
  final String label;
  final bool isActive;
  final VoidCallback onTap;

  const _FilterChip({
    required this.label,
    required this.isActive,
    required this.onTap,
  });

  @override
  State<_FilterChip> createState() => _FilterChipState();
}

class _FilterChipState extends State<_FilterChip> {
  bool _isHovered = false;

  @override
  Widget build(BuildContext context) {
    final isHighlighted = widget.isActive || _isHovered;

    return MouseRegion(
      onEnter: (_) => setState(() => _isHovered = true),
      onExit: (_) => setState(() => _isHovered = false),
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        onTap: () {
          HapticFeedback.selectionClick();
          widget.onTap();
        },
        child: AnimatedContainer(
          duration: SynTheme.fast,
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 10),
          decoration: BoxDecoration(
            color: widget.isActive
                ? SynTheme.accent.withOpacity(0.25)
                : _isHovered
                    ? SynTheme.accent.withOpacity(0.1)
                    : SynTheme.bgCard,
            border: Border.all(
              color: isHighlighted
                  ? SynTheme.accent
                  : SynTheme.accent.withOpacity(0.3),
            ),
            boxShadow: [
              if (isHighlighted)
                BoxShadow(
                  color: SynTheme.accent.withOpacity(0.2),
                  blurRadius: 10,
                ),
            ],
          ),
          child: Text(
            widget.label,
            style: SynTheme.caption(
              color: isHighlighted ? SynTheme.accent : SynTheme.textSecondary,
            ),
          ),
        ),
      ),
    );
  }
}

/// Item cell with hover glow
class _ItemCell extends StatefulWidget {
  final InventoryItem item;
  final bool isSelected;
  final VoidCallback onHover;
  final VoidCallback onTap;

  const _ItemCell({
    required this.item,
    required this.isSelected,
    required this.onHover,
    required this.onTap,
  });

  @override
  State<_ItemCell> createState() => _ItemCellState();
}

class _ItemCellState extends State<_ItemCell> {
  bool _isHovered = false;

  IconData _getIcon(String category) {
    switch (category) {
      case 'key':
        return Icons.vpn_key;
      case 'consumable':
        return Icons.local_drink;
      case 'equipment':
        return Icons.shield;
      case 'misc':
        return Icons.category;
      default:
        return Icons.inventory_2;
    }
  }

  @override
  Widget build(BuildContext context) {
    final isHighlighted = widget.isSelected || _isHovered;

    return MouseRegion(
      onEnter: (_) {
        setState(() => _isHovered = true);
        widget.onHover();
      },
      onExit: (_) => setState(() => _isHovered = false),
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: SynTheme.fast,
          curve: SynTheme.snapIn,
          transform: Matrix4.identity()
            ..translate(
              _isHovered ? -2.0 : 0.0,
              _isHovered ? -2.0 : 0.0,
            ),
          decoration: BoxDecoration(
            color: isHighlighted
                ? SynTheme.accent.withOpacity(0.15)
                : SynTheme.bgCard,
            border: Border.all(
              color: widget.isSelected
                  ? SynTheme.accent
                  : _isHovered
                      ? SynTheme.accent.withOpacity(0.7)
                      : SynTheme.accent.withOpacity(0.2),
              width: widget.isSelected ? 2 : 1,
            ),
            boxShadow: [
              if (isHighlighted)
                BoxShadow(
                  color: SynTheme.accent.withOpacity(0.3),
                  blurRadius: 15,
                  spreadRadius: -3,
                ),
              BoxShadow(
                color: Colors.black.withOpacity(0.5),
                offset: Offset(
                  _isHovered ? 4 : 2,
                  _isHovered ? 4 : 2,
                ),
                blurRadius: 0,
              ),
            ],
          ),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Icon(
                _getIcon(widget.item.category),
                color: isHighlighted ? SynTheme.accent : SynTheme.textSecondary,
                size: 42,
              ),
              const SizedBox(height: 10),
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 10.0),
                child: Text(
                  widget.item.name,
                  style: SynTheme.caption(
                    color: isHighlighted
                        ? SynTheme.textPrimary
                        : SynTheme.textSecondary,
                  ),
                  textAlign: TextAlign.center,
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              if (widget.item.quantity > 1) ...[
                const SizedBox(height: 6),
                Container(
                  padding:
                      const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                  decoration: BoxDecoration(
                    color: SynTheme.accent.withOpacity(0.2),
                    border: Border.all(color: SynTheme.accent.withOpacity(0.5)),
                  ),
                  child: Text(
                    'x${widget.item.quantity}',
                    style: SynTheme.caption(color: SynTheme.accent),
                  ),
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }
}

/// Data model for inventory items
class InventoryItem {
  final String name;
  final String description;
  final String category; // key, consumable, equipment, misc
  final int quantity;

  const InventoryItem({
    required this.name,
    required this.description,
    required this.category,
    this.quantity = 1,
  });
}
