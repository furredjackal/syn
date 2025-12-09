import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_animate/flutter_animate.dart';
import '../ui/widgets/persona_container.dart';

/// Detailed Stats Panel - displays comprehensive character statistics
/// 
/// Props:
/// - onClose: Callback to close the panel
/// - stats: Map of stat categories and values
class DetailedStatsPanel extends StatefulWidget {
  final VoidCallback onClose;
  final Map<String, Map<String, double>> stats;

  const DetailedStatsPanel({
    super.key,
    required this.onClose,
    required this.stats,
  });

  @override
  State<DetailedStatsPanel> createState() => _DetailedStatsPanelState();
}

class _DetailedStatsPanelState extends State<DetailedStatsPanel> {
  String _selectedCategory = 'core';

  List<String> get _categories => widget.stats.keys.toList();

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
                    _buildCategoryTabs(),
                    const SizedBox(height: 20),
                    Expanded(
                      child: _buildStatsView(),
                    ),
                    const SizedBox(height: 20),
                    _buildCloseButton(),
                  ],
                ),
              ),
            ),
          )
              .animate()
              .slideX(begin: -1.0, duration: 400.ms, curve: Curves.easeOut)
              .fadeIn(duration: 300.ms),
        ),
      ),
    );
  }

  Widget _buildHeader() {
    return Row(
      children: [
        Icon(
          Icons.analytics,
          color: const Color(0xFF00E6FF),
          size: 40,
        ),
        const SizedBox(width: 15),
        Text(
          'DETAILED STATISTICS',
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
      ],
    );
  }

  Widget _buildCategoryTabs() {
    return SingleChildScrollView(
      scrollDirection: Axis.horizontal,
      child: Row(
        children: _categories.map((category) {
          return Padding(
            padding: const EdgeInsets.only(right: 10.0),
            child: _buildCategoryTab(category),
          );
        }).toList(),
      ),
    );
  }

  Widget _buildCategoryTab(String category) {
    final isActive = _selectedCategory == category;
    
    return GestureDetector(
      onTap: () => setState(() => _selectedCategory = category),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 12),
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
          category.toUpperCase(),
          style: TextStyle(
            fontSize: 16,
            fontWeight: FontWeight.w700,
            color: isActive ? const Color(0xFF00E6FF) : Colors.white70,
            letterSpacing: 1.5,
          ),
        ),
      ),
    );
  }

  Widget _buildStatsView() {
    final categoryStats = widget.stats[_selectedCategory] ?? {};
    
    if (categoryStats.isEmpty) {
      return Center(
        child: Text(
          'No stats available.',
          style: TextStyle(
            fontSize: 18,
            color: Colors.white30,
            fontStyle: FontStyle.italic,
          ),
        ),
      );
    }

    return ListView(
      children: categoryStats.entries.map((entry) {
        return Padding(
          padding: const EdgeInsets.only(bottom: 20.0),
          child: _buildStatBar(entry.key, entry.value),
        );
      }).toList(),
    );
  }

  Widget _buildStatBar(String statName, double value) {
    // Normalize value to 0-100 range (assuming stats are 0-100)
    final percentage = (value / 100).clamp(0.0, 1.0);
    
    // Color based on value
    Color barColor;
    if (value >= 70) {
      barColor = Colors.green;
    } else if (value >= 40) {
      barColor = const Color(0xFF00E6FF);
    } else {
      barColor = Colors.red;
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Text(
              statName.toUpperCase(),
              style: const TextStyle(
                fontSize: 16,
                fontWeight: FontWeight.w700,
                color: Colors.white,
                letterSpacing: 1.2,
              ),
            ),
            Text(
              value.toStringAsFixed(1),
              style: TextStyle(
                fontSize: 18,
                fontWeight: FontWeight.w900,
                color: barColor,
              ),
            ),
          ],
        ),
        const SizedBox(height: 8),
        Stack(
          children: [
            // Background
            Container(
              height: 12,
              decoration: BoxDecoration(
                color: Colors.white.withOpacity(0.1),
                border: Border.all(color: Colors.white30, width: 1),
              ),
            ),
            // Fill
            FractionallySizedBox(
              widthFactor: percentage,
              child: Container(
                height: 12,
                decoration: BoxDecoration(
                  color: barColor,
                  boxShadow: [
                    BoxShadow(
                      color: barColor.withOpacity(0.5),
                      blurRadius: 8,
                    ),
                  ],
                ),
              ),
            ),
          ],
        ),
      ],
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

  void _handleKeyEvent(KeyEvent event) {
    if (event is KeyDownEvent) {
      if (event.logicalKey == LogicalKeyboardKey.arrowLeft ||
          event.logicalKey == LogicalKeyboardKey.keyA) {
        final currentIndex = _categories.indexOf(_selectedCategory);
        final newIndex = (currentIndex - 1) % _categories.length;
        setState(() {
          _selectedCategory = _categories[newIndex < 0 ? _categories.length - 1 : newIndex];
        });
      } else if (event.logicalKey == LogicalKeyboardKey.arrowRight ||
          event.logicalKey == LogicalKeyboardKey.keyD) {
        final currentIndex = _categories.indexOf(_selectedCategory);
        final newIndex = (currentIndex + 1) % _categories.length;
        setState(() {
          _selectedCategory = _categories[newIndex];
        });
      } else if (event.logicalKey == LogicalKeyboardKey.escape) {
        widget.onClose();
      }
    }
  }
}
