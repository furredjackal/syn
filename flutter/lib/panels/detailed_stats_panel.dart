import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../dev_tools/inspectable_mixin.dart';
import '../ui/syn_ui.dart';

/// Detailed Stats Panel - displays comprehensive character statistics
/// with Persona 5 × Destiny 2 inspired animations and styling.
///
/// Features:
/// - Staggered entrance animations
/// - Animated stat bars with glow effects
/// - Category tabs with hover states
/// - Keyboard navigation support
/// - Live editing via InspectorOverrides
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

class _DetailedStatsPanelState extends State<DetailedStatsPanel>
    with SingleTickerProviderStateMixin {
  String _selectedCategory = 'core';
  late AnimationController _entranceController;
  late Animation<double> _backdropAnimation;
  late Animation<Offset> _panelSlideAnimation;
  late Animation<double> _panelScaleAnimation;

  // Inspector overrides accessor
  final _o = InspectorOverrides.instance;

  List<String> get _categories => widget.stats.keys.toList();

  @override
  void initState() {
    super.initState();
    
    // Register editable properties with the inspector
    _o.register('DetailedStatsPanel', {
      'padding': 40.0,
      'headerFontSize': 32.0,
      'tabFontSize': 16.0,
      'statBarHeight': 24.0,
      'statFontSize': 18.0,
      'statSpacing': 20.0,
      'maxWidth': 950.0,
      'maxHeight': 850.0,
      'backdropOpacity': 0.9,
      'borderRadius': 0.0,
    }, onUpdate: () => setState(() {}));
    
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
      begin: const Offset(-1.5, 0),
      end: Offset.zero,
    ).animate(CurvedAnimation(
      parent: _entranceController,
      curve: SynTheme.snapIn,
    ));

    _panelScaleAnimation = TweenSequence<double>([
      TweenSequenceItem(tween: Tween(begin: 0.8, end: 1.03), weight: 80),
      TweenSequenceItem(tween: Tween(begin: 1.03, end: 1.0), weight: 20),
    ]).animate(CurvedAnimation(
      parent: _entranceController,
      curve: Curves.easeOut,
    ));

    _entranceController.forward();
  }

  @override
  void dispose() {
    _o.unregister('DetailedStatsPanel');
    _entranceController.dispose();
    super.dispose();
  }

  Future<void> _animateClose() async {
    await _entranceController.reverse();
    widget.onClose();
  }

  @override
  Widget build(BuildContext context) {
    // Read editable values from overrides
    final padding = _o.get('DetailedStatsPanel.padding', 40.0);
    final maxWidth = _o.get('DetailedStatsPanel.maxWidth', 950.0);
    final maxHeight = _o.get('DetailedStatsPanel.maxHeight', 850.0);
    final backdropOpacity = _o.get('DetailedStatsPanel.backdropOpacity', 0.9);
    
    return KeyboardListener(
      focusNode: FocusNode()..requestFocus(),
      onKeyEvent: _handleKeyEvent,
      child: AnimatedBuilder(
        animation: _entranceController,
        builder: (context, _) {
          return Container(
            color: Colors.black.withOpacity(backdropOpacity * _backdropAnimation.value),
            child: Center(
              child: SlideTransition(
                position: _panelSlideAnimation,
                child: Transform.scale(
                  scale: _panelScaleAnimation.value,
                  child: ConstrainedBox(
                    constraints: BoxConstraints(maxWidth: maxWidth, maxHeight: maxHeight),
                    child: SynContainer(
                      enableHover: false,
                      child: Padding(
                        padding: EdgeInsets.all(padding),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.stretch,
                          children: [
                            _buildHeader(),
                            const SizedBox(height: 24),
                            _buildCategoryTabs(),
                            const SizedBox(height: 28),
                            Expanded(child: _buildStatsView()),
                            const SizedBox(height: 24),
                            _buildCloseButton(),
                          ],
                        ),
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
          Icon(Icons.analytics, color: SynTheme.accent, size: 44),
          const SizedBox(width: 16),
          Expanded(
            child: Text(
              'DETAILED STATISTICS',
              style: SynTheme.display(),
              overflow: TextOverflow.ellipsis,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildCategoryTabs() {
    return SynStaggeredEntrance(
      index: 1,
      child: SingleChildScrollView(
        scrollDirection: Axis.horizontal,
        child: Row(
          children: _categories.asMap().entries.map((entry) {
            return Padding(
              padding: const EdgeInsets.only(right: 12.0),
              child: _CategoryTab(
                label: entry.value,
                isActive: _selectedCategory == entry.value,
                onTap: () => setState(() => _selectedCategory = entry.value),
              ),
            );
          }).toList(),
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
          style: SynTheme.body(color: SynTheme.textMuted),
        ),
      );
    }

    final entries = categoryStats.entries.toList();

    return ListView.builder(
      itemCount: entries.length,
      itemBuilder: (context, index) {
        final entry = entries[index];
        return Padding(
          padding: const EdgeInsets.only(bottom: 20.0),
          child: SynStaggeredEntrance(
            index: index + 2, // Offset for header + tabs
            child: SynStatBar(
              label: entry.key,
              value: entry.value, // 0-100 range
              showValue: true,
              height: 16,
            ),
          ),
        );
      },
    );
  }

  Widget _buildCloseButton() {
    return SynStaggeredEntrance(
      index: 10,
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

  void _handleKeyEvent(KeyEvent event) {
    if (event is KeyDownEvent) {
      if (event.logicalKey == LogicalKeyboardKey.arrowLeft ||
          event.logicalKey == LogicalKeyboardKey.keyA) {
        final currentIndex = _categories.indexOf(_selectedCategory);
        final newIndex = (currentIndex - 1 + _categories.length) % _categories.length;
        setState(() => _selectedCategory = _categories[newIndex]);
        HapticFeedback.selectionClick();
      } else if (event.logicalKey == LogicalKeyboardKey.arrowRight ||
          event.logicalKey == LogicalKeyboardKey.keyD) {
        final currentIndex = _categories.indexOf(_selectedCategory);
        final newIndex = (currentIndex + 1) % _categories.length;
        setState(() => _selectedCategory = _categories[newIndex]);
        HapticFeedback.selectionClick();
      } else if (event.logicalKey == LogicalKeyboardKey.escape) {
        _animateClose();
      }
    }
  }
}

/// Category tab button with hover effects.
class _CategoryTab extends StatefulWidget {
  final String label;
  final bool isActive;
  final VoidCallback onTap;

  const _CategoryTab({
    required this.label,
    required this.isActive,
    required this.onTap,
  });

  @override
  State<_CategoryTab> createState() => _CategoryTabState();
}

class _CategoryTabState extends State<_CategoryTab> {
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
          curve: SynTheme.snapIn,
          padding: const EdgeInsets.symmetric(horizontal: 22, vertical: 14),
          transform: Matrix4.identity()
            ..translate(_isHovered ? -2.0 : 0.0, _isHovered ? -2.0 : 0.0),
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
              width: widget.isActive ? 2 : 1,
            ),
            boxShadow: [
              if (isHighlighted)
                BoxShadow(
                  color: SynTheme.accent.withOpacity(0.3),
                  blurRadius: 12,
                  spreadRadius: -2,
                ),
              BoxShadow(
                color: Colors.black.withOpacity(0.6),
                offset: Offset(
                  _isHovered ? 4 : 2,
                  _isHovered ? 4 : 2,
                ),
                blurRadius: 0,
              ),
            ],
          ),
          child: Text(
            widget.label.toUpperCase(),
            style: SynTheme.label(
              color: isHighlighted ? SynTheme.accent : SynTheme.textSecondary,
            ),
          ),
        ),
      ),
    );
  }
}
