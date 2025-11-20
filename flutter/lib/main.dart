import 'package:flame/game.dart';
import 'package:flutter/widgets.dart';
import 'overlays/confirmation_dialog_overlay.dart';
import 'overlays/debug_console_overlay.dart';
import 'overlays/loading_screen_overlay.dart';
import 'overlays/pause_menu_overlay.dart';
import 'overlays/settings_form_overlay.dart';
import 'overlays/text_input_overlay.dart';
import 'syn_game.dart';

void main() {
  final synGame = SynGame();
  runApp(
    GameWidget(
      game: synGame,
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
  );
}
