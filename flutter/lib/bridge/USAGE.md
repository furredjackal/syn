# GameBackend Usage Guide

## Overview

The `GameBackend` class provides a clean Dart interface to the Rust simulation engine via `flutter_rust_bridge`. All Rust API types are automatically mapped to Flutter's `GameState` model.

## Initialization

Initialize once at app startup:

```dart
// In main.dart or app initialization
final backend = await GameBackend.create();
```

## Starting a New Game

```dart
import 'package:syn/bridge/bridge_generated/lib.dart';
import 'package:syn/bridge/game_backend.dart';

// Configure the player
final config = ApiPlayerConfig(
  name: 'Alex',
  pronouns: 'they/them', // Optional
  archetype: 'STORYTELLER', // or ANALYST, DREAMER, CHALLENGER
  difficulty: 'BALANCED',   // or FORGIVING, HARSH
  sfwMode: true,
);

// Create new game with random seed
final state = await backend.newGame(
  DateTime.now().millisecondsSinceEpoch,
  config,
);

// state is now a fully populated GameState
print('${state.playerName}, age ${state.age}');
print('Health: ${state.health}');
```

## Game Loop Operations

### Advancing Time

```dart
// Step forward by N ticks (1 tick = 1 hour)
final newState = await backend.step(24); // Advance 1 day

// Access updated data
print('Day ${newState.year * 365 + (newState.age - 6) * 365}');
print('Current event: ${newState.currentEvent?.title ?? "None"}');
```

### Making Choices

```dart
// When a storylet/event is active
if (state.currentEvent != null) {
  final event = state.currentEvent!;
  final choice = event.choices[0]; // User's selected choice
  
  // Apply the choice
  final updatedState = await backend.chooseOption(
    event.id,
    choice.text, // Choice ID (uses label as ID for now)
    24,          // Advance 1 day after choice
  );
  
  print('Choice applied! New karma: ${updatedState.karma}');
}
```

### Getting Full State

For comprehensive data (narrative heat, karma bands, etc.):

```dart
final fullState = await backend.getFullState();

print('Narrative Heat: ${fullState.narrativeHeat}/200');
print('Relationships: ${fullState.relationships.length}');
print('Memories: ${fullState.memories.length}');
```

## Type Mapping

### API Types → GameState

| Rust API Type | Dart Type | GameState Field |
|---------------|-----------|-----------------|
| `ApiSimpleGameState` | `GameState` | via `.fromApiSimple()` |
| `ApiGameStateSnapshot` | `GameState` | via `.fromApiFull()` |
| `ApiDirectorEventView` | `GameEvent` | via `.fromApi()` |
| `ApiSimpleRelationship` | `RelationshipData` | via `.fromApiSimple()` |
| `ApiRelationship` | `RelationshipData` | via `.fromApiFull()` |

### Stats Mapping

Rust API provides stats as a list of `ApiStat` with `kind` (String) and `value` (double).

Mapped to GameState int fields (0-100):
- `Health` → `health`
- `Mood` → `mood` (-10 to +10)
- `Wealth` → `wealth`
- `Charisma` → `charisma`
- `Intelligence` → `intelligence`
- `Wisdom` → `wisdom`
- `Strength` → `strength`
- `Stability` → `stability`

### Relationships Mapping

**Simple version** (`ApiSimpleRelationship`):
- `strength: double` (-1.0 to 1.0) → scaled to all 5 axes
- Derives state label: Enemy, Rival, Stranger, Friend, CloseFriend

**Full version** (`ApiRelationship`):
- 5-axis values directly mapped: `affection`, `trust`, `attraction`, `familiarity`, `resentment`
- Includes band labels: `affectionBand`, `trustBand`, etc.
- Includes role label: Friend, Rival, Crush, Stranger, etc.

## Error Handling

All methods throw `Exception` if the Rust API returns null:

```dart
try {
  final state = await backend.step(24);
} catch (e) {
  print('Simulation error: $e');
  // Handle gracefully - show error dialog, reload last save, etc.
}
```

## Integration with UI

### Using with Provider/Riverpod

```dart
final gameStateProvider = StateNotifierProvider<GameStateNotifier, GameState>((ref) {
  return GameStateNotifier(ref.read(gameBackendProvider));
});

class GameStateNotifier extends StateNotifier<GameState> {
  final GameBackend backend;
  
  GameStateNotifier(this.backend) : super(GameState());
  
  Future<void> startNewGame(String name, String archetype) async {
    final config = ApiPlayerConfig(
      name: name,
      archetype: archetype,
      difficulty: 'BALANCED',
      sfwMode: true,
    );
    
    state = await backend.newGame(
      DateTime.now().millisecondsSinceEpoch,
      config,
    );
  }
  
  Future<void> advanceTime(int ticks) async {
    state = await backend.step(ticks);
  }
  
  Future<void> makeChoice(String storyletId, String choiceId) async {
    state = await backend.chooseOption(storyletId, choiceId, 24);
  }
}
```

### Example Screen

```dart
class GameScreen extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(gameStateProvider);
    final notifier = ref.read(gameStateProvider.notifier);
    
    return Scaffold(
      body: Column(
        children: [
          Text('${state.playerName}, Age ${state.age}'),
          Text('Health: ${state.health}'),
          if (state.currentEvent != null) ...[
            Text(state.currentEvent!.title),
            ...state.currentEvent!.choices.map((choice) =>
              ElevatedButton(
                onPressed: () => notifier.makeChoice(
                  state.currentEvent!.id,
                  choice.text,
                ),
                child: Text(choice.text),
              ),
            ),
          ] else
            ElevatedButton(
              onPressed: () => notifier.advanceTime(24),
              child: Text('Next Day'),
            ),
        ],
      ),
    );
  }
}
```

## Architecture Notes

### Determinism
All simulation is deterministic - same seed produces same results. Use `DateTime.now().millisecondsSinceEpoch` for random seeds, or store/reuse seeds for reproducible runs.

### State Immutability
`GameBackend` methods return **new** `GameState` instances. Never mutate the returned state directly for simulation changes - always call backend methods.

### Performance
- `step()` and `chooseOption()` use `ApiSimpleGameState` (lightweight)
- `getFullState()` uses `ApiGameStateSnapshot` (comprehensive, includes all relationship axes)
- Use simple version for frequent updates, full version for detailed views

### Future Extensions
- Add player name to GameState (currently only in config)
- Retrieve NPC names for relationships (currently shows `NPC_<id>`)
- Expand event data (add description, art paths from storylet content)
