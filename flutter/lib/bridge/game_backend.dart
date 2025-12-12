import 'package:syn/bridge/bridge_generated/frb_generated.dart';
import 'package:syn/bridge/bridge_generated/api.dart';
import 'package:syn/bridge/bridge_generated/lib.dart';
import 'package:syn/models/game_state.dart';

/// Thin wrapper around the generated FRB API.
///
/// Provides a clean Dart-friendly interface that maps Rust API types
/// to Flutter's GameState model.
class GameBackend {
  // Private constructor - use create() factory
  GameBackend._();

  /// Create and initialize the GameBackend.
  ///
  /// This must be called once at app startup before any other operations.
  static Future<GameBackend> create() async {
    await RustLib.init();
    return GameBackend._();
  }

  /// Start a new game with the given seed and player configuration.
  ///
  /// Returns the initial game state after character generation.
  Future<GameState> newGame(int seed, ApiPlayerConfig config) async {
    final snapshot = engineNewGame(
      seed: BigInt.from(seed),
      config: config,
    );

    if (snapshot == null) {
      throw Exception('Failed to create new game');
    }

    return GameState.fromApiSimple(snapshot);
  }

  /// Advance the simulation by the specified number of ticks.
  ///
  /// Returns the updated game state after time progression.
  Future<GameState> step(int ticks) async {
    final snapshot = engineStep(ticks: ticks);

    if (snapshot == null) {
      throw Exception('Failed to step simulation');
    }

    return GameState.fromApiSimple(snapshot);
  }

  /// Make a choice in the current event and advance time.
  ///
  /// Returns the updated game state after applying the choice outcome.
  Future<GameState> chooseOption(
    String storyletId,
    String choiceId,
    int ticks,
  ) async {
    final snapshot = engineChooseOption(
      storyletId: storyletId,
      choiceId: choiceId,
      ticks: ticks,
    );

    if (snapshot == null) {
      throw Exception('Failed to apply choice');
    }

    return GameState.fromApiSimple(snapshot);
  }

  /// Get the full game state snapshot with comprehensive data.
  ///
  /// Includes narrative heat, karma bands, and detailed life stage info.
  Future<GameState> getFullState() async {
    final snapshot = getGameStateSnapshot();

    if (snapshot == null) {
      throw Exception('Failed to get game state');
    }

    return GameState.fromApiFull(snapshot);
  }

  /// Get the current storylet/event if any.
  Future<GameEvent?> getCurrentEvent() async {
    final event = getCurrentStorylet();
    
    if (event == null) {
      return null;
    }

    return GameEvent.fromApi(event);
  }
}
