import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_animate/flutter_animate.dart';
import '../ui/widgets/persona_container.dart';

/// Inventory Panel - displays character's possessions and items
/// 
/// Props:
/// - onClose: Callback to close the panel
/// - items: List of inventory items
/// - onItemSelect: Callback when an item is selected
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

class _InventoryPanelState extends State<InventoryPanel> {
  int _selectedIndex = 0;
  String _filter = 'all'; // all, key, consumable, equipment, misc

  List<InventoryItem> get _filteredItems {
    if (_filter == 'all') return widget.items;
    return widget.items.where((item) => item.category == _filter).toList();
  }

  @override
  Widget build(BuildContext context) {
    return KeyboardListener(
      focusNode: FocusNode()..requestFocus(),
      onKeyEvent: _handleKeyEvent,
      child: Container(
        color: Colors.black.withOpacity(0.85),
        child: Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 900, maxHeight: 800),
            child: PersonaContainer(
              skew: -0.15,
              color: Colors.black.withOpacity(0.95),
              child: Padding(
                padding: const EdgeInsets.all(40.0),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    _buildHeader(),
                    const SizedBox(height: 20),
                    _buildFilterBar(),
                    const SizedBox(height: 20),
                    Expanded(
                      child: Row(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Expanded(
                            flex: 2,
                            child: _buildItemGrid(),
                          ),
                          const SizedBox(width: 20),
                          Expanded(
                            flex: 1,
                            child: _buildItemDetail(),
                          ),
                        ],
                      ),
                    ),
                    const SizedBox(height: 20),
                    _buildCloseButton(),
                  ],
                ),
              ),
            ),
          )
              .animate()
              .slideY(begin: 1.0, duration: 400.ms, curve: Curves.easeOut)
              .fadeIn(duration: 300.ms),
        ),
      ),
    );
  }

  Widget _buildHeader() {
    return Row(
      children: [
        Icon(
          Icons.inventory_2,
          color: const Color(0xFF00E6FF),
          size: 40,
        ),
        const SizedBox(width: 15),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'INVENTORY',
                style: TextStyle(
                  fontSize: 36,
                  fontWeight: FontWeight.w900,
                  color: const Color(0xFF00E6FF),
                  letterSpacing: 4,
                  shadows: [
                    Shadow(
                      color: const Color(0xFF00E6FF).withOpacity(0.5),
                      blurRadius: 15,
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 5),
              Text(
                '${widget.items.length} items',
                style: const TextStyle(
                  fontSize: 14,
                  color: Colors.white60,
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildFilterBar() {
    return Row(
      children: [
        _buildFilterChip('ALL', 'all'),
        const SizedBox(width: 10),
        _buildFilterChip('KEY ITEMS', 'key'),
        const SizedBox(width: 10),
        _buildFilterChip('CONSUMABLE', 'consumable'),
        const SizedBox(width: 10),
        _buildFilterChip('EQUIPMENT', 'equipment'),
        const SizedBox(width: 10),
        _buildFilterChip('MISC', 'misc'),
      ],
    );
  }

  Widget _buildFilterChip(String label, String filterValue) {
    final isActive = _filter == filterValue;
    
    return GestureDetector(
      onTap: () => setState(() {
        _filter = filterValue;
        _selectedIndex = 0;
      }),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 15, vertical: 8),
        decoration: BoxDecoration(
          color: isActive
              ? const Color(0xFF00E6FF).withOpacity(0.3)
              : Colors.white.withOpacity(0.1),
          border: Border.all(
            color: isActive ? const Color(0xFF00E6FF) : Colors.white30,
            width: 2,
          ),
        ),
        child: Text(
          label,
          style: TextStyle(
            fontSize: 12,
            fontWeight: FontWeight.w700,
            color: isActive ? const Color(0xFF00E6FF) : Colors.white70,
          ),
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
          style: TextStyle(
            fontSize: 18,
            color: Colors.white30,
            fontStyle: FontStyle.italic,
          ),
        ),
      );
    }

    return GridView.builder(
      gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
        crossAxisCount: 3,
        crossAxisSpacing: 15,
        mainAxisSpacing: 15,
        childAspectRatio: 1,
      ),
      itemCount: items.length,
      itemBuilder: (context, index) {
        return _buildItemCell(items[index], index);
      },
    );
  }

  Widget _buildItemCell(InventoryItem item, int index) {
    final isSelected = _selectedIndex == index;

    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _selectedIndex = index),
      child: GestureDetector(
        onTap: () {
          setState(() => _selectedIndex = index);
          if (widget.onItemSelect != null) {
            widget.onItemSelect!(item);
          }
        },
        child: Container(
          decoration: BoxDecoration(
            color: isSelected
                ? const Color(0xFF00E6FF).withOpacity(0.2)
                : Colors.black.withOpacity(0.5),
            border: Border.all(
              color: isSelected ? const Color(0xFF00E6FF) : Colors.white30,
              width: isSelected ? 3 : 1,
            ),
            boxShadow: isSelected
                ? [
                    BoxShadow(
                      color: const Color(0xFF00E6FF).withOpacity(0.5),
                      blurRadius: 10,
                    ),
                  ]
                : null,
          ),
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              Icon(
                _getIconForCategory(item.category),
                color: isSelected ? const Color(0xFF00E6FF) : Colors.white70,
                size: 40,
              ),
              const SizedBox(height: 8),
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 8.0),
                child: Text(
                  item.name,
                  style: TextStyle(
                    fontSize: 12,
                    fontWeight: FontWeight.w600,
                    color: isSelected ? Colors.white : Colors.white70,
                  ),
                  textAlign: TextAlign.center,
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              if (item.quantity > 1) ...[
                const SizedBox(height: 4),
                Text(
                  'x${item.quantity}',
                  style: TextStyle(
                    fontSize: 10,
                    color: const Color(0xFF00E6FF),
                    fontWeight: FontWeight.w700,
                  ),
                ),
              ],
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildItemDetail() {
    if (_filteredItems.isEmpty) return const SizedBox();
    
    final item = _filteredItems[_selectedIndex];

    return PersonaContainer(
      skew: -0.1,
      color: Colors.black.withOpacity(0.7),
      child: Padding(
        padding: const EdgeInsets.all(20.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Icon(
              _getIconForCategory(item.category),
              color: const Color(0xFF00E6FF),
              size: 50,
            ),
            const SizedBox(height: 15),
            Text(
              item.name,
              style: const TextStyle(
                fontSize: 22,
                fontWeight: FontWeight.w900,
                color: Colors.white,
              ),
            ),
            const SizedBox(height: 8),
            Text(
              item.category.toUpperCase(),
              style: const TextStyle(
                fontSize: 12,
                color: Color(0xFF00E6FF),
                letterSpacing: 1.5,
              ),
            ),
            const SizedBox(height: 15),
            const Divider(color: Colors.white30),
            const SizedBox(height: 15),
            Expanded(
              child: SingleChildScrollView(
                child: Text(
                  item.description,
                  style: const TextStyle(
                    fontSize: 14,
                    color: Colors.white70,
                    height: 1.5,
                  ),
                ),
              ),
            ),
            if (item.quantity > 1) ...[
              const SizedBox(height: 10),
              Text(
                'Quantity: ${item.quantity}',
                style: const TextStyle(
                  fontSize: 14,
                  color: Color(0xFF00E6FF),
                  fontWeight: FontWeight.w700,
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildCloseButton() {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        onTap: widget.onClose,
        child: PersonaContainer(
          skew: -0.18,
          color: Colors.black.withOpacity(0.6),
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: 15, horizontal: 25),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                const Icon(Icons.close, color: Colors.white70, size: 22),
                const SizedBox(width: 10),
                Text(
                  'CLOSE',
                  style: const TextStyle(
                    fontSize: 20,
                    fontWeight: FontWeight.w900,
                    color: Colors.white,
                    letterSpacing: 2,
                  ),
                ),
              ],
            ),
          ),
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

      final itemsPerRow = 3;
      final totalRows = (items.length / itemsPerRow).ceil();
      final currentRow = _selectedIndex ~/ itemsPerRow;
      final currentCol = _selectedIndex % itemsPerRow;

      if (event.logicalKey == LogicalKeyboardKey.arrowUp ||
          event.logicalKey == LogicalKeyboardKey.keyW) {
        final newRow = (currentRow - 1) % totalRows;
        final newIndex = newRow < 0 ? items.length - 1 : newRow * itemsPerRow + currentCol;
        setState(() {
          _selectedIndex = newIndex.clamp(0, items.length - 1);
        });
      } else if (event.logicalKey == LogicalKeyboardKey.arrowDown ||
          event.logicalKey == LogicalKeyboardKey.keyS) {
        final newRow = (currentRow + 1) % totalRows;
        final newIndex = newRow * itemsPerRow + currentCol;
        setState(() {
          _selectedIndex = newIndex.clamp(0, items.length - 1);
        });
      } else if (event.logicalKey == LogicalKeyboardKey.arrowLeft ||
          event.logicalKey == LogicalKeyboardKey.keyA) {
        setState(() {
          _selectedIndex = (_selectedIndex - 1) % items.length;
          if (_selectedIndex < 0) _selectedIndex = items.length - 1;
        });
      } else if (event.logicalKey == LogicalKeyboardKey.arrowRight ||
          event.logicalKey == LogicalKeyboardKey.keyD) {
        setState(() {
          _selectedIndex = (_selectedIndex + 1) % items.length;
        });
      } else if (event.logicalKey == LogicalKeyboardKey.enter ||
          event.logicalKey == LogicalKeyboardKey.space) {
        if (widget.onItemSelect != null) {
          widget.onItemSelect!(items[_selectedIndex]);
        }
      } else if (event.logicalKey == LogicalKeyboardKey.escape) {
        widget.onClose();
      }
    }
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
