// ==================== FRB API SUMMARY ====================
//
// 1) MAIN DART ENTRYPOINT:
//    - Class: `RustLib` (singleton instance via `RustLib.instance`)
//    - Initialization: `await RustLib.init();` (must be called before using any API)
//    - Access API methods via: `RustLib.instance.api.crateApiMethodName(...)`
//    - Or use the convenience functions in `api.dart` (auto-imports from this file)
//
// 2) RUST FUNCTION MAPPINGS (via RustLib.instance.api):
//
//    New Game:
//    - `crateApiEngineNewGame({required BigInt seed, required ApiPlayerConfig config})`
//      → Returns: `ApiSimpleGameState?`
//      → Convenience: `engineNewGame(seed: seed, config: config)` from api.dart
//
//    Step Simulation:
//    - `crateApiEngineStep({required int ticks})`
//      → Returns: `ApiSimpleGameState?`
//      → Convenience: `engineStep(ticks: ticks)` from api.dart
//
//    - `crateApiEngineTick()` (single tick)
//      → Returns: `void`
//      → Convenience: `engineTick()` from api.dart
//
//    - `crateApiEngineTickMany({required int count})` (multiple ticks)
//      → Returns: `void`
//      → Convenience: `engineTickMany(count: count)` from api.dart
//
//    Choose Option:
//    - `crateApiEngineChooseOption({required String storyletId, required String choiceId, required int ticks})`
//      → Returns: `ApiSimpleGameState?`
//      → Convenience: `engineChooseOption(storyletId: ..., choiceId: ..., ticks: ...)` from api.dart
//
//    State Getters:
//    - `crateApiGetGameStateSnapshot()` (full/comprehensive state)
//      → Returns: `ApiGameStateSnapshot?`
//      → Convenience: `getGameStateSnapshot()` from api.dart
//
//    Event/Storylet Getters:
//    - `crateApiGetCurrentStorylet()` (current event)
//      → Returns: `ApiDirectorEventView?`
//      → Convenience: `getCurrentStorylet()` from api.dart
//
//    - `crateApiGetAvailableChoices()` (choices for current event)
//      → Returns: `List<ApiDirectorChoiceView>`
//      → Convenience: `getAvailableChoices()` from api.dart
//
// 3) DART TYPE MAPPINGS (from Rust structs):
//
//    Character Creation Config:
//    - `ApiPlayerConfig`
//      Fields: name (String), pronouns (String?), archetype (String),
//              difficulty (String), sfwMode (bool)
//
//    Game State Snapshots:
//    - `ApiSimpleGameState` (simplified snapshot for UI)
//      Fields: currentDay (int), currentTick (BigInt), playerAge (int),
//              lifeStage (String), stats (ApiStatsSnapshot), mood (String),
//              karma (double), currentEvent (ApiDirectorEventView?),
//              relationships (List<ApiSimpleRelationship>),
//              recentMemories (List<String>)
//
//    - `ApiGameStateSnapshot` (comprehensive/full snapshot)
//      Fields: currentTick (BigInt), playerAgeYears (int), lifeStage (String),
//              stats (ApiStatsSnapshot), relationships (ApiRelationshipSnapshot),
//              narrativeHeat (double), heatLevel (String), heatTrend (double),
//              currentEvent (ApiDirectorEventView?), karma (double),
//              karmaBand (String), moodBand (String),
//              lifeStageInfo (ApiLifeStageInfo)
//
//    Current Event:
//    - `ApiDirectorEventView`
//      Fields: storyletId (String), title (String),
//              choices (List<ApiDirectorChoiceView>)
//
//    - `ApiDirectorChoiceView`
//      Fields: id (String), label (String)
//
//    Relationship Snapshots:
//    - `ApiRelationshipSnapshot` (full relationship data)
//      Fields: relationships (List<ApiRelationship>)
//
//    - `ApiRelationship` (detailed 5-axis relationship)
//      Fields: actorId (PlatformInt64), targetId (PlatformInt64),
//              affection (double), trust (double), attraction (double),
//              familiarity (double), resentment (double),
//              affectionBand (String), trustBand (String),
//              attractionBand (String), resentmentBand (String),
//              roleLabel (String)
//
//    - `ApiSimpleRelationship` (simplified relationship)
//      Fields: npcId (PlatformInt64), name (String), strength (double)
//
//    Stats:
//    - `ApiStatsSnapshot`
//      Fields: stats (List<ApiStat>), moodBand (String)
//
//    - `ApiStat`
//      Fields: kind (String), value (double)
//
//    Life Stage Info:
//    - `ApiLifeStageInfo`
//      Fields: lifeStage (String), playerAgeYears (int),
//              showWealth (bool), showReputation (bool),
//              showWisdom (bool), showKarma (bool)
//
// USAGE EXAMPLE:
//
//   import 'package:syn/bridge/bridge_generated/api.dart';
//   import 'package:syn/bridge/bridge_generated/lib.dart';
//
//   // Initialize once at app startup
//   await RustLib.init();
//
//   // Create new game
//   final config = ApiPlayerConfig(
//     name: "Alice",
//     archetype: "STORYTELLER",
//     difficulty: "BALANCED",
//     sfwMode: true,
//   );
//   final state = engineNewGame(seed: BigInt.from(42), config: config);
//
//   // Step simulation
//   final newState = engineStep(ticks: 10);
//
//   // Make choice
//   final afterChoice = engineChooseOption(
//     storyletId: "some_event",
//     choiceId: "choice_1",
//     ticks: 5,
//   );
//
//   // Get full state
//   final fullState = getGameStateSnapshot();
//
// ==================== END FRB API SUMMARY ====================