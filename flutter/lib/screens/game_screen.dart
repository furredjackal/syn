import 'package:flame/game.dart';
import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';

import '../syn_game.dart';
import '../ui/widgets/persona_container.dart';
import '../ui/widgets/magnetic_dock.dart';
import '../dev_tools/inspector_panel.dart';
import '../dev_tools/quake_console.dart';
import 'splash_screen.dart';
import 'main_menu_screen.dart';
import 'character_creation_screen.dart';
import 'end_of_life_screen.dart';
import '../overlays/settings_overlay.dart';
import '../overlays/save_load_overlay.dart';
import '../panels/memory_journal_panel.dart';
import '../panels/detailed_stats_panel.dart';
import '../panels/inventory_panel.dart';

/// Game phase enum to track which UI to display
enum GamePhase {
  splash,
  mainMenu,
  characterCreation,
  gameplay,
  endOfLife,
}

/// Phase 1 Hybrid UI: Flame background + Flutter UI overlay
/// Phase 2 Dev Tools: Runtime inspector and debug console
///
/// Architecture:
/// - Layer 1 (Bottom): Flame GameWidget (SynGame for now, will be SynVisualsGame)
/// - Layer 2 (Top): Flutter widgets (HUD, Docks, Event Cards, Main Menu)
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
  bool _showSettingsOverlay = false;
  bool _showSaveLoadOverlay = false;
  bool _showMemoryJournal = false;
  bool _showDetailedStats = false;
  bool _showInventory = false;
  String _saveLoadMode = 'load'; // 'save' or 'load'
  GamePhase _currentPhase = GamePhase.splash;

  // Mock data for panels
  Map<String, dynamic> _lifeSummary = {};
  List<MemoryEntry> _memories = [];
  Map<String, Map<String, double>> _stats = {
    'core': {'Health': 80, 'Energy': 65, 'Mood': 75},
    'social': {'Charisma': 55, 'Empathy': 70, 'Influence': 45},
    'skills': {'Intelligence': 80, 'Creativity': 90, 'Athletic': 40},
  };
  List<InventoryItem> _inventory = [];
  List<SaveSlotData> _saveSlots = List.generate(
    6,
    (i) => i == 0
        ? SaveSlotData(
            name: 'Auto Save',
            age: 25,
            day: 42,
            timestamp: '2024-01-15 14:32',
          )
        : SaveSlotData.empty(),
  );

  // Audio settings state
  bool _audioEnabled = true;
  bool _sfxEnabled = true;
  double _musicVolume = 0.7;
  double _sfxVolume = 0.8;

  @override
  void initState() {
    super.initState();
    _game = SynGame();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.black,
      body: Stack(
        children: [
          // Layer 1 (Bottom): Flame GameWidget Background
          if (_currentPhase == GamePhase.gameplay)
            Positioned.fill(
              child: GameWidget(
                game: _game,
              ),
            ),

          // Layer 2 (Top): Flutter UI based on phase
          _buildPhaseUI(),

          // Overlays (conditional)
          if (_showSettingsOverlay) _buildSettingsOverlay(),
          if (_showSaveLoadOverlay) _buildSaveLoadOverlay(),
          if (_showMemoryJournal) _buildMemoryJournalPanel(),
          if (_showDetailedStats) _buildDetailedStatsPanel(),
          if (_showInventory) _buildInventoryPanel(),

          // Layer 3 (Dev Tools): Inspector Panel (Right)
          if (_showInspector && _currentPhase == GamePhase.gameplay)
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
          if (_showConsole && _currentPhase == GamePhase.gameplay)
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

          // Dev Tools Toggle Buttons (Bottom Right) - Only in gameplay
          if (_currentPhase == GamePhase.gameplay)
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

  Widget _buildPhaseUI() {
    switch (_currentPhase) {
      case GamePhase.splash:
        return SplashScreen(
          onFinish: () => setState(() => _currentPhase = GamePhase.mainMenu),
        );
      case GamePhase.mainMenu:
        return MainMenuScreen(
          onStartGame: () {
            setState(() => _currentPhase = GamePhase.characterCreation);
            _game.showCharacterCreation();
          },
          onSettings: () => setState(() => _showSettingsOverlay = true),
          onDataLoad: () => setState(() {
            _saveLoadMode = 'load';
            _showSaveLoadOverlay = true;
          }),
          onDataSave: () => setState(() {
            _saveLoadMode = 'save';
            _showSaveLoadOverlay = true;
          }),
          onReturnToTitle: () => setState(() => _currentPhase = GamePhase.splash),
        );
      case GamePhase.characterCreation:
        return CharacterCreationScreen(
          onComplete: ({
            required String name,
            required String archetype,
            required bool sfwMode,
            required String difficulty,
          }) {
            debugPrint('[CharacterCreation] Name: $name, Archetype: $archetype');
            setState(() => _currentPhase = GamePhase.gameplay);
            // TODO: Pass character data to Rust simulation
          },
        );
      case GamePhase.gameplay:
        return _buildGameplayUI();
      case GamePhase.endOfLife:
        return EndOfLifeScreen(
          lifeSummary: _lifeSummary,
          onRestart: () {
            setState(() => _currentPhase = GamePhase.characterCreation);
          },
          onReturnToTitle: () {
            setState(() => _currentPhase = GamePhase.mainMenu);
          },
        );
    }
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

  Widget _buildGameplayUI() {
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
            _buildDockIcon(
              Icons.bar_chart,
              'Stats',
              () => setState(() => _showDetailedStats = true),
            ),
            const SizedBox(height: 16),
            _buildDockIcon(
              Icons.inventory_2,
              'Inventory',
              () => setState(() => _showInventory = true),
            ),
            const SizedBox(height: 16),
            _buildDockIcon(Icons.calendar_today, 'Calendar', () {}),
            const SizedBox(height: 16),
            _buildDockIcon(
              Icons.settings,
              'Settings',
              () => setState(() => _showSettingsOverlay = true),
            ),
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
            _buildDockIcon(Icons.people, 'Relations', () {}),
            const SizedBox(height: 16),
            _buildDockIcon(Icons.map, 'Map', () {}),
            const SizedBox(height: 16),
            _buildDockIcon(Icons.book, 'Journal', () {}),
            const SizedBox(height: 16),
            _buildDockIcon(
              Icons.menu_book,
              'Memories',
              () => setState(() => _showMemoryJournal = true),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildDockIcon(IconData icon, String label, VoidCallback onTap) {
    return GestureDetector(
      onTap: onTap,
      child: Tooltip(
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

  // Overlay builders
  Widget _buildSettingsOverlay() {
    return SettingsOverlay(
      onClose: () => setState(() => _showSettingsOverlay = false),
      audioEnabled: _audioEnabled,
      sfxEnabled: _sfxEnabled,
      musicVolume: _musicVolume,
      sfxVolume: _sfxVolume,
      onAudioToggle: (value) => setState(() => _audioEnabled = value),
      onSfxToggle: (value) => setState(() => _sfxEnabled = value),
      onMusicVolumeChange: (value) => setState(() => _musicVolume = value),
      onSfxVolumeChange: (value) => setState(() => _sfxVolume = value),
    );
  }

  Widget _buildSaveLoadOverlay() {
    return SaveLoadOverlay(
      onClose: () => setState(() => _showSaveLoadOverlay = false),
      onSave: (index, name) {
        debugPrint('[SaveLoad] Saving to slot $index with name: $name');
        // TODO: Implement actual save logic
        setState(() => _showSaveLoadOverlay = false);
      },
      onLoad: (index) {
        debugPrint('[SaveLoad] Loading from slot $index');
        // TODO: Implement actual load logic
        setState(() => _showSaveLoadOverlay = false);
      },
      saveSlots: _saveSlots,
      mode: _saveLoadMode,
    );
  }

  Widget _buildMemoryJournalPanel() {
    return MemoryJournalPanel(
      onClose: () => setState(() => _showMemoryJournal = false),
      memories: _memories,
    );
  }

  Widget _buildDetailedStatsPanel() {
    return DetailedStatsPanel(
      onClose: () => setState(() => _showDetailedStats = false),
      stats: _stats,
    );
  }

  Widget _buildInventoryPanel() {
    return InventoryPanel(
      onClose: () => setState(() => _showInventory = false),
      items: _inventory,
      onItemSelect: (item) {
        debugPrint('[Inventory] Selected: ${item.name}');
      },
    );
  }
}
