# Rust Bridge API Reference

This directory contains the generated Flutter bridge to the Rust backend.

## Quick Start

```dart
import 'package:syn/bridge/bridge_generated.dart';

// Initialize world
await initWorld(seed: 12345);

// Game loop
while (running) {
  // Advance 1 tick (1 hour)
  await stepWorld(ticks: 1);
  
  // Get game state
  final state = await getGameStateSnapshot();
  
  // Update UI
  updateStats(state.stats);
  updateMood(state.moodBand);
  
  // Check for events
  if (state.currentEvent != null) {
    displayEvent(state.currentEvent!);
  }
}
```

## Core Functions

### World Management

```dart
// Initialize new game
await initWorld(seed: 12345);

// Load saved game (TODO: implement save system)
await loadWorld(seed: 12345);

// Advance time by N hours
await stepWorld(ticks: 24); // 1 day
```

### Game State

```dart
// Get all game state in one call
final state = await getGameStateSnapshot();

// Access state data
final age = state.playerAgeYears;
final stage = state.lifeStage;
final heat = state.narrativeHeat;
final karma = state.karma;
```

### Events & Storylets

```dart
// Get current event
final event = await getCurrentStorylet();
if (event != null) {
  print('Event: ${event.title}');
  
  // Get choices
  final choices = await getAvailableChoices();
  for (final choice in choices) {
    print('Choice: ${choice.label}');
  }
  
  // Make a choice
  final nextEvent = await apiChooseOption(
    storyletId: event.storyletId,
    choiceId: choices[0].id,
    ticksToAdvance: 24,
  );
}
```

### Player Stats

```dart
// Get all stats
final stats = await getPlayerStats();
for (final stat in stats.stats) {
  print('${stat.kind}: ${stat.value}');
}

// Quick accessors
final mood = await getPlayerMood();
final karma = await getPlayerKarma();
```

### Relationships

```dart
// Get relationship network
final network = await getRelationshipNetwork();
for (final rel in network.relationships) {
  print('Relationship with NPC ${rel.targetId}:');
  print('  Role: ${rel.roleLabel}');
  print('  Affection: ${rel.affectionBand}');
  print('  Trust: ${rel.trustBand}');
}
```

### Memories

```dart
// Get memory journal
final memories = await getMemoryJournal();
for (final memory in memories) {
  print('Memory: ${memory.eventId}');
  print('  Intensity: ${memory.emotionalIntensity}');
  print('  Tick: ${memory.simTick}');
}
```

### Digital Legacy (End-of-Life)

```dart
// Get life stage summary
final legacy = await getLifeStageSummary();
if (legacy.hasImprint) {
  final imprint = legacy.imprint!;
  print('Digital Imprint #${imprint.id}');
  print('Created at age ${imprint.createdAtAgeYears}');
  
  // Display legacy vector
  final lv = imprint.legacyVector;
  print('Compassion: ${lv.compassionVsCruelty}');
  print('Ambition: ${lv.ambitionVsComfort}');
  
  // Display relationship roles
  for (final role in imprint.relationshipRoles) {
    print('NPC ${role.targetId}: ${role.role}');
  }
}
```

## Component Integration

### RelationshipNetworkComponent

```dart
class RelationshipNetworkComponent extends StatefulWidget {
  @override
  Widget build(BuildContext context) {
    return FutureBuilder(
      future: getRelationshipNetwork(),
      builder: (context, snapshot) {
        if (!snapshot.hasData) return LoadingSpinner();
        
        final network = snapshot.data!;
        return RelationshipGraph(
          relationships: network.relationships,
        );
      },
    );
  }
}
```

### MemoryJournalComponent

```dart
class MemoryJournalComponent extends StatefulWidget {
  @override
  Widget build(BuildContext context) {
    return FutureBuilder(
      future: getMemoryJournal(),
      builder: (context, snapshot) {
        if (!snapshot.hasData) return LoadingSpinner();
        
        final memories = snapshot.data!;
        return ListView.builder(
          itemCount: memories.length,
          itemBuilder: (context, i) => MemoryCard(memories[i]),
        );
      },
    );
  }
}
```

### EndOfLifeComponent

```dart
class EndOfLifeComponent extends StatefulWidget {
  @override
  Widget build(BuildContext context) {
    return FutureBuilder(
      future: getLifeStageSummary(),
      builder: (context, snapshot) {
        if (!snapshot.hasData) return LoadingSpinner();
        
        final legacy = snapshot.data!;
        if (!legacy.hasImprint) {
          return Text('No digital legacy yet');
        }
        
        return DigitalLegacyView(legacy.imprint!);
      },
    );
  }
}
```

## Regenerating the Bridge

After making changes to `rust/syn_api/src/lib.rs`:

```bash
cd flutter
flutter_rust_bridge_codegen generate
```

This updates `lib/bridge/bridge_generated.dart`.

## Documentation

See `/docs/frb_api_cohesion.md` for complete API documentation.

## Testing

Run integration tests:

```bash
flutter test test/integration/bridge_integration_test.dart
```
