import 'package:flame/game.dart';
import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';

import '../syn_game.dart';
import '../ui/widgets/persona_container.dart';
import '../ui/widgets/magnetic_dock.dart';
import '../dev_tools/inspector_panel.dart';
import '../dev_tools/quake_console.dart';

/// Phase 1 Hybrid UI: Flame background + Flutter UI overlay
/// Phase 2 Dev Tools: Runtime inspector and debug console
///
/// Architecture:
/// - Layer 1 (Bottom): Flame GameWidget (SynGame for now, will be SynVisualsGame)
/// - Layer 2 (Top): Flutter widgets (HUD, Docks, Event Cards)
/// - Layer 3 (Dev Tools): Inspector Panel and Quake Console (toggleable)
/// - Style: Persona 5 aesthetic with skewed containers and high contrast
class GameScreen extends StatefulWidget {
  const GameScreen({super.key});

  @override
  State<GameScreen> createState() => _GameScreenState();
}

class _GameScreenState extends State<GameScreen> {
  late final SynGame _game;
  bool _showInspector = false;
  bool _showConsole = false;

  @override
  void initState() {
    super.initState();
    _game = SynGame();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.transparent,
      body: Stack(
        children: [
          // Layer 1 (Bottom): Flame GameWidget Background
          // TODO: Replace SynGame with SynVisualsGame once city background is implemented
          Positioned.fill(
            child: GameWidget(
              game: _game,
            ),
          ),

          // Layer 2 (Top): Flutter UI Overlay
          _buildFlutterUILayer(),

          // Layer 3 (Dev Tools): Inspector Panel (Right)
          if (_showInspector)
            Positioned(
              top: 0,
              right: 0,
              bottom: 0,
              child: InspectorPanel(game: _game)
                  .animate()
                  .slideX(
                    begin: 1,
                    duration: 300.ms,
                    curve: Curves.easeOutExpo,
                  )
                  .fadeIn(duration: 200.ms),
            ),

          // Layer 3 (Dev Tools): Quake Console (Top)
          if (_showConsole)
            Positioned(
              top: 0,
              left: 0,
              right: 0,
              child: QuakeConsole(
                onCommand: _handleConsoleCommand,
              )
                  .animate()
                  .slideY(
                    begin: -1,
                    duration: 300.ms,
                    curve: Curves.easeOutExpo,
                  )
                  .fadeIn(duration: 200.ms),
            ),

          // Dev Tools Toggle Buttons (Bottom Right)
          Positioned(
            bottom: 20,
            right: 20,
            child: _buildDevToolsButtons()
                .animate()
                .fadeIn(delay: 600.ms, duration: 400.ms),
          ),
        ],
      ),
    );
  }

  void _handleConsoleCommand(String command) {
    // Mock implementation - just print to debug console
    // TODO: Wire this up to the Rust bridge for actual command execution
    debugPrint('[Console] Command executed: $command');
    
    // Future examples:
    // - 'spawn entity <type>' -> Call Rust to spawn an entity
    // - 'set_stat health 100' -> Update player stats
    // - 'trigger_event <id>' -> Force trigger a storylet
    // - 'timeskip <days>' -> Advance simulation time
  }

  Widget _buildFlutterUILayer() {
    return Stack(
      children: [
        // Top Bar: DAY and MOOD indicators
        Positioned(
          top: 20,
          left: 20,
          right: 20,
          child: _buildTopBar()
              .animate()
              .slideY(begin: -1, duration: 600.ms, curve: Curves.easeOutExpo)
              .fadeIn(duration: 400.ms),
        ),

        // Left Dock: Stats, Inventory, etc.
        Align(
          alignment: Alignment.centerLeft,
          child: MagneticDock(
            isLeft: true,
            child: _buildLeftDock(),
          ).animate().fadeIn(delay: 200.ms, duration: 400.ms),
        ),

        // Right Dock: Relations, Map, etc.
        Align(
          alignment: Alignment.centerRight,
          child: MagneticDock(
            isLeft: false,
            child: _buildRightDock(),
          ).animate().fadeIn(delay: 300.ms, duration: 400.ms),
        ),

        // Center: Placeholder Event Card
        Align(
          alignment: Alignment.center,
          child: _buildEventCard()
              .animate()
              .scale(
                begin: const Offset(0.8, 0.8),
                duration: 500.ms,
                curve: Curves.easeOutExpo,
              )
              .fadeIn(delay: 400.ms, duration: 400.ms),
        ),
      ],
    );
  }

  Widget _buildTopBar() {
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceBetween,
      children: [
        PersonaContainer(
          color: Colors.black,
          child: Padding(
            padding: const EdgeInsets.symmetric(
              horizontal: 24,
              vertical: 12,
            ),
            child: Text(
              'DAY: 1',
              style: TextStyle(
                color: Colors.cyanAccent,
                fontSize: 20,
                fontWeight: FontWeight.bold,
                letterSpacing: 2,
              ),
            ),
          ),
        ),
        PersonaContainer(
          color: Colors.black,
          child: Padding(
            padding: const EdgeInsets.symmetric(
              horizontal: 24,
              vertical: 12,
            ),
            child: Text(
              'MOOD: NEUTRAL',
              style: TextStyle(
                color: Colors.cyanAccent,
                fontSize: 20,
                fontWeight: FontWeight.bold,
                letterSpacing: 2,
              ),
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildLeftDock() {
    return PersonaContainer(
      color: Colors.black,
      child: Padding(
        padding: const EdgeInsets.all(12.0),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            _buildDockIcon(Icons.bar_chart, 'Stats'),
            const SizedBox(height: 16),
            _buildDockIcon(Icons.inventory_2, 'Inventory'),
            const SizedBox(height: 16),
            _buildDockIcon(Icons.calendar_today, 'Calendar'),
            const SizedBox(height: 16),
            _buildDockIcon(Icons.settings, 'Settings'),
          ],
        ),
      ),
    );
  }

  Widget _buildRightDock() {
    return PersonaContainer(
      color: Colors.black,
      child: Padding(
        padding: const EdgeInsets.all(12.0),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            _buildDockIcon(Icons.people, 'Relations'),
            const SizedBox(height: 16),
            _buildDockIcon(Icons.map, 'Map'),
            const SizedBox(height: 16),
            _buildDockIcon(Icons.book, 'Journal'),
            const SizedBox(height: 16),
            _buildDockIcon(Icons.menu_book, 'Memories'),
          ],
        ),
      ),
    );
  }

  Widget _buildDockIcon(IconData icon, String label) {
    return Tooltip(
      message: label,
      child: Container(
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          color: Colors.cyanAccent.withValues(alpha: 0.1),
          border: Border.all(color: Colors.cyanAccent, width: 1),
        ),
        child: Icon(
          icon,
          color: Colors.cyanAccent,
          size: 28,
        ),
      ),
    );
  }

  Widget _buildEventCard() {
    return PersonaContainer(
      color: Colors.black.withValues(alpha: 0.95),
      child: Container(
        width: 500,
        padding: const EdgeInsets.all(32),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            // Event Card Header
            Text(
              'A NEW DAY BEGINS',
              style: TextStyle(
                color: Colors.cyanAccent,
                fontSize: 28,
                fontWeight: FontWeight.bold,
                letterSpacing: 3,
              ),
            ),
            const SizedBox(height: 8),
            Container(
              height: 2,
              color: Colors.cyanAccent,
            ),
            const SizedBox(height: 16),
            
            // Event Description
            Text(
              'Welcome to SYN: Simulate Your Narrative. This is a placeholder event card. '
              'Events will be driven by the Rust simulation layer through the Event Director.',
              style: TextStyle(
                color: Colors.white,
                fontSize: 16,
                height: 1.6,
              ),
            ),
            const SizedBox(height: 24),
            
            // Choice Buttons
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                _buildChoiceButton('CONTINUE', Colors.cyanAccent),
                const SizedBox(width: 12),
                _buildChoiceButton('SKIP', Colors.grey),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildChoiceButton(String label, Color accentColor) {
    return PersonaContainer(
      color: Colors.black,
      skew: -0.1,
      child: Container(
        decoration: BoxDecoration(
          border: Border.all(color: accentColor, width: 2),
        ),
        child: Padding(
          padding: const EdgeInsets.symmetric(
            horizontal: 24,
            vertical: 12,
          ),
          child: Text(
            label,
            style: TextStyle(
              color: accentColor,
              fontSize: 14,
              fontWeight: FontWeight.bold,
              letterSpacing: 1.5,
            ),
          ),
        ),
      ),
    );
  }

  /// Dev Tools toggle buttons (Inspector and Console)
  Widget _buildDevToolsButtons() {
    return Container(
      decoration: BoxDecoration(
        color: Colors.black.withValues(alpha: 0.8),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.cyanAccent.withValues(alpha: 0.5), width: 1),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          IconButton(
            icon: Icon(
              Icons.code,
              color: _showConsole ? Colors.green : Colors.grey,
              size: 24,
            ),
            onPressed: () {
              setState(() {
                _showConsole = !_showConsole;
              });
            },
            tooltip: 'Toggle Console',
          ),
          Container(
            width: 1,
            height: 24,
            color: Colors.cyanAccent.withValues(alpha: 0.3),
          ),
          IconButton(
            icon: Icon(
              Icons.tune,
              color: _showInspector ? Colors.cyanAccent : Colors.grey,
              size: 24,
            ),
            onPressed: () {
              setState(() {
                _showInspector = !_showInspector;
              });
            },
            tooltip: 'Toggle Inspector',
          ),
        ],
      ),
    );
  }
}
