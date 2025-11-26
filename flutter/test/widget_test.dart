import 'package:flutter/material.dart';
import 'package:flame/game.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:syn/overlays/confirmation_dialog_overlay.dart';
import 'package:syn/overlays/loading_screen_overlay.dart';
import 'package:syn/overlays/pause_menu_overlay.dart';
import 'package:syn/overlays/settings_form_overlay.dart';
import 'package:syn/overlays/text_input_overlay.dart';
import 'package:syn/overlays/debug_console_overlay.dart';
import 'package:syn/syn_game.dart';

void main() {
  group('Widget Tests', () {
    testWidgets('GameWidget renders with SynGame', (WidgetTester tester) async {
      final game = SynGame();
      await tester.pumpWidget(
        MaterialApp(
          home: GameWidget(
            game: game,
            overlayBuilderMap: {
              'text_input': (context, g) => buildTextInputOverlay(context, g as SynGame),
              'pause_menu': (context, g) => buildPauseMenuOverlay(context, g as SynGame),
              'confirm_dialog': (context, g) => buildConfirmDialogOverlay(context, g as SynGame),
              'loading': (context, g) => buildLoadingOverlay(context),
              'settings_form': (context, g) => buildSettingsFormOverlay(context, g as SynGame),
              'debug_console': (context, g) => buildDebugConsoleOverlay(context, g as SynGame),
            },
          ),
        ),
      );

      // Smoke check: GameWidget is in the tree
      expect(find.byType(GameWidget), findsOneWidget);
      // Initial route is splash; overlay is not active
      expect(game.overlays.isActive('pause_menu'), isFalse);
    });

    testWidgets('Pause overlay shows and hides on resume', (WidgetTester tester) async {
      final game = SynGame();
      await tester.pumpWidget(
        MaterialApp(
          home: GameWidget(
            game: game,
            overlayBuilderMap: {
              'pause_menu': (context, g) => buildPauseMenuOverlay(context, g as SynGame),
            },
          ),
        ),
      );

      // Show pause overlay
      game.showPauseOverlay();
      await tester.pumpAndSettle();
      expect(game.overlays.isActive('pause_menu'), isTrue);
      expect(find.text('PAUSE'), findsOneWidget);

      // Tap RESUME to close overlay
      await tester.tap(find.text('RESUME'));
      await tester.pumpAndSettle();
      expect(game.overlays.isActive('pause_menu'), isFalse);
      expect(find.text('PAUSE'), findsNothing);
    });

    testWidgets('Confirmation dialog confirm calls handler', (WidgetTester tester) async {
      final game = SynGame();
      await tester.pumpWidget(
        MaterialApp(
          home: GameWidget(
            game: game,
            overlayBuilderMap: {
              'confirm_dialog': (context, g) => buildConfirmDialogOverlay(context, g as SynGame),
            },
          ),
        ),
      );

      var confirmed = false;
      game.showConfirmationDialog(
        title: 'Confirm Action',
        message: 'Are you sure?',
        onConfirm: () => confirmed = true,
      );

      await tester.pumpAndSettle();
      expect(find.text('Confirm Action'), findsOneWidget);
      expect(find.text('Are you sure?'), findsOneWidget);
      expect(game.overlays.isActive('confirm_dialog'), isTrue);

      // Tap Confirm
      await tester.tap(find.text('Confirm'));
      await tester.pumpAndSettle();

      expect(confirmed, isTrue);
      expect(game.overlays.isActive('confirm_dialog'), isFalse);
    });
  });
}
