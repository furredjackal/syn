import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../models/game_state.dart';
import '../theme/theme.dart';
import '../widgets/stat_bar.dart';
import '../widgets/character_info.dart';
import '../widgets/particle_system.dart';

class GameScreen extends StatefulWidget {
  const GameScreen({Key? key}) : super(key: key);

  @override
  State<GameScreen> createState() => _GameScreenState();
}

class _GameScreenState extends State<GameScreen> {
  final List<StatChangeNotificationWidget> _floatingTexts = [];

  @override
  void initState() {
    super.initState();
    // Defer state update to after build phase completes
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _loadNextEvent();
    });
  }

  void _loadNextEvent() {
    final gameState = context.read<GameState>();
    gameState.setCurrentEvent(
      GameEvent(
        id: 'demo_001',
        title: 'A New Beginning',
        description:
            'You wake up on your first day of school. Your parents have prepared your lunch.',
        choices: [
          GameChoice(
              text: 'Eat breakfast',
              statChanges: {'health': 10},
              keyboardShortcut: 1),
          GameChoice(
              text: 'Skip breakfast',
              statChanges: {'health': -5},
              keyboardShortcut: 2),
        ],
        lifeStage: 'Child',
        age: 6,
      ),
    );
  }

  void _handleChoice(int index) {
    final gameState = context.read<GameState>();
    if (gameState.currentEvent != null) {
      final choice = gameState.currentEvent!.choices[index];

      // Show stat change animations
      choice.statChanges.forEach((stat, change) {
        setState(() {
          _floatingTexts.add(
            StatChangeNotificationWidget(
              statName: stat,
              statValue: change.toDouble(),
              position: const Offset(400, 300),
            ),
          );
        });
      });

      gameState.applyChoice(choice);
      Future.delayed(const Duration(milliseconds: 500), () {
        _loadNextEvent();
      });
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF0A0E27),
      body: Consumer<GameState>(
        builder: (context, gameState, _) {
          return Stack(
            children: [
              Container(
                decoration: BoxDecoration(
                  gradient: LinearGradient(
                    begin: Alignment.topLeft,
                    end: Alignment.bottomRight,
                    colors: [
                      const Color(0xFF0A0E27),
                      MoodColors.forMood(gameState.mood.toDouble()),
                      const Color(0xFF2D1B4E),
                    ],
                  ),
                ),
              ),
              SafeArea(
                child: Column(
                  children: [
                    _buildTopBar(context, gameState),
                    const SizedBox(height: 16),
                    Expanded(
                      child: Row(
                        children: [
                          Expanded(
                            child: SingleChildScrollView(
                              child: _buildStatPanel(context, gameState),
                            ),
                          ),
                          const SizedBox(width: 16),
                          Expanded(
                            flex: 2,
                            child: _buildLegacyEventPanel(gameState),
                          ),
                          const SizedBox(width: 16),
                          Expanded(
                            child: SingleChildScrollView(
                              child:
                                  _buildRelationshipPanel(context, gameState),
                            ),
                          ),
                        ],
                      ),
                    ),
                    const SizedBox(height: 16),
                    _buildQuickMenuBar(context, gameState),
                  ],
                ),
              ),
            ],
          );
        },
      ),
    );
  }

  Widget _buildTopBar(BuildContext context, GameState gameState) {
    final lifeStageTheme = LifeStageTheme.fromStage(gameState.lifeStage);
    final moodColor = MoodColors.forMood(gameState.mood.toDouble());

    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      decoration: BoxDecoration(
        color: Colors.black.withOpacity(0.3),
        border: Border(
          bottom: BorderSide(
            color: const Color(0xFF00D9FF).withOpacity(0.3),
            width: 1,
          ),
        ),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            decoration: BoxDecoration(
              border: Border.all(color: lifeStageTheme.primaryColor, width: 2),
              color: lifeStageTheme.primaryColor.withOpacity(0.1),
            ),
            child: Row(
              children: [
                Text(lifeStageTheme.badge,
                    style: const TextStyle(fontSize: 20)),
                const SizedBox(width: 8),
                Text(
                  gameState.lifeStage.toUpperCase(),
                  style: Theme.of(context)
                      .textTheme
                      .labelMedium
                      ?.copyWith(color: lifeStageTheme.primaryColor),
                ),
              ],
            ),
          ),
          Column(
            children: [
              Text('AGE',
                  style: Theme.of(context).textTheme.labelMedium?.copyWith(
                      fontSize: 10, color: Colors.white.withOpacity(0.6))),
              Text(gameState.age.toString(),
                  style: Theme.of(context)
                      .textTheme
                      .titleMedium
                      ?.copyWith(fontSize: 32)),
            ],
          ),
          Column(
            children: [
              Text('MOOD',
                  style: Theme.of(context).textTheme.labelMedium?.copyWith(
                      fontSize: 10, color: Colors.white.withOpacity(0.6))),
              const SizedBox(height: 4),
              Container(
                width: 60,
                height: 60,
                decoration: BoxDecoration(
                  shape: BoxShape.circle,
                  border: Border.all(color: moodColor, width: 2),
                  color: moodColor.withOpacity(0.1),
                  boxShadow: [
                    BoxShadow(
                        color: moodColor.withOpacity(0.5),
                        blurRadius: 15,
                        spreadRadius: 2)
                  ],
                ),
                child: Center(
                  child: Text(gameState.mood.toString(),
                      style: Theme.of(context)
                          .textTheme
                          .titleSmall
                          ?.copyWith(color: moodColor, fontSize: 20)),
                ),
              ),
            ],
          ),
          IconButton(
              onPressed: () => _showQuickMenu(context),
              icon: const Icon(Icons.menu),
              color: const Color(0xFF00D9FF)),
        ],
      ),
    );
  }

  Widget _buildLegacyEventPanel(GameState gameState) {
    final event = gameState.currentEvent;
    if (event == null) {
      return const Center(
        child: CircularProgressIndicator(
          valueColor: AlwaysStoppedAnimation<Color>(Color(0xFF00D9FF)),
        ),
      );
    }

    return Container(
      padding: const EdgeInsets.all(24),
      decoration: BoxDecoration(
        color: Colors.black.withValues(alpha: 0.35),
        borderRadius: BorderRadius.circular(12),
        border: Border.all(color: const Color(0xFF00D9FF).withValues(alpha: 0.8)),
      ),
      child: SingleChildScrollView(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              event.title.toUpperCase(),
              style: Theme.of(context).textTheme.titleLarge?.copyWith(
                    color: const Color(0xFF00D9FF),
                    letterSpacing: 1.5,
                  ),
            ),
            const SizedBox(height: 12),
            Text(
              event.description,
              style: Theme.of(context)
                  .textTheme
                  .bodyMedium
                  ?.copyWith(color: Colors.white.withValues(alpha: 0.85)),
            ),
            const SizedBox(height: 24),
            ...List.generate(event.choices.length, (index) {
              final choice = event.choices[index];
              return Padding(
                padding: const EdgeInsets.symmetric(vertical: 8),
                child: OutlinedButton(
                  style: OutlinedButton.styleFrom(
                    side: const BorderSide(color: Color(0xFF00D9FF)),
                    padding: const EdgeInsets.symmetric(
                      horizontal: 16,
                      vertical: 12,
                    ),
                  ),
                  onPressed: () => _handleChoice(index),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        choice.text.toUpperCase(),
                        style: Theme.of(context)
                            .textTheme
                            .titleSmall
                            ?.copyWith(color: Colors.white),
                      ),
                      if (choice.statChanges.isNotEmpty) ...[
                        const SizedBox(height: 6),
                        Wrap(
                          spacing: 12,
                          runSpacing: 4,
                          children: choice.statChanges.entries
                              .map(
                                (entry) => Text(
                                  '${entry.value >= 0 ? '+' : ''}${entry.value} ${entry.key}',
                                  style: Theme.of(context)
                                      .textTheme
                                      .labelMedium
                                      ?.copyWith(
                                        color: entry.value >= 0
                                            ? Colors.greenAccent
                                            : Colors.redAccent,
                                      ),
                                ),
                              )
                              .toList(),
                        ),
                      ],
                    ],
                  ),
                ),
              );
            }),
          ],
        ),
      ),
    );
  }

  Widget _buildStatPanel(BuildContext context, GameState gameState) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        border: Border.all(
            color: const Color(0xFF00D9FF).withOpacity(0.3), width: 1),
        color: Colors.black.withOpacity(0.2),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('STATS',
              style: Theme.of(context)
                  .textTheme
                  .titleSmall
                  ?.copyWith(color: const Color(0xFF00D9FF))),
          const SizedBox(height: 16),
          StatBar(label: 'Health', value: gameState.health),
          const SizedBox(height: 12),
          StatBar(label: 'Wealth', value: gameState.wealth),
          const SizedBox(height: 12),
          StatBar(label: 'Charisma', value: gameState.charisma),
          const SizedBox(height: 12),
          StatBar(label: 'Intelligence', value: gameState.intelligence),
          const SizedBox(height: 12),
          StatBar(label: 'Wisdom', value: gameState.wisdom),
          const SizedBox(height: 12),
          StatBar(label: 'Strength', value: gameState.strength),
          const SizedBox(height: 12),
          StatBar(label: 'Stability', value: gameState.stability),
        ],
      ),
    );
  }

  Widget _buildRelationshipPanel(BuildContext context, GameState gameState) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        border: Border.all(
            color: const Color(0xFF00D9FF).withOpacity(0.3), width: 1),
        color: Colors.black.withOpacity(0.2),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('RELATIONSHIPS',
              style: Theme.of(context)
                  .textTheme
                  .titleSmall
                  ?.copyWith(color: const Color(0xFF00D9FF))),
          const SizedBox(height: 16),
          if (gameState.relationships.isEmpty)
            Text('No relationships yet.',
                style: Theme.of(context)
                    .textTheme
                    .bodySmall
                    ?.copyWith(color: Colors.white.withOpacity(0.5)))
          else
            ...gameState.relationships
                .map((rel) => Padding(
                    padding: const EdgeInsets.only(bottom: 16),
                    child: CharacterInfo(relationship: rel)))
                .toList(),
        ],
      ),
    );
  }

  Widget _buildQuickMenuBar(BuildContext context, GameState gameState) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
      decoration: BoxDecoration(
        color: Colors.black.withOpacity(0.3),
        border: Border(
            top: BorderSide(
                color: const Color(0xFF00D9FF).withOpacity(0.3), width: 1)),
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceEvenly,
        children: [
          _QuickMenuButton(
              label: 'MEMORY (M)',
              onPressed: () => Navigator.pushNamed(context, '/journal')),
          _QuickMenuButton(
              label: 'SAVE (Ctrl+S)', onPressed: () => _handleSave(context)),
          _QuickMenuButton(
              label: 'SETTINGS (ESC)',
              onPressed: () => Navigator.pushNamed(context, '/settings')),
          _QuickMenuButton(
              label: 'MENU',
              onPressed: () =>
                  Navigator.pushReplacementNamed(context, '/menu')),
        ],
      ),
    );
  }

  void _showQuickMenu(BuildContext context) {
    showModalBottomSheet(
        context: context,
        backgroundColor: const Color(0xFF15192E),
        builder: (context) => SafeArea(
                child: Wrap(children: [
              ListTile(
                  title: const Text('Memory Journal'),
                  onTap: () {
                    Navigator.pop(context);
                    Navigator.pushNamed(context, '/journal');
                  }),
              ListTile(
                  title: const Text('Save Game'),
                  onTap: () {
                    Navigator.pop(context);
                    _handleSave(context);
                  }),
              ListTile(
                  title: const Text('Settings'),
                  onTap: () {
                    Navigator.pop(context);
                    Navigator.pushNamed(context, '/settings');
                  }),
              ListTile(
                  title: const Text('Return to Menu'),
                  onTap: () {
                    Navigator.pop(context);
                    Navigator.pushReplacementNamed(context, '/menu');
                  }),
            ])));
  }

  void _handleSave(BuildContext context) {
    ScaffoldMessenger.of(context).showSnackBar(const SnackBar(
        content: Text('Game saved successfully'),
        duration: Duration(seconds: 1)));
  }
}

class _QuickMenuButton extends StatefulWidget {
  final String label;
  final VoidCallback onPressed;

  const _QuickMenuButton(
      {required this.label, required this.onPressed, Key? key})
      : super(key: key);

  @override
  State<_QuickMenuButton> createState() => _QuickMenuButtonState();
}

class _QuickMenuButtonState extends State<_QuickMenuButton> {
  bool _isHovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _isHovered = true),
      onExit: (_) => setState(() => _isHovered = false),
      child: GestureDetector(
        onTap: widget.onPressed,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 200),
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          decoration: BoxDecoration(
            border: Border.all(
                color: _isHovered
                    ? const Color(0xFF00D9FF)
                    : const Color(0xFF00D9FF).withOpacity(0.3),
                width: 1),
            color: _isHovered
                ? const Color(0xFF00D9FF).withOpacity(0.1)
                : Colors.transparent,
          ),
          child: Text(
            widget.label,
            style: Theme.of(context).textTheme.labelMedium?.copyWith(
                  color: _isHovered
                      ? const Color(0xFF00D9FF)
                      : Colors.white.withOpacity(0.7),
                ),
          ),
        ),
      ),
    );
  }
}
