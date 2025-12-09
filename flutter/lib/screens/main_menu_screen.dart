import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_animate/flutter_animate.dart';

import '../ui/widgets/persona_container.dart';

/// Main Menu Screen - Hybrid Architecture Flutter Widget
///
/// Persona 5 styled main menu with:
/// - Large "SYN" title (top-left)
/// - Vertical slash-styled menu buttons
/// - Status sidebar panel (right)
/// - Keyboard navigation support
class MainMenuScreen extends StatefulWidget {
  final VoidCallback onStartGame;
  final VoidCallback onSettings;
  final VoidCallback? onTutorial;
  final VoidCallback? onDataLoad;
  final VoidCallback? onDataSave;
  final VoidCallback? onReturnToTitle;

  const MainMenuScreen({
    super.key,
    required this.onStartGame,
    required this.onSettings,
    this.onTutorial,
    this.onDataLoad,
    this.onDataSave,
    this.onReturnToTitle,
  });

  @override
  State<MainMenuScreen> createState() => _MainMenuScreenState();
}

class _MainMenuScreenState extends State<MainMenuScreen> {
  int _selectedIndex = 0;

  final List<String> _menuLabels = [
    'STORY',
    'TUTORIAL',
    'SETTINGS',
    'DATA LOAD',
    'DATA SAVE',
    'RETURN TO TITLE',
  ];

  @override
  Widget build(BuildContext context) {
    return KeyboardListener(
      focusNode: FocusNode()..requestFocus(),
      onKeyEvent: _handleKeyEvent,
      child: Container(
        color: Colors.black.withValues(alpha: 0.85),
        child: Stack(
          children: [
            // Large "SYN" Title (Top Left)
            Positioned(
              left: 80,
              top: 60,
              child: Text(
                'SYN',
                style: TextStyle(
                  color: Colors.white,
                  fontSize: 72,
                  fontWeight: FontWeight.w900,
                  letterSpacing: 8,
                  shadows: [
                    Shadow(
                      color: Colors.cyanAccent.withValues(alpha: 0.5),
                      blurRadius: 20,
                      offset: const Offset(0, 0),
                    ),
                  ],
                ),
              )
                  .animate()
                  .fadeIn(duration: 600.ms)
                  .slideX(
                      begin: -0.2, duration: 600.ms, curve: Curves.easeOutExpo),
            ),

            // Menu Options Column (Left Side)
            Positioned(
              left: 80,
              top: 180,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: List.generate(_menuLabels.length, (index) {
                  return Padding(
                    padding: const EdgeInsets.only(bottom: 16),
                    child: _buildMenuButton(
                      label: _menuLabels[index],
                      isActive: index == _selectedIndex,
                      onTap: () => _triggerAction(index),
                      index: index,
                    ),
                  );
                }),
              )
                  .animate()
                  .fadeIn(delay: 200.ms, duration: 600.ms)
                  .slideX(
                      begin: -0.3, duration: 600.ms, curve: Curves.easeOutExpo),
            ),

            // Status/Info Sidebar Panel (Right Side)
            Positioned(
              right: 60,
              top: 180,
              child: _buildStatusSidebar()
                  .animate()
                  .fadeIn(delay: 400.ms, duration: 600.ms)
                  .slideX(
                      begin: 0.3, duration: 600.ms, curve: Curves.easeOutExpo),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildMenuButton({
    required String label,
    required bool isActive,
    required VoidCallback onTap,
    required int index,
  }) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) {
        setState(() {
          _selectedIndex = index;
        });
      },
      child: GestureDetector(
        onTap: onTap,
        child: PersonaContainer(
          color: isActive ? Colors.white : Colors.black,
          skew: -0.2,
          child: Container(
            width: 400,
            height: 70,
            alignment: Alignment.centerLeft,
            padding: const EdgeInsets.only(left: 80),
            child: Row(
              children: [
                // Active indicator arrow
                if (isActive)
                  Icon(
                    Icons.arrow_right,
                    color: Colors.black,
                    size: 32,
                  )
                      .animate(onPlay: (controller) => controller.repeat())
                      .slideX(
                        begin: -0.2,
                        end: 0.2,
                        duration: 800.ms,
                        curve: Curves.easeInOut,
                      ),
                if (isActive) const SizedBox(width: 12),

                Text(
                  label,
                  style: TextStyle(
                    color: isActive ? Colors.black : Colors.white,
                    fontSize: 28,
                    fontWeight: FontWeight.w900,
                    letterSpacing: 3,
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildStatusSidebar() {
    return Container(
      width: 420,
      padding: const EdgeInsets.all(28),
      decoration: BoxDecoration(
        color: const Color(0xFF0E0E17).withValues(alpha: 0.92),
        borderRadius: BorderRadius.circular(20),
        border: Border.all(
          color: Colors.cyanAccent,
          width: 3,
        ),
        boxShadow: [
          BoxShadow(
            color: Colors.cyanAccent.withValues(alpha: 0.3),
            blurRadius: 20,
            spreadRadius: 0,
          ),
        ],
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            'HYBRID ARCHITECTURE',
            style: TextStyle(
              color: Colors.cyanAccent,
              fontSize: 16,
              fontWeight: FontWeight.w700,
              letterSpacing: 2,
            ),
          ),
          const SizedBox(height: 16),
          Text(
            'A menu designed with bold typography, '
            'angled geometry, and neon synthwave accents.\n\n'
            'Use ↑/↓ to navigate, Enter to select.\n'
            'Or click/tap a command.\n\n'
            'Now powered by Flutter + Flame hybrid architecture.',
            style: TextStyle(
              color: const Color(0xFFEFEFEF),
              fontSize: 16,
              height: 1.5,
            ),
          ),
          const SizedBox(height: 20),
          Container(
            height: 2,
            decoration: BoxDecoration(
              gradient: LinearGradient(
                colors: [
                  Colors.cyanAccent,
                  Colors.purple.shade400,
                ],
              ),
            ),
          ),
          const SizedBox(height: 16),
          Row(
            children: [
              Icon(Icons.code, color: Colors.cyanAccent, size: 20),
              const SizedBox(width: 8),
              Text(
                'Flutter UI Layer Active',
                style: TextStyle(
                  color: Colors.cyanAccent.withValues(alpha: 0.8),
                  fontSize: 14,
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }

  void _handleKeyEvent(KeyEvent event) {
    if (event is! KeyDownEvent) return;

    if (event.logicalKey == LogicalKeyboardKey.arrowDown ||
        event.logicalKey == LogicalKeyboardKey.keyS) {
      setState(() {
        _selectedIndex = (_selectedIndex + 1) % _menuLabels.length;
      });
    } else if (event.logicalKey == LogicalKeyboardKey.arrowUp ||
        event.logicalKey == LogicalKeyboardKey.keyW) {
      setState(() {
        _selectedIndex =
            (_selectedIndex - 1 + _menuLabels.length) % _menuLabels.length;
      });
    } else if (event.logicalKey == LogicalKeyboardKey.enter ||
        event.logicalKey == LogicalKeyboardKey.space) {
      _triggerAction(_selectedIndex);
    }
  }

  void _triggerAction(int index) {
    switch (index) {
      case 0: // STORY
        widget.onStartGame();
        break;
      case 1: // TUTORIAL
        if (widget.onTutorial != null) {
          widget.onTutorial!();
        } else {
          debugPrint('[Menu] Tutorial coming soon');
        }
        break;
      case 2: // SETTINGS
        widget.onSettings();
        break;
      case 3: // DATA LOAD
        if (widget.onDataLoad != null) {
          widget.onDataLoad!();
        } else {
          debugPrint('[Menu] Data Load coming soon');
        }
        break;
      case 4: // DATA SAVE
        if (widget.onDataSave != null) {
          widget.onDataSave!();
        } else {
          debugPrint('[Menu] Data Save coming soon');
        }
        break;
      case 5: // RETURN TO TITLE
        if (widget.onReturnToTitle != null) {
          widget.onReturnToTitle!();
        } else {
          debugPrint('[Menu] Return to Title coming soon');
        }
        break;
    }
  }
}
