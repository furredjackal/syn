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

  GameEvent({
    required this.id,
    required this.title,
    required this.description,
    this.artPath,
    required this.choices,
    required this.lifeStage,
    required this.age,
  });
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

  // Stats (0-100 scale)
  int health = 75;
  int mood = 0; // -10 to +10
  int wealth = 50;
  int charisma = 50;
  int intelligence = 50;
  int wisdom = 50;
  int strength = 50;
  int stability = 50;

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
