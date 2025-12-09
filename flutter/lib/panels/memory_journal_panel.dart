import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_animate/flutter_animate.dart';
import '../ui/widgets/persona_container.dart';

/// Memory Journal Panel - displays character's memories and significant events
/// 
/// Props:
/// - onClose: Callback to close the panel
/// - memories: List of memory entries
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

class _MemoryJournalPanelState extends State<MemoryJournalPanel> {
  int _selectedIndex = 0;
  String _filter = 'all'; // all, positive, negative, neutral

  List<MemoryEntry> get _filteredMemories {
    if (_filter == 'all') return widget.memories;
    return widget.memories
        .where((m) => m.emotionalTone == _filter)
        .toList();
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
                      child: _buildMemoryList(),
                    ),
                    const SizedBox(height: 20),
                    _buildCloseButton(),
                  ],
                ),
              ),
            ),
          )
              .animate()
              .slideX(begin: 1.0, duration: 400.ms, curve: Curves.easeOut)
              .fadeIn(duration: 300.ms),
        ),
      ),
    );
  }

  Widget _buildHeader() {
    return Row(
      children: [
        Icon(
          Icons.book,
          color: const Color(0xFF00E6FF),
          size: 40,
        ),
        const SizedBox(width: 15),
        Expanded(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                'MEMORY JOURNAL',
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
                '${widget.memories.length} memories recorded',
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
        _buildFilterChip('POSITIVE', 'positive'),
        const SizedBox(width: 10),
        _buildFilterChip('NEGATIVE', 'negative'),
        const SizedBox(width: 10),
        _buildFilterChip('NEUTRAL', 'neutral'),
      ],
    );
  }

  Widget _buildFilterChip(String label, String filterValue) {
    final isActive = _filter == filterValue;
    
    return GestureDetector(
      onTap: () => setState(() => _filter = filterValue),
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
            fontSize: 14,
            fontWeight: FontWeight.w700,
            color: isActive ? const Color(0xFF00E6FF) : Colors.white70,
          ),
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
          style: TextStyle(
            fontSize: 18,
            color: Colors.white30,
            fontStyle: FontStyle.italic,
          ),
        ),
      );
    }

    return ListView.builder(
      itemCount: memories.length,
      itemBuilder: (context, index) {
        return Padding(
          padding: const EdgeInsets.only(bottom: 15.0),
          child: _buildMemoryCard(memories[index], index),
        );
      },
    );
  }

  Widget _buildMemoryCard(MemoryEntry memory, int index) {
    final isSelected = _selectedIndex == index;
    
    Color toneColor;
    IconData toneIcon;
    
    switch (memory.emotionalTone) {
      case 'positive':
        toneColor = Colors.green;
        toneIcon = Icons.sentiment_satisfied;
        break;
      case 'negative':
        toneColor = Colors.red;
        toneIcon = Icons.sentiment_dissatisfied;
        break;
      default:
        toneColor = Colors.grey;
        toneIcon = Icons.sentiment_neutral;
    }

    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _selectedIndex = index),
      child: PersonaContainer(
        skew: -0.1,
        color: isSelected
            ? const Color(0xFF00E6FF).withOpacity(0.15)
            : Colors.black.withOpacity(0.5),
        child: Padding(
          padding: const EdgeInsets.all(20.0),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Icon(toneIcon, color: toneColor, size: 20),
                  const SizedBox(width: 10),
                  Expanded(
                    child: Text(
                      memory.title,
                      style: TextStyle(
                        fontSize: 18,
                        fontWeight: FontWeight.w700,
                        color: Colors.white,
                      ),
                    ),
                  ),
                  Text(
                    'Day ${memory.day}',
                    style: const TextStyle(
                      fontSize: 12,
                      color: Colors.white38,
                    ),
                  ),
                ],
              ),
              const SizedBox(height: 10),
              Text(
                memory.description,
                style: const TextStyle(
                  fontSize: 14,
                  color: Colors.white70,
                  height: 1.4,
                ),
              ),
              if (memory.involvedCharacters.isNotEmpty) ...[
                const SizedBox(height: 10),
                Wrap(
                  spacing: 8,
                  children: memory.involvedCharacters.map((name) {
                    return Container(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 8,
                        vertical: 4,
                      ),
                      decoration: BoxDecoration(
                        color: const Color(0xFF00E6FF).withOpacity(0.2),
                        border: Border.all(
                          color: const Color(0xFF00E6FF),
                          width: 1,
                        ),
                      ),
                      child: Text(
                        name,
                        style: const TextStyle(
                          fontSize: 11,
                          color: Color(0xFF00E6FF),
                          fontWeight: FontWeight.w600,
                        ),
                      ),
                    );
                  }).toList(),
                ),
              ],
            ],
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
      final memories = _filteredMemories;
      
      if (event.logicalKey == LogicalKeyboardKey.arrowUp ||
          event.logicalKey == LogicalKeyboardKey.keyW) {
        setState(() {
          _selectedIndex = (_selectedIndex - 1) % memories.length;
          if (_selectedIndex < 0) _selectedIndex = memories.length - 1;
        });
      } else if (event.logicalKey == LogicalKeyboardKey.arrowDown ||
          event.logicalKey == LogicalKeyboardKey.keyS) {
        setState(() {
          _selectedIndex = (_selectedIndex + 1) % memories.length;
        });
      } else if (event.logicalKey == LogicalKeyboardKey.escape) {
        widget.onClose();
      }
    }
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
