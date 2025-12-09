import 'package:flame/game.dart';
import 'package:flutter/material.dart';
import 'overlays/confirmation_dialog_overlay.dart';
import 'overlays/debug_console_overlay.dart';
import 'overlays/loading_screen_overlay.dart';
import 'overlays/pause_menu_overlay.dart';
import 'overlays/settings_form_overlay.dart';
import 'overlays/text_input_overlay.dart';
import 'syn_game.dart';
import 'ui/widgets/persona_container.dart';
import 'ui/widgets/magnetic_dock.dart';

void main() {
  final synGame = SynGame();
  runApp(
    MaterialApp(
      debugShowCheckedModeBanner: false,
      home: GameScreen(synGame: synGame),
    ),
  );
}

class GameScreen extends StatelessWidget {
  final SynGame synGame;

  const GameScreen({super.key, required this.synGame});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Stack(
        children: [
          // Layer 1 (Bottom): Flame GameWidget Background
          Positioned.fill(
            child: GameWidget(
              game: synGame,
              overlayBuilderMap: {
                'text_input': (context, game) =>
                    buildTextInputOverlay(context, game as SynGame),
                'pause_menu': (context, game) =>
                    buildPauseMenuOverlay(context, game as SynGame),
                'confirm_dialog': (context, game) =>
                    buildConfirmDialogOverlay(context, game as SynGame),
                'loading': (context, game) => buildLoadingOverlay(context),
                'settings_form': (context, game) =>
                    buildSettingsFormOverlay(context, game as SynGame),
                'debug_console': (context, game) =>
                    buildDebugConsoleOverlay(context, game as SynGame),
              },
            ),
          ),

          // Layer 2 (Top): Flutter UI Layer
          // Top Bar: DAY and MOOD
          Positioned(
            top: 20,
            left: 20,
            right: 20,
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                PersonaContainer(
                  color: Colors.black87,
                  child: Padding(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 24,
                      vertical: 12,
                    ),
                    child: Text(
                      'DAY 1',
                      style: TextStyle(
                        color: Colors.white,
                        fontSize: 24,
                        fontWeight: FontWeight.bold,
                        letterSpacing: 2,
                      ),
                    ),
                  ),
                ),
                PersonaContainer(
                  color: Colors.black87,
                  child: Padding(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 24,
                      vertical: 12,
                    ),
                    child: Text(
                      'MOOD: NEUTRAL',
                      style: TextStyle(
                        color: Colors.white,
                        fontSize: 24,
                        fontWeight: FontWeight.bold,
                        letterSpacing: 2,
                      ),
                    ),
                  ),
                ),
              ],
            ),
          ),

          // Left Dock: MagneticDock with Icons
          Align(
            alignment: Alignment.centerLeft,
            child: MagneticDock(
              isLeft: true,
              child: PersonaContainer(
                color: Colors.black87,
                child: Padding(
                  padding: const EdgeInsets.all(12.0),
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      _buildDockIcon(Icons.bar_chart, 'Stats'),
                      const SizedBox(height: 16),
                      _buildDockIcon(Icons.inventory, 'Inventory'),
                      const SizedBox(height: 16),
                      _buildDockIcon(Icons.people, 'Relations'),
                      const SizedBox(height: 16),
                      _buildDockIcon(Icons.map, 'Map'),
                    ],
                  ),
                ),
              ),
            ),
          ),

          // Center: Placeholder Event Card
          Center(
            child: PersonaContainer(
              color: Colors.black.withOpacity(0.9),
              child: Container(
                width: 400,
                padding: const EdgeInsets.all(32),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'EVENT CARD',
                      style: TextStyle(
                        color: Colors.red,
                        fontSize: 28,
                        fontWeight: FontWeight.bold,
                        letterSpacing: 3,
                      ),
                    ),
                    const SizedBox(height: 16),
                    Text(
                      'This is a placeholder event card. Events will appear here driven by the Rust simulation layer.',
                      style: TextStyle(
                        color: Colors.white,
                        fontSize: 16,
                        height: 1.5,
                      ),
                    ),
                    const SizedBox(height: 24),
                    Row(
                      mainAxisAlignment: MainAxisAlignment.end,
                      children: [
                        _buildChoiceButton('ACCEPT'),
                        const SizedBox(width: 12),
                        _buildChoiceButton('DECLINE'),
                      ],
                    ),
                  ],
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildDockIcon(IconData icon, String label) {
    return Tooltip(
      message: label,
      child: Container(
        padding: const EdgeInsets.all(12),
        decoration: BoxDecoration(
          color: Colors.white.withOpacity(0.1),
          border: Border.all(color: Colors.white, width: 1),
        ),
        child: Icon(
          icon,
          color: Colors.white,
          size: 28,
        ),
      ),
    );
  }

  Widget _buildChoiceButton(String label) {
    return PersonaContainer(
      color: Colors.red.withOpacity(0.8),
      skew: -0.1,
      child: Padding(
        padding: const EdgeInsets.symmetric(
          horizontal: 20,
          vertical: 10,
        ),
        child: Text(
          label,
          style: TextStyle(
            color: Colors.white,
            fontSize: 14,
            fontWeight: FontWeight.bold,
            letterSpacing: 1.5,
          ),
        ),
      ),
    );
  }
}
