import 'package:flame/game.dart';
import 'package:flutter/widgets.dart';
import 'overlays/confirmation_dialog_overlay.dart';
import 'overlays/loading_screen_overlay.dart';
import 'overlays/pause_menu_overlay.dart';
import 'overlays/transition_overlay.dart';
import 'syn_game.dart';

void main() {
  final synGame = SynGame();
  runApp(
    GameWidget(
      game: synGame,
      overlayBuilderMap: {
        'PauseMenuOverlay': (context, game) =>
            PauseMenuOverlay(game: game as SynGame),
        'LoadingScreenOverlay': (context, game) =>
            const LoadingScreenOverlay(),
        'TransitionOverlay': (context, game) {
          final syn = game as SynGame;
          return TransitionOverlay(onComplete: syn.onTransitionOverlayComplete);
        },
        'ConfirmationDialogOverlay': (context, game) =>
            ConfirmationDialogOverlay(game: game as SynGame),
      },
    ),
  );
}

/*
 * SYN Overlay Classification (Nov 2024)
 * NOTE: PauseMenuOverlay, LoadingScreenOverlay, and TransitionOverlay /
 *       ConfirmationDialogOverlay are now triggered directly from SynGame
 *       (pause toggle, scene transitions, long operations, and confirmations).
 *       The remaining overlays are registered but still unused until wired later.
 *
 * ID                      | Role / Decision Tree                        | Keep as | Notes
 * ------------------------|---------------------------------------------|---------|-------------------------------
 * PauseMenuOverlay        | Pauses game, modal menu                     | Overlay | Active via ESC + quick menu.
 * LoadingScreenOverlay    | Blocking loading screen (modal)             | Overlay | Used during gameplay start/load.
 * TransitionOverlay       | Modal transition animation                  | Overlay | Used during scene transitions.
 * ConfirmationDialogOverlay| Modal confirm dialog                       | Overlay | Used for destructive prompts.
 *
 * No living HUD elements are implemented as overlays, so nothing needs
 * migration to Flame HUD components at this time; main work is wiring or
 * removing unused overlay widgets in the future.
 */
