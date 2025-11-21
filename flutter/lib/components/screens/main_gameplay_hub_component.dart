import 'package:flame/components.dart';
import 'package:flutter/material.dart';

import '../../models/game_state.dart';
import '../../syn_game.dart';
import '../ui/buttons/text_button_component.dart';
import '../ui/cards/event_card_component.dart';
import '../ui/cards/npc_card_component.dart';
import '../ui/display/stat_bar_component.dart';
import '../ui/display/stat_ring_component.dart';
import '../ui/system/background_layer_component.dart';
import '../ui/system/notification_queue_component.dart';

/// Main gameplay hub: a layout compositor that positions existing UI components.
class MainGameplayHubComponent extends PositionComponent
    with HasGameReference<SynGame> {
  late final BackgroundLayerComponent _background;
  late final TopBarComponent _topBar;
  late final StatPanelComponent _statPanel;
  late final RelationshipPanelComponent _relationshipPanel;
  late final QuickMenuBarComponent _quickMenu;
  late final EventCardComponent _eventCard;
  late final NotificationQueueComponent _notificationQueue;

  GameEvent get _activeEvent => game.gameState.currentEvent ?? _placeholderEvent;

  GameEvent _buildEventPlaceholder(GameState state) {
    final tags = ['NIGHT', 'RADIO', 'FOG'];
    return GameEvent(
      id: 'signal-in-the-fog',
      title: 'Signal in the Fog',
      description:
          'A flickering broadcast spills through the neon mist. Trace it, boost it, or jam it — each move shapes the mood of the city.',
      lifeStage: state.lifeStage.toUpperCase(),
      age: state.age,
      tags: tags,
      deltas: const {'health': 2, 'stability': -1},
      choices: [
        GameChoice(
          text: 'TRACE THE SOURCE',
          statChanges: const {'health': -2, 'stability': 2},
          keyboardShortcut: 1,
        ),
        GameChoice(
          text: 'BOOST THE SIGNAL',
          statChanges: const {'wealth': 1, 'charisma': 1, 'stability': -1},
          keyboardShortcut: 2,
        ),
        GameChoice(
          text: 'JAM THE FREQUENCY',
          statChanges: const {'stability': 2, 'intelligence': -1},
          keyboardShortcut: 3,
        ),
      ],
    );
  }

  GameEvent get _placeholderEvent => _buildEventPlaceholder(game.gameState);

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    size = game.size.clone();

    _background = BackgroundLayerComponent()
      ..position = Vector2.zero()
      ..size = size.clone()
      ..priority = 0;

    _topBar = TopBarComponent(
      gameState: game.gameState,
      onStats: game.showDetailedStats,
      onSettings: game.showSettings,
      onSave: game.showSaveLoad,
      onPause: game.togglePauseOverlay,
    )..priority = 10;

    _statPanel =
        StatPanelComponent(gameState: game.gameState)..priority = 10;

    _relationshipPanel = RelationshipPanelComponent(
      relationships: game.gameState.relationships,
      onOpenNetwork: game.showRelationshipNetwork,
    )..priority = 10;

    _quickMenu = QuickMenuBarComponent(
      onMemory: game.showMemoryJournal,
      onMap: game.showWorldMap,
      onPossessions: game.showPossessions,
      onSaveLoad: game.showSaveLoad,
      onSettings: game.showSettings,
      onPause: game.togglePauseOverlay,
    )..priority = 12;

    _eventCard = EventCardComponent(
      event: _activeEvent,
      onChoice: _handleChoice,
    )..priority = 20;

    _notificationQueue = NotificationQueueComponent(
      initialMessages: const [
        '+10 TRUST — Kaz backs your move',
        'HEALTH +2 — Efficient recovery',
        'STABILITY -1 — Static lingers',
      ],
    )..priority = 30;

    _applyLayout();

    addAll([
      _background,
      _topBar,
      _statPanel,
      _relationshipPanel,
      _quickMenu,
      _eventCard,
      _notificationQueue,
    ]);
  }

  @override
  void onGameResize(Vector2 newSize) {
    super.onGameResize(newSize);
    size = newSize;
    _applyLayout();
  }

  void _applyLayout() {
    final w = size.x;
    final h = size.y;

    _background.size = size.clone();

    _topBar
      ..position = Vector2(0.10 * w, 0.05 * h)
      ..size = Vector2(0.80 * w, 0.10 * h);

    _statPanel
      ..position = Vector2(0.08 * w, 0.20 * h)
      ..size = Vector2(0.18 * w, 0.52 * h);

    _relationshipPanel
      ..position = Vector2(0.74 * w, 0.20 * h)
      ..size = Vector2(0.18 * w, 0.52 * h);

    _eventCard
      ..position = Vector2(0.22 * w, 0.17 * h)
      ..size = Vector2(0.56 * w, 0.58 * h);

    _quickMenu
      ..position = Vector2(0.18 * w, 0.80 * h)
      ..size = Vector2(0.64 * w, 0.10 * h);

    _notificationQueue.position = Vector2(0.70 * w, 0.06 * h);
  }

  void _handleChoice(int index) {
    final choice = _activeEvent.choices[index];
    choice.statChanges.forEach(game.gameState.updateStat);
    _notificationQueue.addMessage(
      'CHOICE ${index + 1}: ${choice.text.toUpperCase()}',
    );
  }
}

class GameplayHub extends MainGameplayHubComponent {}

class TopBarComponent extends PositionComponent
    with HasGameReference<SynGame> {
  TopBarComponent({
    required this.gameState,
    required this.onStats,
    required this.onSettings,
    required this.onSave,
    required this.onPause,
    super.position,
    super.size,
  });

  final GameState gameState;
  final VoidCallback onStats;
  final VoidCallback onSettings;
  final VoidCallback onSave;
  final VoidCallback onPause;

  @override
  Future<void> onLoad() async {
    await super.onLoad();

    final stage = TextComponent(
      text: gameState.lifeStage.toUpperCase(),
      textRenderer: TextPaint(
        style: const TextStyle(
          color: Colors.white,
          fontWeight: FontWeight.w800,
          fontSize: 16,
          letterSpacing: 2.0,
        ),
      ),
      position: Vector2(20, 10),
    );
    add(stage);

    final age = TextComponent(
      text: 'AGE ${gameState.age}',
      textRenderer: TextPaint(
        style: const TextStyle(
          color: Color(0xFFB8C2D6),
          fontWeight: FontWeight.w700,
          fontSize: 13,
          letterSpacing: 1.3,
        ),
      ),
      position: Vector2(20, size.y - 26),
    );
    add(age);

    final moodRing = StatRingComponent(
      label: 'MOOD',
      value: (gameState.mood + 10).toDouble(),
      maxValue: 20,
      color: const Color(0xFF00E6FF),
      position: Vector2(size.x - 210, (size.y - 80) / 2),
      radius: 42,
    );
    add(moodRing);

    final actions = [
      ('STATS', onStats),
      ('SETTINGS', onSettings),
      ('SAVE', onSave),
      ('PAUSE', onPause),
    ];
    final buttonWidth = 100.0;
    final startX = size.x - (actions.length * buttonWidth) - 18;
    for (var i = 0; i < actions.length; i++) {
      final entry = actions[i];
      final button = TextButtonComponent(
        label: entry.$1,
        onTap: entry.$2,
        size: Vector2(buttonWidth - 12, size.y - 16),
        position: Vector2(startX + i * buttonWidth, 8),
      );
      add(button);
    }
  }

  @override
  void render(Canvas canvas) {
    final path = Path()
      ..moveTo(18, 0)
      ..lineTo(size.x - 22, 0)
      ..lineTo(size.x, size.y * 0.6)
      ..lineTo(size.x - 18, size.y)
      ..lineTo(18, size.y)
      ..lineTo(0, size.y * 0.45)
      ..close();

    canvas.drawShadow(path, const Color(0xAA000000), 10, false);
    canvas.drawPath(
      path,
      Paint()
        ..shader = const LinearGradient(
          colors: [Color(0xCC0D111C), Color(0xEE0C131F)],
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
        ).createShader(Rect.fromLTWH(0, 0, size.x, size.y)),
    );
    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 3
        ..color = const Color(0xFF00D9FF),
    );

    final accent = Path()
      ..moveTo(size.x * 0.52, -2)
      ..lineTo(size.x * 0.48, size.y + 6)
      ..lineTo(size.x * 0.62, size.y + 6)
      ..close();
    canvas.drawPath(
      accent,
      Paint()
        ..shader = const LinearGradient(
          colors: [Color(0x3300D9FF), Color(0x5500FFC8)],
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
        ).createShader(Rect.fromLTWH(0, -2, size.x, size.y + 8)),
    );
  }
}

class StatPanelComponent extends PositionComponent
    with HasGameReference<SynGame> {
  StatPanelComponent({required this.gameState, super.position, super.size});

  final GameState gameState;

  @override
  Future<void> onLoad() async {
    await super.onLoad();

    final stats = _buildStats();
    var y = 22.0;
    for (final stat in stats) {
      final row = _StatRow(
        label: stat.$1,
        value: stat.$2,
        rawValue: stat.$3,
      )
        ..position = Vector2(18, y)
        ..size = Vector2(size.x - 36, 48);
      add(row);
      y += 60;
    }
  }

  List<(String, double, int)> _buildStats() {
    return [
      ('HEALTH', gameState.health / 100, gameState.health),
      ('WEALTH', gameState.wealth / 100, gameState.wealth),
      ('CHARISMA', gameState.charisma / 100, gameState.charisma),
      ('INTELLECT', gameState.intelligence / 100, gameState.intelligence),
      ('STABILITY', gameState.stability / 100, gameState.stability),
      ('CREATIVITY', gameState.wisdom / 100, gameState.wisdom),
    ];
  }

  @override
  void render(Canvas canvas) {
    final path = Path()
      ..moveTo(16, 0)
      ..lineTo(size.x - 10, 6)
      ..lineTo(size.x, size.y * 0.25)
      ..lineTo(size.x, size.y - 10)
      ..lineTo(size.x - 16, size.y)
      ..lineTo(10, size.y - 6)
      ..lineTo(0, size.y * 0.15)
      ..close();

    canvas.drawShadow(path, const Color(0x99000000), 14, false);
    canvas.drawPath(
      path,
      Paint()
        ..shader = const LinearGradient(
          colors: [Color(0xEE0B101C), Color(0xDD0E1728)],
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
        ).createShader(Rect.fromLTWH(0, 0, size.x, size.y)),
    );
    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2.6
        ..color = const Color(0xFF00D9FF),
    );

    final slash = Path()
      ..moveTo(size.x * 0.72, -8)
      ..lineTo(size.x * 0.82, size.y * 0.4)
      ..lineTo(size.x * 0.7, size.y * 0.4 + 12)
      ..close();
    canvas.drawPath(
      slash,
      Paint()..color = const Color(0x2200D9FF),
    );
  }
}

class _StatRow extends PositionComponent {
  _StatRow({
    required this.label,
    required this.value,
    required this.rawValue,
  });

  final String label;
  final double value;
  final int rawValue;

  @override
  Future<void> onLoad() async {
    await super.onLoad();

    final labelText = TextComponent(
      text: label,
      textRenderer: TextPaint(
        style: const TextStyle(
          color: Colors.white,
          fontWeight: FontWeight.w800,
          letterSpacing: 1.4,
          fontSize: 12,
        ),
      ),
      position: Vector2.zero(),
    );
    add(labelText);

    final bar = StatBarComponent(
      value: value,
      position: Vector2(0, 18),
      size: Vector2(size.x, 10),
    );
    add(bar);

    final numeric = TextComponent(
      text: rawValue.toString(),
      textRenderer: TextPaint(
        style: const TextStyle(
          color: Color(0xFFB8C2D6),
          fontWeight: FontWeight.w600,
          fontSize: 12,
        ),
      ),
      position: Vector2(0, 32),
    );
    add(numeric);
  }
}

class RelationshipPanelComponent extends PositionComponent
    with HasGameReference<SynGame> {
  RelationshipPanelComponent({
    required this.relationships,
    required this.onOpenNetwork,
    super.position,
    super.size,
  });

  final List<RelationshipData> relationships;
  final VoidCallback onOpenNetwork;

  @override
  Future<void> onLoad() async {
    await super.onLoad();

    final entries = relationships.isNotEmpty
        ? relationships
        : _placeholderRelationships();

    var y = 22.0;
    for (final rel in entries.take(4)) {
      final card = NPCCardComponent(
        relationship: rel,
        onTap: onOpenNetwork,
        position: Vector2(14, y),
        size: Vector2(size.x - 28, 78),
      );
      add(card);
      y += 90;
    }
  }

  List<RelationshipData> _placeholderRelationships() {
    return [
      RelationshipData(
        npcId: 'kaz',
        npcName: 'Kaz',
        affection: 4,
        trust: 6,
        attraction: 3,
        familiarity: 6,
        resentment: 2,
        state: 'Ally',
      ),
      RelationshipData(
        npcId: 'ila',
        npcName: 'Ila',
        affection: 2,
        trust: 5,
        attraction: 1,
        familiarity: 4,
        resentment: 3,
        state: 'Confidant',
      ),
      RelationshipData(
        npcId: 'fixer',
        npcName: 'Fixer',
        affection: 1,
        trust: 4,
        attraction: 0,
        familiarity: 5,
        resentment: 5,
        state: 'Contact',
      ),
    ];
  }

  @override
  void render(Canvas canvas) {
    final path = Path()
      ..moveTo(12, 0)
      ..lineTo(size.x - 12, 6)
      ..lineTo(size.x, size.y * 0.18)
      ..lineTo(size.x - 8, size.y)
      ..lineTo(14, size.y - 6)
      ..lineTo(0, size.y * 0.2)
      ..close();

    canvas.drawShadow(path, const Color(0x99000000), 14, false);
    canvas.drawPath(
      path,
      Paint()
        ..shader = const LinearGradient(
          colors: [Color(0xEE0C111C), Color(0xDD101A2C)],
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
        ).createShader(Rect.fromLTWH(0, 0, size.x, size.y)),
    );
    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2.6
        ..color = const Color(0xFFFF3C8F),
    );

    final slash = Path()
      ..moveTo(size.x * 0.18, -8)
      ..lineTo(size.x * 0.32, size.y * 0.45)
      ..lineTo(size.x * 0.2, size.y * 0.45 + 14)
      ..close();
    canvas.drawPath(
      slash,
      Paint()..color = const Color(0x22FF3C8F),
    );
  }
}

class QuickMenuBarComponent extends PositionComponent {
  QuickMenuBarComponent({
    required this.onMemory,
    required this.onMap,
    required this.onPossessions,
    required this.onSaveLoad,
    required this.onSettings,
    required this.onPause,
    super.position,
    super.size,
  });

  final VoidCallback onMemory;
  final VoidCallback onMap;
  final VoidCallback onPossessions;
  final VoidCallback onSaveLoad;
  final VoidCallback onSettings;
  final VoidCallback onPause;

  @override
  Future<void> onLoad() async {
    await super.onLoad();

    final entries = [
      ('MEMORY', onMemory),
      ('MAP', onMap),
      ('POSSESSIONS', onPossessions),
      ('SAVE/LOAD', onSaveLoad),
      ('SETTINGS', onSettings),
      ('PAUSE', onPause),
    ];

    final buttonWidth = size.x / entries.length;
    for (var i = 0; i < entries.length; i++) {
      final button = TextButtonComponent(
        label: entries[i].$1,
        onTap: entries[i].$2,
        size: Vector2(buttonWidth - 18, size.y - 16),
        position: Vector2(i * buttonWidth + 9, 8),
      );
      add(button);
    }
  }

  @override
  void render(Canvas canvas) {
    final path = Path()
      ..moveTo(18, 0)
      ..lineTo(size.x - 18, 0)
      ..lineTo(size.x, size.y * 0.65)
      ..lineTo(size.x - 18, size.y)
      ..lineTo(18, size.y)
      ..lineTo(0, size.y * 0.35)
      ..close();

    canvas.drawShadow(path, const Color(0x99000000), 12, false);
    canvas.drawPath(
      path,
      Paint()
        ..shader = const LinearGradient(
          colors: [Color(0xEE0D131F), Color(0xDD0F1A2C)],
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
        ).createShader(Rect.fromLTWH(0, 0, size.x, size.y)),
    );
    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 3
        ..color = const Color(0xFF00D9FF),
    );
  }
}
