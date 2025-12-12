import 'package:flame/game.dart';
import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';

import '../models/game_phase.dart';
import '../models/game_state.dart' as game_model;
import '../bridge/game_backend.dart';
import '../bridge/bridge_generated/lib.dart';
import '../syn_game.dart';
import '../ui/widgets/persona_container.dart';
import '../ui/widgets/magnetic_dock.dart';
import '../dev_tools/inspector_panel.dart';
import '../dev_tools/quake_console.dart';
import 'splash_screen.dart';
import 'main_menu_screen.dart';
import 'character_creation_screen.dart';
import 'end_of_life_screen.dart';
import '../overlays/text_input_overlay.dart';
import '../overlays/pause_menu_overlay.dart';
import '../overlays/confirmation_dialog_overlay.dart';
import '../overlays/loading_screen_overlay.dart';
import '../overlays/settings_form_overlay.dart';
import '../overlays/debug_console_overlay.dart';
import '../overlays/settings_overlay.dart';
import '../overlays/save_load_overlay.dart';
import '../panels/memory_journal_panel.dart';
import '../panels/detailed_stats_panel.dart';
import '../panels/inventory_panel.dart';
import '../ui/overlays/world_map_overlay.dart';
import '../ui/overlays/event_canvas_overlay.dart';
import '../ui/overlays/relationship_network_overlay.dart';
import '../ui/overlays/possession_overlay.dart';

/// Phase 1 Hybrid UI: Flame background + Flutter UI overlay
/// Phase 2 Dev Tools: Runtime inspector and debug console
///
/// Architecture:
/// - Layer 1 (Bottom): Flame GameWidget (SynGame for now, will be SynVisualsGame)
/// - Layer 2 (Top): Flutter widgets (HUD, Docks, Event Cards, Main Menu)
/// - Layer 3 (Dev Tools): Inspector Panel and Quake Console (toggleable)
/// - Style: Persona 5 aesthetic with skewed containers and high contrast
class GameScreen extends StatefulWidget {
  final SynGame synGame;

  const GameScreen({super.key, required this.synGame});

  @override
  State<GameScreen> createState() => _GameScreenState();
}

class _GameScreenState extends State<GameScreen> {
  late GameBackend _backend;
  game_model.GameState? _gameState;

  late GamePhase _phase;
  bool _showInspector = false;
  bool _showConsole = false;
  bool _showSettingsOverlay = false;
  bool _showSaveLoadOverlay = false;
  bool _showMemoryJournal = false;
  bool _showDetailedStats = false;
  bool _showInventory = false;
  bool _showWorldMap = false;
  bool _showEventCanvas = false;
  bool _showRelationshipNetwork = false;
  bool _showPossession = false;
  String _saveLoadMode = 'load'; // 'save' or 'load'

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
    _phase = GamePhase.mainMenu;

    // Initialize GameBackend asynchronously
    Future.microtask(() async {
      _backend = await GameBackend.create();
      // Optionally pre-load an existing state here.
      setState(() {});
    });
  }

  void _setPhase(GamePhase phase) => setState(() => _phase = phase);

  String _getMoodLabel(int mood) {
    if (mood >= 7) return 'ECSTATIC';
    if (mood >= 4) return 'HAPPY';
    if (mood >= 1) return 'CONTENT';
    if (mood >= -1) return 'NEUTRAL';
    if (mood >= -4) return 'SAD';
    if (mood >= -7) return 'DEPRESSED';
    return 'MISERABLE';
  }

  Future<void> _handleEventChoiceSelected(String storyletId, String choiceId) async {
    debugPrint('[GameScreen] Choice selected: $choiceId for event: $storyletId');
    
    // Call backend to process choice (0 ticks = process immediately)
    final newState = await _backend.chooseOption(storyletId, choiceId, 0);
    
    setState(() {
      _gameState = newState;
    });
  }

  void _handleNewGameRequested() {
    _setPhase(GamePhase.characterCreation);
  }

  void _handleLoadRequested() {
    // later: load from backend
    _setPhase(GamePhase.gameplay);
  }

  void _handleSettingsRequested() {
    // for now, just log; later can show a settings overlay
    debugPrint('Settings requested');
  }

  void _handleQuitRequested() {
    debugPrint('Quit requested (not implemented)');
  }

  Future<void> _handleCharacterCreated({
    required String name,
    required String archetype,
    required bool sfwMode,
    required String difficulty,
  }) async {
    // Create API config from character creation data
    final apiConfig = ApiPlayerConfig(
      name: name,
      pronouns: null, // Not collected in current UI
      archetype: archetype,
      difficulty: difficulty,
      sfwMode: sfwMode,
    );

    // Generate seed from current timestamp for deterministic simulation
    final seed = DateTime.now().microsecondsSinceEpoch;

    // Initialize new game via Rust backend
    final newState = await _backend.newGame(seed, apiConfig);

    setState(() {
      _gameState = newState;
      _phase = GamePhase.gameplay;
    });

    debugPrint('[GameScreen] New game created with seed $seed');
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: Colors.black,
      body: Stack(
        children: [
          // Layer 1 (Bottom): Flame GameWidget Background
          Positioned.fill(
            child: GameWidget(
              game: widget.synGame,
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

          // Layer 2 (Top): Flutter UI based on phase
          Positioned.fill(child: _buildPhaseUi()),

          // Overlays (conditional)
          if (_showSettingsOverlay) _buildSettingsOverlay(),
          if (_showSaveLoadOverlay) _buildSaveLoadOverlay(),
          if (_showMemoryJournal) _buildMemoryJournalPanel(),
          if (_showDetailedStats) _buildDetailedStatsPanel(),
          if (_showInventory) _buildInventoryPanel(),
          if (_showWorldMap) _buildWorldMapOverlay(),
          if (_showEventCanvas) _buildEventCanvasOverlay(),
          if (_showRelationshipNetwork) _buildRelationshipNetworkOverlay(),
          if (_showPossession) _buildPossessionOverlay(),

          // Layer 3 (Dev Tools): Inspector Panel (Right)
          if (_showInspector && _phase == GamePhase.gameplay)
            Positioned(
              top: 0,
              right: 0,
              bottom: 0,
              child: InspectorPanel(game: widget.synGame)
                  .animate()
                  .slideX(
                    begin: 1,
                    duration: 300.ms,
                    curve: Curves.easeOutExpo,
                  )
                  .fadeIn(duration: 200.ms),
            ),

          // Layer 3 (Dev Tools): Quake Console (Top)
          if (_showConsole && _phase == GamePhase.gameplay)
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
          if (_phase == GamePhase.gameplay)
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

  Widget _buildPhaseUi() {
    switch (_phase) {
      case GamePhase.splash:
        return SplashScreen(
          onFinish: () => _setPhase(GamePhase.mainMenu),
        );
      case GamePhase.mainMenu:
        return MainMenuScreen(
          onNewGame: _handleNewGameRequested,
          onLoadGame: _handleLoadRequested,
          onSettings: _handleSettingsRequested,
          onQuit: _handleQuitRequested,
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
            _handleCharacterCreated(
              name: name,
              archetype: archetype,
              sfwMode: sfwMode,
              difficulty: difficulty,
            );
          },
        );
      case GamePhase.gameplay:
        return _buildGameplayUi();
      case GamePhase.endOfLife:
        return EndOfLifeScreen(
          lifeSummary: _lifeSummary,
          onRestart: () => _setPhase(GamePhase.characterCreation),
          onReturnToTitle: () => _setPhase(GamePhase.mainMenu),
        );
      case GamePhase.postLife:
        return Center(
          child: Text(
            'Post-Life WIP',
            style: TextStyle(color: Colors.white, fontSize: 32),
          ),
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

  Widget _buildGameplayUi() {
    // Show loading indicator while game state is being initialized
    if (_gameState == null) {
      return const Center(child: CircularProgressIndicator());
    }

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

        // Center: Event Canvas (displays current storylet)
        Center(
          child: EventCanvasOverlay(
            gameState: _gameState!,
            onChoiceSelected: _handleEventChoiceSelected,
          ),
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
              'YEAR: ${_gameState!.year} | AGE: ${_gameState!.age}',
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
              'MOOD: ${_getMoodLabel(_gameState!.mood)}',
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
            _buildDockIcon(
              Icons.people,
              'Relations',
              () => setState(() => _showRelationshipNetwork = true),
            ),
            const SizedBox(height: 16),
            _buildDockIcon(
              Icons.map,
              'Map',
              () => setState(() => _showWorldMap = true),
            ),
            const SizedBox(height: 16),
            _buildDockIcon(
              Icons.book,
              'Journal',
              () => setState(() => _showMemoryJournal = true),
            ),
            const SizedBox(height: 16),
            _buildDockIcon(
              Icons.sync_alt,
              'Possession',
              () => setState(() => _showPossession = true),
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

  Widget _buildWorldMapOverlay() {
    return WorldMapOverlay(
      onClose: () => setState(() => _showWorldMap = false),
      onRegionSelect: (regionId) {
        debugPrint('[WorldMap] Selected region: $regionId');
        // TODO: Implement region navigation
        setState(() => _showWorldMap = false);
      },
      regions: const [], // Use default placeholder regions
    );
  }

  Widget _buildEventCanvasOverlay() {
    // If no game state yet, show nothing
    if (_gameState == null) {
      return const SizedBox.shrink();
    }

    return EventCanvasOverlay(
      gameState: _gameState!,
      onChoiceSelected: (String storyletId, String choiceId) async {
        debugPrint('[EventCanvas] Selected choice: $choiceId for event: $storyletId');
        
        // Call Rust backend to process choice
        final updatedState = await _backend.chooseOption(storyletId, choiceId, 24);
        
        setState(() {
          _gameState = updatedState;
          _showEventCanvas = false;
        });
      },
    );
  }

  Widget _buildRelationshipNetworkOverlay() {
    return RelationshipNetworkOverlay(
      onClose: () => setState(() => _showRelationshipNetwork = false),
      relationships: const [], // Use default placeholder data
    );
  }

  Widget _buildPossessionOverlay() {
    return PossessionOverlay(
      onClose: () => setState(() => _showPossession = false),
      onHostSelect: (hostId) {
        debugPrint('[Possession] Selected host: $hostId');
        // TODO: Implement possession logic with Rust backend
        setState(() => _showPossession = false);
      },
      hosts: const [], // Use default placeholder hosts
    );
  }
}
