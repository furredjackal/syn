import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_animate/flutter_animate.dart';
import '../widgets/persona_container.dart';

/// World Map Overlay - shows city districts/regions for navigation
///
/// Props:
/// - onClose: Callback to close the overlay
/// - onRegionSelect: Callback when a region is selected
/// - regions: List of available regions
class WorldMapOverlay extends StatefulWidget {
  final VoidCallback onClose;
  final Function(String regionId) onRegionSelect;
  final List<MapRegion> regions;

  const WorldMapOverlay({
    super.key,
    required this.onClose,
    required this.onRegionSelect,
    this.regions = const [],
  });

  @override
  State<WorldMapOverlay> createState() => _WorldMapOverlayState();
}

class _WorldMapOverlayState extends State<WorldMapOverlay> {
  int _selectedIndex = 0;

  @override
  Widget build(BuildContext context) {
    final screenWidth = MediaQuery.of(context).size.width;
    final screenHeight = MediaQuery.of(context).size.height;

    return KeyboardListener(
      focusNode: FocusNode()..requestFocus(),
      onKeyEvent: _handleKeyEvent,
      child: Container(
        color: Colors.black.withValues(alpha: 0.92),
        child: Stack(
          children: [
            // Title
            Positioned(
              left: 60,
              top: 40,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    'WORLD MAP',
                    style: TextStyle(
                      color: Colors.white,
                      fontSize: 48,
                      fontWeight: FontWeight.w900,
                      letterSpacing: 4,
                      shadows: [
                        Shadow(
                          color: Colors.cyanAccent.withValues(alpha: 0.5),
                          blurRadius: 15,
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(height: 8),
                  Text(
                    'Select a location to visit',
                    style: TextStyle(
                      color: Colors.white.withValues(alpha: 0.7),
                      fontSize: 16,
                      letterSpacing: 1.2,
                    ),
                  ),
                ],
              )
                  .animate()
                  .fadeIn(duration: 400.ms)
                  .slideY(begin: -0.2, duration: 400.ms, curve: Curves.easeOut),
            ),

            // Region Grid
            Center(
              child: Container(
                constraints: BoxConstraints(
                  maxWidth: screenWidth * 0.8,
                  maxHeight: screenHeight * 0.6,
                ),
                child: widget.regions.isEmpty
                    ? _buildPlaceholderRegions()
                    : _buildRegionGrid(),
              )
                  .animate()
                  .fadeIn(delay: 200.ms, duration: 600.ms)
                  .scale(
                      begin: const Offset(0.95, 0.95),
                      duration: 600.ms,
                      curve: Curves.easeOutExpo),
            ),

            // Close Button (ESC hint)
            Positioned(
              right: 60,
              top: 40,
              child: _buildCloseButton()
                  .animate()
                  .fadeIn(delay: 300.ms, duration: 400.ms),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildRegionGrid() {
    return GridView.builder(
      padding: const EdgeInsets.all(20),
      gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
        crossAxisCount: 3,
        crossAxisSpacing: 24,
        mainAxisSpacing: 24,
        childAspectRatio: 1.2,
      ),
      itemCount: widget.regions.length,
      itemBuilder: (context, index) {
        final region = widget.regions[index];
        final isSelected = index == _selectedIndex;

        return _buildRegionCard(
          region: region,
          isSelected: isSelected,
          onTap: () => _selectRegion(index),
        );
      },
    );
  }

  Widget _buildPlaceholderRegions() {
    // Default regions if none provided
    final placeholderRegions = [
      MapRegion(id: 'downtown', name: 'Downtown', description: 'City center'),
      MapRegion(id: 'industrial', name: 'Industrial District', description: 'Factories and warehouses'),
      MapRegion(id: 'residential', name: 'Residential Area', description: 'Suburbs and apartments'),
      MapRegion(id: 'red_light', name: 'Red Light District', description: 'Nightlife and entertainment'),
      MapRegion(id: 'corporate', name: 'Corporate Zone', description: 'Office towers'),
      MapRegion(id: 'slums', name: 'The Slums', description: 'Poor district'),
    ];

    return GridView.builder(
      padding: const EdgeInsets.all(20),
      gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
        crossAxisCount: 3,
        crossAxisSpacing: 24,
        mainAxisSpacing: 24,
        childAspectRatio: 1.2,
      ),
      itemCount: placeholderRegions.length,
      itemBuilder: (context, index) {
        final region = placeholderRegions[index];
        final isSelected = index == _selectedIndex;

        return _buildRegionCard(
          region: region,
          isSelected: isSelected,
          onTap: () => _selectRegion(index),
        );
      },
    );
  }

  Widget _buildRegionCard({
    required MapRegion region,
    required bool isSelected,
    required VoidCallback onTap,
  }) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        onTap: onTap,
        child: PersonaContainer(
          color: isSelected ? Colors.cyanAccent : Colors.black,
          borderColor: isSelected ? Colors.cyanAccent : Colors.white.withValues(alpha: 0.3),
          borderWidth: isSelected ? 3 : 2,
          skew: -0.15,
          child: Container(
            padding: const EdgeInsets.all(20),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  region.name,
                  style: TextStyle(
                    color: isSelected ? Colors.black : Colors.white,
                    fontSize: 22,
                    fontWeight: FontWeight.w800,
                    letterSpacing: 1.5,
                  ),
                ),
                const SizedBox(height: 8),
                Text(
                  region.description,
                  style: TextStyle(
                    color: isSelected
                        ? Colors.black.withValues(alpha: 0.7)
                        : Colors.white.withValues(alpha: 0.6),
                    fontSize: 14,
                    letterSpacing: 0.5,
                  ),
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                ),
              ],
            ),
          ),
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
          color: Colors.black,
          borderColor: Colors.redAccent.withValues(alpha: 0.5),
          skew: -0.15,
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 12),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(
                  'ESC',
                  style: TextStyle(
                    color: Colors.white.withValues(alpha: 0.7),
                    fontSize: 14,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const SizedBox(width: 8),
                const Text(
                  'CLOSE',
                  style: TextStyle(
                    color: Colors.white,
                    fontSize: 16,
                    fontWeight: FontWeight.w700,
                    letterSpacing: 1.2,
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  void _handleKeyEvent(KeyEvent event) {
    if (event is! KeyDownEvent) return;

    if (event.logicalKey == LogicalKeyboardKey.escape) {
      widget.onClose();
    } else if (event.logicalKey == LogicalKeyboardKey.arrowUp) {
      setState(() {
        _selectedIndex = (_selectedIndex - 3).clamp(0, widget.regions.length - 1);
      });
    } else if (event.logicalKey == LogicalKeyboardKey.arrowDown) {
      setState(() {
        _selectedIndex = (_selectedIndex + 3).clamp(0, widget.regions.length - 1);
      });
    } else if (event.logicalKey == LogicalKeyboardKey.arrowLeft) {
      setState(() {
        _selectedIndex = (_selectedIndex - 1).clamp(0, widget.regions.length - 1);
      });
    } else if (event.logicalKey == LogicalKeyboardKey.arrowRight) {
      setState(() {
        _selectedIndex = (_selectedIndex + 1).clamp(0, widget.regions.length - 1);
      });
    } else if (event.logicalKey == LogicalKeyboardKey.enter ||
        event.logicalKey == LogicalKeyboardKey.space) {
      _selectRegion(_selectedIndex);
    }
  }

  void _selectRegion(int index) {
    setState(() {
      _selectedIndex = index;
    });

    final regionId = widget.regions.isEmpty
        ? ['downtown', 'industrial', 'residential', 'red_light', 'corporate', 'slums'][index]
        : widget.regions[index].id;

    widget.onRegionSelect(regionId);
  }
}

/// Data model for map regions
class MapRegion {
  final String id;
  final String name;
  final String description;
  final bool isLocked;

  const MapRegion({
    required this.id,
    required this.name,
    required this.description,
    this.isLocked = false,
  });
}
