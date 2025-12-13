import 'package:syn/bridge/bridge_generated/lib.dart';

/// Game event choice
class GameChoice {
  final String text;
  final Map<String, int> statChanges;
  final int keyboardShortcut;

  GameChoice({
    required this.text,
    required this.statChanges,
    required this.keyboardShortcut,
  });
}

/// Game event/storylet
class GameEvent {
  final String id;
  final String title;
  final String description;
  final String? artPath;
  final List<GameChoice> choices;
  final String lifeStage;
  final int age;
  final List<String> tags;
  final Map<String, int> deltas;

  GameEvent({
    required this.id,
    required this.title,
    required this.description,
    this.artPath,
    required this.choices,
    required this.lifeStage,
    required this.age,
    this.tags = const [],
    this.deltas = const {},
  });

  /// Create GameEvent from ApiDirectorEventView.
  factory GameEvent.fromApi(ApiDirectorEventView api) {
    return GameEvent(
      id: api.storyletId,
      title: api.title,
      description: api.title, // Use title as description for now
      choices: api.choices
          .asMap()
          .entries
          .map((entry) => GameChoice(
                text: entry.value.label,
                statChanges: {},
                keyboardShortcut: entry.key + 1,
              ))
          .toList(),
      lifeStage: '', // Not available in API
      age: 0, // Not available in API
    );
  }
}

/// Relationship data
class RelationshipData {
  final String npcId;
  final String npcName;
  final double affection; // -10 to +10
  final double trust; // -10 to +10
  final double attraction; // -10 to +10
  final double familiarity; // 0 to +10
  final double resentment; // -10 to +10
  final String state; // Stranger, Friend, CloseFriend, etc.

  RelationshipData({
    required this.npcId,
    required this.npcName,
    required this.affection,
    required this.trust,
    required this.attraction,
    required this.familiarity,
    required this.resentment,
    required this.state,
  });

  /// Create from ApiSimpleRelationship (simplified version).
  factory RelationshipData.fromApiSimple(ApiSimpleRelationship api) {
    final strength = api.strength; // -1.0 to 1.0
    return RelationshipData(
      npcId: api.npcId.toString(),
      npcName: api.name,
      affection: strength * 10, // Scale to -10..10
      trust: strength * 10,
      attraction: 0.0,
      familiarity: strength.abs() * 10,
      resentment: strength < 0 ? -strength * 10 : 0.0,
      state: _deriveState(strength),
    );
  }

  /// Create from ApiRelationship (full 5-axis version).
  factory RelationshipData.fromApiFull(ApiRelationship api) {
    return RelationshipData(
      npcId: api.actorId.toString(),
      npcName: 'NPC_${api.targetId}', // TODO: Get actual name from elsewhere
      affection: api.affection,
      trust: api.trust,
      attraction: api.attraction,
      familiarity: api.familiarity,
      resentment: api.resentment,
      state: api.roleLabel,
    );
  }

  /// Derive relationship state label from simplified strength value.
  static String _deriveState(double strength) {
    if (strength < -0.7) return 'Enemy';
    if (strength < -0.3) return 'Rival';
    if (strength < 0.3) return 'Stranger';
    if (strength < 0.7) return 'Friend';
    return 'CloseFriend';
  }
}

/// Memory journal entry
class MemoryEntry {
  final String id;
  final String eventTitle;
  final String description;
  final DateTime timestamp;
  final double emotionalIntensity; // -1.0 to +1.0
  final List<String> tags;

  MemoryEntry({
    required this.id,
    required this.eventTitle,
    required this.description,
    required this.timestamp,
    required this.emotionalIntensity,
    required this.tags,
  });
}

/// Main game state
class GameState {
  // Player info
  String playerName = '';
  String archetype = 'STORYTELLER';
  String difficulty = 'BALANCED';
  int age = 6;
  String lifeStage = 'Child';
  int year = 0;

  // Stats (0-100 scale) - mapped from Rust's 11 StatKinds
  int health = 75;
  int mood = 0; // -10 to +10 (MoodBand)
  int wealth = 50;
  int charisma = 50;
  int intelligence = 50;
  int wisdom = 50;
  int strength = 50;
  int stability = 50;
  int appearance = 50;
  int reputation = 50;
  int curiosity = 50;
  int energy = 75;
  int libido = 50;

  // Karma
  int karma = 0;

  // Narrative heat
  double narrativeHeat = 50.0; // 0 to 200

  // Content mode
  bool sfwMode = true;

  // Relationships
  List<RelationshipData> relationships = [];

  // Memory journal
  List<MemoryEntry> memories = [];

  // Current event
  GameEvent? currentEvent;

  // Settings
  bool soundEnabled = true;
  bool musicEnabled = true;
  double masterVolume = 0.8;
  bool colorBlindMode = false;
  bool reducedMotion = false;
  double fontSize = 1.0;

  // Save data
  bool hasActiveSave = false;

  GameState() {
    _initializeDefaults();
  }

  /// Create GameState from ApiSimpleGameState (used by most operations).
  factory GameState.fromApiSimple(ApiSimpleGameState api) {
    final state = GameState();
    
    // Time and age
    state.age = api.playerAge;
    state.lifeStage = api.lifeStage;
    state.year = api.currentDay ~/ 365;
    
    // Debug assertion: ensure age is correctly set for new games
    assert(state.age >= 0, 'Player age should be non-negative');
    if (api.currentDay == 0 || api.currentDay == 1) {
      // New game should have player starting at age 6
      assert(state.age >= 6, 'New game should start with age >= 6, got ${state.age}');
    }
    
    // Parse stats from the stats list (all 11 Rust StatKinds)
    for (final stat in api.stats.stats) {
      final value = stat.value.toInt();
      switch (stat.kind.toLowerCase()) {
        case 'health':
          state.health = value;
          break;
        case 'mood':
          state.mood = value;
          break;
        case 'wealth':
          state.wealth = value;
          break;
        case 'charisma':
          state.charisma = value;
          break;
        case 'intelligence':
          state.intelligence = value;
          break;
        case 'wisdom':
          state.wisdom = value;
          break;
        case 'strength':
          state.strength = value;
          break;
        case 'stability':
          state.stability = value;
          break;
        case 'appearance':
          state.appearance = value;
          break;
        case 'reputation':
          state.reputation = value;
          break;
        case 'curiosity':
          state.curiosity = value;
          break;
        case 'energy':
          state.energy = value;
          break;
        case 'libido':
          state.libido = value;
          break;
      }
    }
    
    // Karma
    state.karma = api.karma.toInt();
    
    // Current event
    if (api.currentEvent != null) {
      state.currentEvent = GameEvent.fromApi(api.currentEvent!);
    }
    
    // Relationships
    state.relationships = api.relationships
        .map((rel) => RelationshipData.fromApiSimple(rel))
        .toList();
    
    // Memories
    state.memories = api.recentMemories
        .asMap()
        .entries
        .map((entry) => MemoryEntry(
              id: 'mem_${entry.key}',
              eventTitle: 'Memory ${entry.key + 1}',
              description: entry.value,
              timestamp: DateTime.now().subtract(Duration(days: entry.key)),
              emotionalIntensity: 0.5,
              tags: [],
            ))
        .toList();
    
    return state;
  }

  /// Create GameState from ApiGameStateSnapshot (comprehensive version).
  factory GameState.fromApiFull(ApiGameStateSnapshot api) {
    final state = GameState();
    
    // Time and age
    state.age = api.playerAgeYears;
    state.lifeStage = api.lifeStage;
    state.year = api.currentTick.toInt() ~/ (24 * 365);
    
    // Parse stats from the stats list
    for (final stat in api.stats.stats) {
      final value = stat.value.toInt();
      switch (stat.kind.toLowerCase()) {
        case 'health':
          state.health = value;
          break;
        case 'mood':
          state.mood = value;
          break;
        case 'wealth':
          state.wealth = value;
          break;
        case 'charisma':
          state.charisma = value;
          break;
        case 'intelligence':
          state.intelligence = value;
          break;
        case 'wisdom':
          state.wisdom = value;
          break;
        case 'strength':
          state.strength = value;
          break;
        case 'stability':
          state.stability = value;
          break;
      }
    }
    
    // Karma
    state.karma = api.karma.toInt();
    
    // Narrative heat
    state.narrativeHeat = api.narrativeHeat;
    
    // Current event
    if (api.currentEvent != null) {
      state.currentEvent = GameEvent.fromApi(api.currentEvent!);
    }
    
    // Relationships - use full relationship data
    state.relationships = api.relationships.relationships
        .map((rel) => RelationshipData.fromApiFull(rel))
        .toList();
    
    return state;
  }

  void _initializeDefaults() {
    // Initialize with default values
    health = 75;
    mood = 0;
    wealth = 50;
  }

  // Update player name
  void setPlayerName(String name) {
    playerName = name;
  }

  // Update archetype
  void setArchetype(String newArchetype) {
    archetype = newArchetype;
  }

  // Update difficulty
  void setDifficulty(String newDifficulty) {
    difficulty = newDifficulty;
  }

  // Update age and life stage
  void advanceAge(int years) {
    age += years;
    _updateLifeStage();
  }

  void _updateLifeStage() {
    if (age < 12) {
      lifeStage = 'Child';
    } else if (age < 20) {
      lifeStage = 'Teen';
    } else if (age < 65) {
      lifeStage = 'Adult';
    } else {
      lifeStage = 'Elder';
    }
  }

  // Update mood
  void setMood(int newMood) {
    mood = newMood.clamp(-10, 10);
  }

  // Update narrative heat
  void addNarrativeHeat(double amount) {
    narrativeHeat = (narrativeHeat + amount).clamp(0, 200);
  }

  void setNarrativeHeat(double value) {
    narrativeHeat = value.clamp(0, 200);
  }

  // Update stats
  void updateStat(String statName, int change) {
    switch (statName.toLowerCase()) {
      case 'health':
        health = (health + change).clamp(0, 100);
        break;
      case 'wealth':
        wealth = (wealth + change).clamp(0, 100);
        break;
      case 'charisma':
        charisma = (charisma + change).clamp(0, 100);
        break;
      case 'intelligence':
        intelligence = (intelligence + change).clamp(0, 100);
        break;
      case 'wisdom':
        wisdom = (wisdom + change).clamp(0, 100);
        break;
      case 'strength':
        strength = (strength + change).clamp(0, 100);
        break;
      case 'stability':
        stability = (stability + change).clamp(0, 100);
        break;
    }
  }

  // Update karma
  void addKarma(int amount) {
    karma += amount;
  }

  // Set current event
  void setCurrentEvent(GameEvent? event) {
    currentEvent = event;
  }

  // Apply choice effects
  void applyChoice(GameChoice choice) {
    choice.statChanges.forEach((stat, change) {
      updateStat(stat, change);
    });
  }

  // Add memory
  void addMemory(MemoryEntry memory) {
    memories.add(memory);
  }

  // Update relationship
  void updateRelationship(RelationshipData updated) {
    final index = relationships.indexWhere((r) => r.npcId == updated.npcId);
    if (index >= 0) {
      relationships[index] = updated;
    } else {
      relationships.add(updated);
    }
  }

  // Settings
  void toggleSound() {
    soundEnabled = !soundEnabled;
  }

  void toggleMusic() {
    musicEnabled = !musicEnabled;
  }

  void setMasterVolume(double volume) {
    masterVolume = volume.clamp(0.0, 1.0);
  }

  void toggleColorBlindMode() {
    colorBlindMode = !colorBlindMode;
  }

  void toggleReducedMotion() {
    reducedMotion = !reducedMotion;
  }

  void setFontSize(double scale) {
    fontSize = scale;
  }

  // Save/Load
  void markSaveExists() {
    hasActiveSave = true;
  }
}
