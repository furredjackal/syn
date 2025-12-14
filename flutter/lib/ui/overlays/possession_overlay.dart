import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_animate/flutter_animate.dart';
import '../../dev_tools/inspectable_mixin.dart';
import '../widgets/persona_container.dart';

/// Possession Overlay - choose a new host to possess (PostLife mechanic)
///
/// Props:
/// - onClose: Callback to close the overlay
/// - onHostSelect: Callback when a host is selected
/// - hosts: List of available hosts
class PossessionOverlay extends StatefulWidget {
  final VoidCallback onClose;
  final Function(String hostId) onHostSelect;
  final List<PossessionHost> hosts;

  const PossessionOverlay({
    super.key,
    required this.onClose,
    required this.onHostSelect,
    this.hosts = const [],
  });

  @override
  State<PossessionOverlay> createState() => _PossessionOverlayState();
}

class _PossessionOverlayState extends State<PossessionOverlay> {
  int _selectedIndex = 0;
  final _o = InspectorOverrides.instance;

  @override
  void initState() {
    super.initState();
    _o.register('PossessionOverlay', {
      'backdropOpacity': 0.95,
      'widthFactor': 0.75,
      'heightFactor': 0.65,
      'titleFontSize': 48.0,
      'subtitleFontSize': 16.0,
      'titleLeft': 60.0,
      'titleTop': 40.0,
    }, onUpdate: () => setState(() {}));
  }

  @override
  void dispose() {
    _o.unregister('PossessionOverlay');
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final screenWidth = MediaQuery.of(context).size.width;
    final screenHeight = MediaQuery.of(context).size.height;
    
    final backdropOpacity = _o.get('PossessionOverlay.backdropOpacity', 0.95);
    final widthFactor = _o.get('PossessionOverlay.widthFactor', 0.75);
    final heightFactor = _o.get('PossessionOverlay.heightFactor', 0.65);
    final titleFontSize = _o.get('PossessionOverlay.titleFontSize', 48.0);
    final subtitleFontSize = _o.get('PossessionOverlay.subtitleFontSize', 16.0);
    final titleLeft = _o.get('PossessionOverlay.titleLeft', 60.0);
    final titleTop = _o.get('PossessionOverlay.titleTop', 40.0);

    return KeyboardListener(
      focusNode: FocusNode()..requestFocus(),
      onKeyEvent: _handleKeyEvent,
      child: Container(
        color: Colors.black.withValues(alpha: backdropOpacity),
        child: Stack(
          children: [
            // Title
            Positioned(
              left: titleLeft,
              top: titleTop,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    'CHOOSE YOUR HOST',
                    style: TextStyle(
                      color: Colors.white,
                      fontSize: titleFontSize,
                      fontWeight: FontWeight.w900,
                      letterSpacing: 4,
                      shadows: [
                        Shadow(
                          color: Colors.deepPurpleAccent.withValues(alpha: 0.7),
                          blurRadius: 20,
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(height: 8),
                  Text(
                    'Your essence will inhabit a new vessel',
                    style: TextStyle(
                      color: Colors.white.withValues(alpha: 0.7),
                      fontSize: subtitleFontSize,
                      letterSpacing: 1.2,
                    ),
                  ),
                ],
              )
                  .animate()
                  .fadeIn(duration: 400.ms)
                  .slideY(begin: -0.2, duration: 400.ms, curve: Curves.easeOut),
            ),

            // Host Grid
            Center(
              child: Container(
                constraints: BoxConstraints(
                  maxWidth: screenWidth * widthFactor,
                  maxHeight: screenHeight * heightFactor,
                ),
                child: widget.hosts.isEmpty
                    ? _buildPlaceholderHosts()
                    : _buildHostGrid(),
              )
                  .animate()
                  .fadeIn(delay: 200.ms, duration: 600.ms)
                  .scale(
                      begin: const Offset(0.95, 0.95),
                      duration: 600.ms,
                      curve: Curves.easeOutExpo),
            ),

            // Close Button
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

  Widget _buildHostGrid() {
    return GridView.builder(
      padding: const EdgeInsets.all(20),
      gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
        crossAxisCount: 3,
        crossAxisSpacing: 20,
        mainAxisSpacing: 20,
        childAspectRatio: 0.8,
      ),
      itemCount: widget.hosts.length,
      itemBuilder: (context, index) {
        final host = widget.hosts[index];
        final isSelected = index == _selectedIndex;

        return _buildHostCard(
          host: host,
          isSelected: isSelected,
          onTap: () => _selectHost(index),
        );
      },
    );
  }

  Widget _buildPlaceholderHosts() {
    final placeholderHosts = [
      PossessionHost(
        id: 'corporate_exec',
        name: 'Corporate Executive',
        age: 42,
        occupation: 'CEO',
        traits: ['Ambitious', 'Ruthless'],
        compatibility: 85,
      ),
      PossessionHost(
        id: 'street_artist',
        name: 'Street Artist',
        age: 27,
        occupation: 'Artist',
        traits: ['Creative', 'Rebellious'],
        compatibility: 72,
      ),
      PossessionHost(
        id: 'detective',
        name: 'Detective',
        age: 38,
        occupation: 'Investigator',
        traits: ['Perceptive', 'Jaded'],
        compatibility: 90,
      ),
      PossessionHost(
        id: 'hacker',
        name: 'Hacker',
        age: 24,
        occupation: 'Tech Specialist',
        traits: ['Intelligent', 'Paranoid'],
        compatibility: 78,
      ),
      PossessionHost(
        id: 'bartender',
        name: 'Bartender',
        age: 35,
        occupation: 'Service Worker',
        traits: ['Charismatic', 'Empathetic'],
        compatibility: 68,
      ),
      PossessionHost(
        id: 'corpo_security',
        name: 'Security Officer',
        age: 31,
        occupation: 'Corporate Security',
        traits: ['Disciplined', 'Observant'],
        compatibility: 80,
      ),
    ];

    return GridView.builder(
      padding: const EdgeInsets.all(20),
      gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
        crossAxisCount: 3,
        crossAxisSpacing: 20,
        mainAxisSpacing: 20,
        childAspectRatio: 0.8,
      ),
      itemCount: placeholderHosts.length,
      itemBuilder: (context, index) {
        final host = placeholderHosts[index];
        final isSelected = index == _selectedIndex;

        return _buildHostCard(
          host: host,
          isSelected: isSelected,
          onTap: () => _selectHost(index),
        );
      },
    );
  }

  Widget _buildHostCard({
    required PossessionHost host,
    required bool isSelected,
    required VoidCallback onTap,
  }) {
    final compatColor = _getCompatibilityColor(host.compatibility);

    return MouseRegion(
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        onTap: onTap,
        child: PersonaContainer(
          color: isSelected ? compatColor.withValues(alpha: 0.2) : Colors.black,
          borderColor: isSelected ? compatColor : Colors.white.withValues(alpha: 0.3),
          borderWidth: isSelected ? 3 : 2,
          skew: -0.15,
          child: Container(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // Compatibility Badge
                Align(
                  alignment: Alignment.topRight,
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
                    decoration: BoxDecoration(
                      color: compatColor,
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Text(
                      '${host.compatibility}%',
                      style: const TextStyle(
                        color: Colors.black,
                        fontSize: 14,
                        fontWeight: FontWeight.w800,
                      ),
                    ),
                  ),
                ),
                const SizedBox(height: 12),

                // Name
                Text(
                  host.name,
                  style: const TextStyle(
                    color: Colors.white,
                    fontSize: 20,
                    fontWeight: FontWeight.w800,
                    letterSpacing: 1.0,
                  ),
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                ),
                const SizedBox(height: 8),

                // Age and Occupation
                Text(
                  '${host.age} years old',
                  style: TextStyle(
                    color: Colors.white.withValues(alpha: 0.7),
                    fontSize: 13,
                  ),
                ),
                const SizedBox(height: 4),
                Text(
                  host.occupation,
                  style: TextStyle(
                    color: Colors.cyanAccent.withValues(alpha: 0.9),
                    fontSize: 14,
                    fontWeight: FontWeight.w600,
                    letterSpacing: 0.5,
                  ),
                ),
                const Spacer(),

                // Traits
                Wrap(
                  spacing: 6,
                  runSpacing: 6,
                  children: host.traits.map((trait) {
                    return Container(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 10,
                        vertical: 4,
                      ),
                      decoration: BoxDecoration(
                        color: Colors.white.withValues(alpha: 0.1),
                        borderRadius: BorderRadius.circular(4),
                        border: Border.all(
                          color: Colors.white.withValues(alpha: 0.3),
                        ),
                      ),
                      child: Text(
                        trait,
                        style: TextStyle(
                          color: Colors.white.withValues(alpha: 0.8),
                          fontSize: 11,
                        ),
                      ),
                    );
                  }).toList(),
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

    final hostCount = widget.hosts.isEmpty ? 6 : widget.hosts.length;

    if (event.logicalKey == LogicalKeyboardKey.escape) {
      widget.onClose();
    } else if (event.logicalKey == LogicalKeyboardKey.arrowUp) {
      setState(() {
        _selectedIndex = (_selectedIndex - 3).clamp(0, hostCount - 1);
      });
    } else if (event.logicalKey == LogicalKeyboardKey.arrowDown) {
      setState(() {
        _selectedIndex = (_selectedIndex + 3).clamp(0, hostCount - 1);
      });
    } else if (event.logicalKey == LogicalKeyboardKey.arrowLeft) {
      setState(() {
        _selectedIndex = (_selectedIndex - 1).clamp(0, hostCount - 1);
      });
    } else if (event.logicalKey == LogicalKeyboardKey.arrowRight) {
      setState(() {
        _selectedIndex = (_selectedIndex + 1).clamp(0, hostCount - 1);
      });
    } else if (event.logicalKey == LogicalKeyboardKey.enter ||
        event.logicalKey == LogicalKeyboardKey.space) {
      _selectHost(_selectedIndex);
    }
  }

  void _selectHost(int index) {
    setState(() {
      _selectedIndex = index;
    });

    final hostId = widget.hosts.isEmpty
        ? ['corporate_exec', 'street_artist', 'detective', 'hacker', 'bartender', 'corpo_security'][index]
        : widget.hosts[index].id;

    widget.onHostSelect(hostId);
  }

  Color _getCompatibilityColor(int compatibility) {
    if (compatibility >= 80) return Colors.greenAccent;
    if (compatibility >= 60) return Colors.cyanAccent;
    if (compatibility >= 40) return Colors.orangeAccent;
    return Colors.redAccent;
  }
}

/// Data model for possession hosts
class PossessionHost {
  final String id;
  final String name;
  final int age;
  final String occupation;
  final List<String> traits;
  final int compatibility; // 0-100

  const PossessionHost({
    required this.id,
    required this.name,
    required this.age,
    required this.occupation,
    required this.traits,
    required this.compatibility,
  });
}
