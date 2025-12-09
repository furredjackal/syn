import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_animate/flutter_animate.dart';
import '../ui/widgets/persona_container.dart';

/// End of Life Screen - displays life summary and restart option
/// 
/// Props:
/// - onRestart: Callback when player wants to start a new life
/// - onReturnToTitle: Callback to return to main menu
/// - lifeSummary: Map containing life statistics
class EndOfLifeScreen extends StatefulWidget {
  final VoidCallback onRestart;
  final VoidCallback onReturnToTitle;
  final Map<String, dynamic>? lifeSummary;

  const EndOfLifeScreen({
    super.key,
    required this.onRestart,
    required this.onReturnToTitle,
    this.lifeSummary,
  });

  @override
  State<EndOfLifeScreen> createState() => _EndOfLifeScreenState();
}

class _EndOfLifeScreenState extends State<EndOfLifeScreen> {
  int _selectedIndex = 0;
  final int _totalOptions = 2; // Restart, Return to Title

  @override
  Widget build(BuildContext context) {
    return KeyboardListener(
      focusNode: FocusNode()..requestFocus(),
      onKeyEvent: _handleKeyEvent,
      child: Container(
        decoration: BoxDecoration(
          gradient: LinearGradient(
            begin: Alignment.topLeft,
            end: Alignment.bottomRight,
            colors: [
              Colors.black,
              Colors.grey.shade900,
              Colors.black,
            ],
          ),
        ),
        child: SafeArea(
          child: SingleChildScrollView(
            child: Padding(
              padding: const EdgeInsets.all(40.0),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.stretch,
                children: [
                  _buildHeader(),
                  const SizedBox(height: 60),
                  _buildLifeSummary(),
                  const SizedBox(height: 60),
                  _buildActions(),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildHeader() {
    return Column(
      children: [
        Text(
          'END OF LIFE',
          style: TextStyle(
            fontSize: 72,
            fontWeight: FontWeight.w900,
            color: Colors.white,
            letterSpacing: 8,
            shadows: [
              Shadow(
                color: const Color(0xFF00E6FF).withOpacity(0.8),
                blurRadius: 20,
                offset: const Offset(0, 0),
              ),
              const Shadow(
                color: Colors.black,
                blurRadius: 4,
                offset: Offset(3, 3),
              ),
            ],
          ),
          textAlign: TextAlign.center,
        )
            .animate()
            .fadeIn(duration: 600.ms, curve: Curves.easeOut)
            .scale(begin: const Offset(0.8, 0.8), curve: Curves.easeOutBack),
        const SizedBox(height: 20),
        Container(
          height: 3,
          width: 400,
          decoration: BoxDecoration(
            gradient: LinearGradient(
              colors: [
                Colors.transparent,
                const Color(0xFF00E6FF),
                Colors.transparent,
              ],
            ),
            boxShadow: [
              BoxShadow(
                color: const Color(0xFF00E6FF).withOpacity(0.5),
                blurRadius: 10,
              ),
            ],
          ),
        )
            .animate(delay: 300.ms)
            .scale(
              begin: const Offset(0, 1),
              duration: 400.ms,
              curve: Curves.easeOut,
            ),
      ],
    );
  }

  Widget _buildLifeSummary() {
    final summary = widget.lifeSummary ?? _getDefaultSummary();

    return PersonaContainer(
      skew: -0.15,
      child: Padding(
        padding: const EdgeInsets.all(30.0),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'YOUR LIFE',
              style: TextStyle(
                fontSize: 36,
                fontWeight: FontWeight.w900,
                color: const Color(0xFF00E6FF),
                letterSpacing: 4,
              ),
            ),
            const SizedBox(height: 30),
            _buildStatRow('Name', summary['name'] ?? 'Unknown'),
            _buildStatRow('Age at Death', '${summary['age'] ?? 0} years'),
            _buildStatRow('Archetype', summary['archetype'] ?? 'None'),
            const SizedBox(height: 20),
            const Divider(color: Color(0xFF00E6FF), thickness: 2),
            const SizedBox(height: 20),
            Text(
              'STATISTICS',
              style: TextStyle(
                fontSize: 28,
                fontWeight: FontWeight.w900,
                color: const Color(0xFF00E6FF),
                letterSpacing: 3,
              ),
            ),
            const SizedBox(height: 20),
            _buildStatRow('Relationships Formed', '${summary['relationships'] ?? 0}'),
            _buildStatRow('Major Events', '${summary['events'] ?? 0}'),
            _buildStatRow('Career Changes', '${summary['careers'] ?? 0}'),
            _buildStatRow('Locations Visited', '${summary['locations'] ?? 0}'),
            const SizedBox(height: 20),
            const Divider(color: Color(0xFF00E6FF), thickness: 2),
            const SizedBox(height: 20),
            Text(
              'LEGACY',
              style: TextStyle(
                fontSize: 28,
                fontWeight: FontWeight.w900,
                color: const Color(0xFF00E6FF),
                letterSpacing: 3,
              ),
            ),
            const SizedBox(height: 15),
            Text(
              summary['legacy'] ?? 'Your story was unique.',
              style: const TextStyle(
                fontSize: 18,
                color: Colors.white70,
                fontStyle: FontStyle.italic,
                height: 1.5,
              ),
            ),
          ],
        ),
      ),
    )
        .animate()
        .slideY(begin: 0.3, duration: 600.ms, curve: Curves.easeOut)
        .fadeIn(duration: 600.ms);
  }

  Widget _buildStatRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8.0),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(
            label,
            style: const TextStyle(
              fontSize: 18,
              color: Colors.white70,
              fontWeight: FontWeight.w600,
            ),
          ),
          Text(
            value,
            style: const TextStyle(
              fontSize: 20,
              color: Colors.white,
              fontWeight: FontWeight.w700,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildActions() {
    return Column(
      children: [
        _buildActionButton(
          index: 0,
          label: 'BEGIN NEW LIFE',
          icon: Icons.refresh,
          onTap: widget.onRestart,
        ),
        const SizedBox(height: 20),
        _buildActionButton(
          index: 1,
          label: 'RETURN TO TITLE',
          icon: Icons.home,
          onTap: widget.onReturnToTitle,
        ),
      ],
    );
  }

  Widget _buildActionButton({
    required int index,
    required String label,
    required IconData icon,
    required VoidCallback onTap,
  }) {
    final isSelected = _selectedIndex == index;

    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _selectedIndex = index),
      child: GestureDetector(
        onTap: onTap,
        child: PersonaContainer(
          skew: -0.18,
          color: isSelected
              ? const Color(0xFF00E6FF).withOpacity(0.2)
              : Colors.black.withOpacity(0.6),
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: 20, horizontal: 30),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Icon(
                  icon,
                  color: isSelected ? const Color(0xFF00E6FF) : Colors.white70,
                  size: 28,
                ),
                const SizedBox(width: 15),
                Text(
                  label,
                  style: TextStyle(
                    fontSize: 24,
                    fontWeight: FontWeight.w900,
                    color: isSelected ? const Color(0xFF00E6FF) : Colors.white,
                    letterSpacing: 2,
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    )
        .animate(delay: (index * 100).ms)
        .slideX(begin: 0.3, duration: 400.ms, curve: Curves.easeOut)
        .fadeIn(duration: 400.ms);
  }

  void _handleKeyEvent(KeyEvent event) {
    if (event is KeyDownEvent) {
      if (event.logicalKey == LogicalKeyboardKey.arrowUp ||
          event.logicalKey == LogicalKeyboardKey.keyW) {
        setState(() {
          _selectedIndex = (_selectedIndex - 1) % _totalOptions;
          if (_selectedIndex < 0) _selectedIndex = _totalOptions - 1;
        });
      } else if (event.logicalKey == LogicalKeyboardKey.arrowDown ||
          event.logicalKey == LogicalKeyboardKey.keyS) {
        setState(() {
          _selectedIndex = (_selectedIndex + 1) % _totalOptions;
        });
      } else if (event.logicalKey == LogicalKeyboardKey.enter ||
          event.logicalKey == LogicalKeyboardKey.space) {
        _triggerAction(_selectedIndex);
      }
    }
  }

  void _triggerAction(int index) {
    switch (index) {
      case 0:
        widget.onRestart();
        break;
      case 1:
        widget.onReturnToTitle();
        break;
    }
  }

  Map<String, dynamic> _getDefaultSummary() {
    return {
      'name': 'Unknown',
      'age': 0,
      'archetype': 'None',
      'relationships': 0,
      'events': 0,
      'careers': 0,
      'locations': 0,
      'legacy': 'Your story was never told.',
    };
  }
}
