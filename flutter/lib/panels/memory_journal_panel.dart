import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import '../dev_tools/inspectable_mixin.dart';
import '../ui/syn_ui.dart';

/// Memory Journal Panel - displays character's memories and significant events
/// with Persona 5 × Destiny 2 inspired styling.
///
/// Features:
/// - Timeline-style memory list
/// - Emotional tone color coding
/// - Staggered entrance animations
/// - Character tags with glow
/// - Live editing via InspectorOverrides
class MemoryJournalPanel extends StatefulWidget {
  final VoidCallback onClose;
  final List<MemoryEntry> memories;

  const MemoryJournalPanel({
    super.key,
    required this.onClose,
    required this.memories,
  });

  @override
  State<MemoryJournalPanel> createState() => _MemoryJournalPanelState();
}

class _MemoryJournalPanelState extends State<MemoryJournalPanel>
    with SingleTickerProviderStateMixin {
  int _selectedIndex = 0;
  String _filter = 'all';
  late AnimationController _entranceController;
  late Animation<double> _backdropAnimation;
  late Animation<Offset> _panelSlideAnimation;
  final _o = InspectorOverrides.instance;

  List<MemoryEntry> get _filteredMemories {
    if (_filter == 'all') return widget.memories;
    return widget.memories.where((m) => m.emotionalTone == _filter).toList();
  }

  @override
  void initState() {
    super.initState();
    
    _o.register('MemoryJournalPanel', {
      'padding': 40.0,
      'maxWidth': 950.0,
      'maxHeight': 850.0,
      'backdropOpacity': 0.9,
      'cardSpacing': 16.0,
      'titleFontSize': 28.0,
      'memoryFontSize': 14.0,
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
      begin: const Offset(1.2, 0),
      end: Offset.zero,
    ).animate(CurvedAnimation(
      parent: _entranceController,
      curve: SynTheme.snapIn,
    ));

    _entranceController.forward();
  }

  @override
  void dispose() {
    _o.unregister('MemoryJournalPanel');
    _entranceController.dispose();
    super.dispose();
  }

  Future<void> _animateClose() async {
    await _entranceController.reverse();
    widget.onClose();
  }

  @override
  Widget build(BuildContext context) {
    final padding = _o.get('MemoryJournalPanel.padding', 40.0);
    final maxWidth = _o.get('MemoryJournalPanel.maxWidth', 950.0);
    final maxHeight = _o.get('MemoryJournalPanel.maxHeight', 850.0);
    final backdropOpacity = _o.get('MemoryJournalPanel.backdropOpacity', 0.9);
    
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
                          _buildFilterBar(),
                          const SizedBox(height: 24),
                          Expanded(child: _buildMemoryList()),
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
          Icon(Icons.book, color: SynTheme.accent, size: 44),
          const SizedBox(width: 16),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('MEMORY JOURNAL', style: SynTheme.display()),
                const SizedBox(height: 4),
                Text(
                  '${widget.memories.length} memories recorded',
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
    final filters = ['all', 'positive', 'negative', 'neutral'];
    final labels = ['ALL', 'POSITIVE', 'NEGATIVE', 'NEUTRAL'];
    final colors = [
      SynTheme.accent,
      Colors.green,
      Colors.red,
      Colors.grey,
    ];

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
                activeColor: colors[entry.key],
                onTap: () => setState(() => _filter = entry.value),
              ),
            );
          }).toList(),
        ),
      ),
    );
  }

  Widget _buildMemoryList() {
    final memories = _filteredMemories;

    if (memories.isEmpty) {
      return Center(
        child: Text(
          'No memories found.',
          style: SynTheme.body(color: SynTheme.textMuted),
        ),
      );
    }

    return ListView.builder(
      itemCount: memories.length,
      itemBuilder: (context, index) {
        return Padding(
          padding: const EdgeInsets.only(bottom: 16.0),
          child: SynStaggeredEntrance(
            index: index + 2,
            staggerDelay: const Duration(milliseconds: 40),
            slideFrom: const Offset(0.2, 0),
            child: _MemoryCard(
              memory: memories[index],
              isSelected: _selectedIndex == index,
              onHover: () => setState(() => _selectedIndex = index),
            ),
          ),
        );
      },
    );
  }

  Widget _buildCloseButton() {
    return SynStaggeredEntrance(
      index: 20,
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
      final memories = _filteredMemories;
      if (memories.isEmpty) return;

      if (event.logicalKey == LogicalKeyboardKey.arrowUp ||
          event.logicalKey == LogicalKeyboardKey.keyW) {
        setState(() =>
            _selectedIndex = (_selectedIndex - 1 + memories.length) % memories.length);
        HapticFeedback.selectionClick();
      } else if (event.logicalKey == LogicalKeyboardKey.arrowDown ||
          event.logicalKey == LogicalKeyboardKey.keyS) {
        setState(() => _selectedIndex = (_selectedIndex + 1) % memories.length);
        HapticFeedback.selectionClick();
      } else if (event.logicalKey == LogicalKeyboardKey.escape) {
        _animateClose();
      }
    }
  }
}

/// Filter chip with color theming
class _FilterChip extends StatefulWidget {
  final String label;
  final bool isActive;
  final Color activeColor;
  final VoidCallback onTap;

  const _FilterChip({
    required this.label,
    required this.isActive,
    required this.activeColor,
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
    final color = widget.activeColor;

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
          padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 10),
          decoration: BoxDecoration(
            color: widget.isActive
                ? color.withOpacity(0.25)
                : _isHovered
                    ? color.withOpacity(0.1)
                    : SynTheme.bgCard,
            border: Border.all(
              color: isHighlighted ? color : color.withOpacity(0.3),
            ),
            boxShadow: [
              if (isHighlighted)
                BoxShadow(
                  color: color.withOpacity(0.2),
                  blurRadius: 10,
                ),
            ],
          ),
          child: Text(
            widget.label,
            style: SynTheme.caption(
              color: isHighlighted ? color : SynTheme.textSecondary,
            ),
          ),
        ),
      ),
    );
  }
}

/// Memory card with emotional tone theming
class _MemoryCard extends StatefulWidget {
  final MemoryEntry memory;
  final bool isSelected;
  final VoidCallback onHover;

  const _MemoryCard({
    required this.memory,
    required this.isSelected,
    required this.onHover,
  });

  @override
  State<_MemoryCard> createState() => _MemoryCardState();
}

class _MemoryCardState extends State<_MemoryCard> {
  bool _isHovered = false;

  Color get _toneColor {
    switch (widget.memory.emotionalTone) {
      case 'positive':
        return Colors.green;
      case 'negative':
        return Colors.red;
      default:
        return Colors.grey;
    }
  }

  IconData get _toneIcon {
    switch (widget.memory.emotionalTone) {
      case 'positive':
        return Icons.sentiment_satisfied;
      case 'negative':
        return Icons.sentiment_dissatisfied;
      default:
        return Icons.sentiment_neutral;
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
      child: AnimatedContainer(
        duration: SynTheme.fast,
        curve: SynTheme.snapIn,
        transform: Matrix4.identity()
          ..translate(_isHovered ? -3.0 : 0.0, _isHovered ? -3.0 : 0.0),
        padding: const EdgeInsets.all(20.0),
        decoration: BoxDecoration(
          color: isHighlighted
              ? SynTheme.accent.withOpacity(0.1)
              : SynTheme.bgCard,
          border: Border.all(
            color: widget.isSelected
                ? SynTheme.accent
                : _isHovered
                    ? SynTheme.accent.withOpacity(0.5)
                    : SynTheme.accent.withOpacity(0.2),
            width: widget.isSelected ? 2 : 1,
          ),
          boxShadow: [
            if (isHighlighted)
              BoxShadow(
                color: SynTheme.accent.withOpacity(0.2),
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
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Header row
            Row(
              children: [
                // Emotional tone indicator
                Container(
                  padding: const EdgeInsets.all(6),
                  decoration: BoxDecoration(
                    color: _toneColor.withOpacity(0.2),
                    border: Border.all(color: _toneColor.withOpacity(0.5)),
                  ),
                  child: Icon(_toneIcon, color: _toneColor, size: 18),
                ),
                const SizedBox(width: 12),
                Expanded(
                  child: Text(
                    widget.memory.title,
                    style: SynTheme.title(
                      color: isHighlighted
                          ? SynTheme.textPrimary
                          : SynTheme.textSecondary,
                    ),
                  ),
                ),
                // Day indicator
                Container(
                  padding:
                      const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
                  decoration: BoxDecoration(
                    border: Border.all(color: SynTheme.accent.withOpacity(0.3)),
                    color: SynTheme.bgSurface,
                  ),
                  child: Text(
                    'DAY ${widget.memory.day}',
                    style: SynTheme.caption(color: SynTheme.textMuted),
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            // Description
            Text(
              widget.memory.description,
              style: SynTheme.body(color: SynTheme.textSecondary),
            ),
            // Character tags
            if (widget.memory.involvedCharacters.isNotEmpty) ...[
              const SizedBox(height: 14),
              Wrap(
                spacing: 8,
                runSpacing: 6,
                children: widget.memory.involvedCharacters.map((name) {
                  return Container(
                    padding:
                        const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
                    decoration: BoxDecoration(
                      color: SynTheme.accent.withOpacity(0.15),
                      border: Border.all(color: SynTheme.accent),
                    ),
                    child: Text(
                      name,
                      style: SynTheme.caption(color: SynTheme.accent),
                    ),
                  );
                }).toList(),
              ),
            ],
          ],
        ),
      ),
    );
  }
}

/// Data model for memory entries
class MemoryEntry {
  final String title;
  final String description;
  final int day;
  final String emotionalTone; // positive, negative, neutral
  final List<String> involvedCharacters;

  const MemoryEntry({
    required this.title,
    required this.description,
    required this.day,
    required this.emotionalTone,
    this.involvedCharacters = const [],
  });
}
