import 'package:flame/game.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_animate/flutter_animate.dart';

import '../models/game_phase.dart';
import '../models/game_state.dart' as game_model;
import '../bridge/game_backend.dart';
import '../bridge/bridge_generated/lib.dart';
import '../syn_game.dart';
import '../ui/syn_ui.dart';
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
import '../ui/overlays/relationship_network_overlay.dart'
    show RelationshipNetworkOverlay, CharacterRelationship;
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
  late QuakeConsoleController _consoleController;
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
  bool _showKeyboardShortcuts = false;
  String _saveLoadMode = 'load'; // 'save' or 'load'

  // Time control state
  bool _isPaused = false;
  int _speedMultiplier = 1;
  int _currentHour = 8; // 8 AM default

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
    _consoleController = QuakeConsoleController();

    // Initialize GameBackend asynchronously
    Future.microtask(() async {
      _backend = await GameBackend.create();
      _consoleController.info('GameBackend initialized');
      // Optionally pre-load an existing state here.
      setState(() {});
    });
  }

  @override
  void dispose() {
    _consoleController.dispose();
    super.dispose();
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
    // Wrap in keyboard listener for global shortcuts
    return KeyboardListener(
      focusNode: FocusNode(),
      autofocus: true,
      onKeyEvent: _handleKeyEvent,
      child: Scaffold(
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

            // Layer 1.5: Mood-reactive overlay (only during gameplay)
            if (_phase == GamePhase.gameplay && _gameState != null)
              Positioned.fill(
                child: IgnorePointer(
                  child: MoodBackground(
                    mood: _gameState!.mood.toDouble(),
                    child: const SizedBox.expand(),
                  ),
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
            if (_showKeyboardShortcuts) _buildKeyboardShortcutsOverlay(),

            // Layer 3 (Dev Tools): Inspector Panel (Right)
            if (_showInspector && _phase == GamePhase.gameplay)
              Positioned(
                top: 0,
                right: 0,
                bottom: 0,
                child: InspectorPanel(game: widget.synGame, appContext: context)
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
                  controller: _consoleController,
                  onCommand: _handleConsoleCommand,
                  onClose: () => setState(() => _showConsole = false),
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
                bottom: 100, // Moved up to make room for bottom bar
                right: 20,
                child: _buildDevToolsButtons()
                    .animate()
                    .fadeIn(delay: 600.ms, duration: 400.ms),
              ),
          ],
        ),
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
    final parts = command.trim().split(RegExp(r'\s+'));
    if (parts.isEmpty) return;

    final cmd = parts[0].toLowerCase();
    final args = parts.skip(1).toList();

    try {
      switch (cmd) {
        // === SIMULATION COMMANDS (via GameBackend/Rust) ===
        case 'step':
        case 'tick':
          final ticks = args.isNotEmpty ? int.tryParse(args[0]) ?? 1 : 1;
          _consoleController.info('Advancing simulation $ticks tick(s)...');
          _backend.step(ticks).then((newState) {
            setState(() => _gameState = newState);
            _consoleController.info('Simulation advanced (Year ${newState.year}, Age ${newState.age})');
          }).catchError((e) {
            _consoleController.error('Failed to advance: $e');
          });
          break;

        case 'state':
          _consoleController.info('Current State:');
          _consoleController.addLog('  Year: ${_gameState?.year}, Age: ${_gameState?.age}');
          _consoleController.addLog('  Mood: ${_gameState?.mood}');
          break;

        case 'event':
          if (_gameState?.currentEvent != null) {
            final evt = _gameState!.currentEvent!;
            _consoleController.info('Active Event: ${evt.title}');
            _consoleController.addLog('  Choices: ${evt.choices.map((c) => c.text).join(", ")}');
          } else {
            _consoleController.warn('No active event');
          }
          break;

        // === VISUAL DEBUG COMMANDS (via SynGame/Flame) ===
        case 'fps':
          // Toggle FPS display in Flame
          widget.synGame.debugMode = !widget.synGame.debugMode;
          _consoleController.info('FPS display ${widget.synGame.debugMode ? "enabled" : "disabled"}');
          break;

        case 'spawn':
          if (args.isEmpty) {
            _consoleController.warn('Usage: spawn <type>');
          } else {
            _consoleController.info('Spawning ${args[0]} (visual debug)');
            // TODO: Call widget.synGame.spawnDebugEntity(args[0]);
            _consoleController.warn('Not yet implemented');
          }
          break;

        case 'layer':
          if (args.isEmpty) {
            _consoleController.warn('Usage: layer <name> [on|off]');
          } else {
            _consoleController.info('Toggling layer ${args[0]}');
            // TODO: Call widget.synGame.toggleLayer(args[0], toggle?)
            _consoleController.warn('Not yet implemented');
          }
          break;

        // === UTILITY COMMANDS ===
        case 'help':
          _consoleController.info('Available Commands:');
          _consoleController.addLog('  Simulation: step [n], state, event');
          _consoleController.addLog('  Visual: fps, spawn <type>, layer <name> [on|off]');
          _consoleController.addLog('  Utility: help, clear');
          break;

        case 'clear':
          _consoleController.clear();
          break;

        default:
          _consoleController.warn('Unknown command: $cmd (try "help")');
      }
    } catch (e, stack) {
      _consoleController.error('Command failed: $e');
      debugPrint('Console command error: $e\n$stack');
    }
  }

  Widget _buildGameplayUi() {
    // Show loading indicator while game state is being initialized
    if (_gameState == null) {
      return const Center(child: CircularProgressIndicator());
    }

    return Stack(
      children: [
        // Top Bar: DAY and MOOD indicators + Stat Rings
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

        // Bottom Bar: Time controls, quick actions, mini stats
        Positioned(
          bottom: 20,
          left: 20,
          right: 20,
          child: _buildBottomBar()
              .animate()
              .slideY(begin: 1, duration: 600.ms, curve: Curves.easeOutExpo)
              .fadeIn(duration: 400.ms),
        ),
      ],
    );
  }

  Widget _buildTopBar() {
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceBetween,
      children: [
        // Left: Year/Age info
        SynContainer(
          enableHover: false,
          child: Padding(
            padding: const EdgeInsets.symmetric(
              horizontal: 24,
              vertical: 12,
            ),
            child: Text(
              'YEAR: ${_gameState!.year} | AGE: ${_gameState!.age}',
              style: SynTheme.label(color: SynTheme.accent),
            ),
          ),
        ),
        
        // Center: Stat rings
        Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            SynStatRing(
              value: _gameState!.health.toDouble().clamp(0, 100),
              label: 'Health',
              icon: Icons.favorite,
              color: SynTheme.accentHot,
              size: 80,
              thickness: 6,
            ),
            const SizedBox(width: 12),
            SynStatRing(
              value: _gameState!.energy.toDouble().clamp(0, 100),
              label: 'Energy',
              icon: Icons.bolt,
              color: SynTheme.accentWarm,
              size: 80,
              thickness: 6,
            ),
            const SizedBox(width: 12),
            SynStatRing(
              value: ((_gameState!.mood + 10) * 5).toDouble().clamp(0, 100),
              label: 'Mood',
              icon: Icons.sentiment_satisfied,
              size: 80,
              thickness: 6,
            ),
          ],
        ),
        
        // Right: Mood label
        SynContainer(
          enableHover: false,
          child: Padding(
            padding: const EdgeInsets.symmetric(
              horizontal: 24,
              vertical: 12,
            ),
            child: Text(
              'MOOD: ${_getMoodLabel(_gameState!.mood)}',
              style: SynTheme.label(color: SynTheme.accent),
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildLeftDock() {
    return SynContainer(
      enableHover: false,
      skew: -0.08,
      child: Padding(
        padding: const EdgeInsets.all(12.0),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            SynDockIcon(
              icon: Icons.bar_chart,
              label: 'Stats',
              onTap: () => setState(() => _showDetailedStats = true),
            ),
            const SizedBox(height: 16),
            SynDockIcon(
              icon: Icons.inventory_2,
              label: 'Inventory',
              onTap: () => setState(() => _showInventory = true),
            ),
            const SizedBox(height: 16),
            SynDockIcon(
              icon: Icons.calendar_today,
              label: 'Calendar',
              onTap: () {},
            ),
            const SizedBox(height: 16),
            SynDockIcon(
              icon: Icons.settings,
              label: 'Settings',
              onTap: () => setState(() => _showSettingsOverlay = true),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildRightDock() {
    return SynContainer(
      enableHover: false,
      skew: 0.08,
      child: Padding(
        padding: const EdgeInsets.all(12.0),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            SynDockIcon(
              icon: Icons.people,
              label: 'Relations',
              onTap: () => setState(() => _showRelationshipNetwork = true),
            ),
            const SizedBox(height: 16),
            SynDockIcon(
              icon: Icons.map,
              label: 'Map',
              onTap: () => setState(() => _showWorldMap = true),
            ),
            const SizedBox(height: 16),
            SynDockIcon(
              icon: Icons.book,
              label: 'Journal',
              onTap: () => setState(() => _showMemoryJournal = true),
            ),
            const SizedBox(height: 16),
            SynDockIcon(
              icon: Icons.sync_alt,
              label: 'Possession',
              onTap: () => setState(() => _showPossession = true),
            ),
          ],
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
    // TODO: Convert GameState.memories to panel MemoryEntry format
    // For now, use mock data
    return MemoryJournalPanel(
      onClose: () => setState(() => _showMemoryJournal = false),
      memories: _memories,
    );
  }

  Widget _buildDetailedStatsPanel() {
    // Convert GameState stats to the format expected by DetailedStatsPanel
    // These are REAL stats from the Rust simulation backend
    final stats = _gameState != null ? {
      'core': {
        'Health': _gameState!.health.toDouble(),
        'Energy': _gameState!.energy.toDouble(),
        'Mood': (_gameState!.mood + 10) * 5.0, // Convert -10..10 to 0..100
      },
      'social': {
        'Charisma': _gameState!.charisma.toDouble(),
        'Reputation': _gameState!.reputation.toDouble(),
        'Wealth': _gameState!.wealth.toDouble(),
      },
      'skills': {
        'Intelligence': _gameState!.intelligence.toDouble(),
        'Wisdom': _gameState!.wisdom.toDouble(),
        'Curiosity': _gameState!.curiosity.toDouble(),
      },
    } : _stats;

    return DetailedStatsPanel(
      onClose: () => setState(() => _showDetailedStats = false),
      stats: stats,
    );
  }

  Widget _buildInventoryPanel() {
    // TODO: Add inventory field to GameState
    // For now, use mock data
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
    // Convert GameState.relationships (RelationshipData) to CharacterRelationship
    final relationships = _gameState?.relationships.map((rel) {
      return CharacterRelationship(
        name: rel.npcName,
        role: rel.state, // Friend, CloseFriend, etc.
        strength: ((rel.affection + rel.trust) / 2 + 10) / 2, // Convert -10..10 to 0..10
      );
    }).toList() ?? [];
    
    return RelationshipNetworkOverlay(
      onClose: () => setState(() => _showRelationshipNetwork = false),
      relationships: relationships,
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

  Widget _buildKeyboardShortcutsOverlay() {
    return KeyboardShortcutsOverlay(
      onClose: () => setState(() => _showKeyboardShortcuts = false),
    );
  }

  Widget _buildBottomBar() {
    return SynBottomBar(
      day: _gameState!.day,
      year: _gameState!.year,
      hour: _currentHour,
      health: _gameState!.health.toDouble().clamp(0, 100),
      energy: _gameState!.energy.toDouble().clamp(0, 100),
      mood: ((_gameState!.mood + 10) * 5).toDouble().clamp(0, 100),
      isPaused: _isPaused,
      speedMultiplier: _speedMultiplier,
      currentActivity: null, // TODO: Wire to current action from backend
      onAdvanceTime: _handleAdvanceTime,
      onTogglePause: () => setState(() => _isPaused = !_isPaused),
      onSpeedChange: (speed) => setState(() => _speedMultiplier = speed),
      onOpenCalendar: () {
        // TODO: Implement calendar overlay
        _consoleController.info('Calendar not yet implemented');
      },
      onRest: () => _handleQuickAction('rest'),
      onWork: () => _handleQuickAction('work'),
      onSocialize: () => _handleQuickAction('socialize'),
    );
  }

  void _handleAdvanceTime() async {
    if (_isPaused) return;
    
    // Advance one tick (1 hour)
    final newState = await _backend.step(1);
    setState(() {
      _gameState = newState;
      _currentHour = (_currentHour + 1) % 24;
    });
  }

  void _handleQuickAction(String action) async {
    _consoleController.info('Quick action: $action');
    // TODO: Wire to backend action system
    // For now, just advance time
    final newState = await _backend.step(1);
    setState(() {
      _gameState = newState;
      _currentHour = (_currentHour + 1) % 24;
    });
  }

  void _handleKeyEvent(KeyEvent event) {
    if (event is! KeyDownEvent) return;
    if (_phase != GamePhase.gameplay) return;
    
    // Don't handle keys if console is open
    if (_showConsole) return;

    final key = event.logicalKey;

    // Keyboard shortcuts
    if (key == LogicalKeyboardKey.slash || key == LogicalKeyboardKey.question) {
      setState(() => _showKeyboardShortcuts = !_showKeyboardShortcuts);
      return;
    }

    if (key == LogicalKeyboardKey.escape) {
      // Close any open overlay
      if (_showKeyboardShortcuts) {
        setState(() => _showKeyboardShortcuts = false);
      } else if (_showDetailedStats) {
        setState(() => _showDetailedStats = false);
      } else if (_showInventory) {
        setState(() => _showInventory = false);
      } else if (_showMemoryJournal) {
        setState(() => _showMemoryJournal = false);
      } else if (_showWorldMap) {
        setState(() => _showWorldMap = false);
      } else if (_showRelationshipNetwork) {
        setState(() => _showRelationshipNetwork = false);
      } else if (_showSettingsOverlay) {
        setState(() => _showSettingsOverlay = false);
      }
      return;
    }

    // Panel shortcuts
    if (key == LogicalKeyboardKey.keyS) {
      setState(() => _showDetailedStats = !_showDetailedStats);
    } else if (key == LogicalKeyboardKey.keyI) {
      setState(() => _showInventory = !_showInventory);
    } else if (key == LogicalKeyboardKey.keyJ) {
      setState(() => _showMemoryJournal = !_showMemoryJournal);
    } else if (key == LogicalKeyboardKey.keyM) {
      setState(() => _showWorldMap = !_showWorldMap);
    } else if (key == LogicalKeyboardKey.keyR) {
      setState(() => _showRelationshipNetwork = !_showRelationshipNetwork);
    }

    // Time controls
    if (key == LogicalKeyboardKey.space) {
      setState(() => _isPaused = !_isPaused);
    } else if (key == LogicalKeyboardKey.arrowRight) {
      _handleAdvanceTime();
    } else if (key == LogicalKeyboardKey.digit1) {
      setState(() => _speedMultiplier = 1);
    } else if (key == LogicalKeyboardKey.digit2) {
      setState(() => _speedMultiplier = 2);
    } else if (key == LogicalKeyboardKey.digit3) {
      setState(() => _speedMultiplier = 4);
    }

    // Dev tools
    if (key == LogicalKeyboardKey.backquote) {
      setState(() => _showConsole = !_showConsole);
    } else if (key == LogicalKeyboardKey.f11) {
      setState(() => _showInspector = !_showInspector);
    }
  }
}
