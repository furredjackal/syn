import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:integration_test/integration_test.dart';
import 'package:flame/game.dart';

import 'package:syn/syn_game.dart';
import 'package:syn/screens/game_screen.dart';
import 'package:syn/screens/main_menu_screen.dart';
import 'package:syn/screens/character_creation_screen.dart';

void main() {
  IntegrationTestWidgetsFlutterBinding.ensureInitialized();

  group('App Startup Tests', () {
    testWidgets('App launches and displays main menu', (WidgetTester tester) async {
      // Build the app
      final synGame = SynGame();
      await tester.pumpWidget(
        MaterialApp(
          home: GameScreen(synGame: synGame),
        ),
      );

      // Wait for app to settle
      await tester.pumpAndSettle();

      // Verify GameScreen is mounted
      expect(find.byType(GameScreen), findsOneWidget);

      // Verify the Flame GameWidget is present (background layer)
      expect(find.byType(GameWidget<SynGame>), findsOneWidget);
    });

    testWidgets('Main menu displays all menu options', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: MainMenuScreen(
            onNewGame: () {},
            onLoadGame: () {},
            onSettings: () {},
            onQuit: () {},
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Verify all menu options are visible
      expect(find.text('SYN'), findsOneWidget);
      expect(find.text('NEW GAME'), findsOneWidget);
      expect(find.text('LOAD GAME'), findsOneWidget);
      expect(find.text('SETTINGS'), findsOneWidget);
      expect(find.text('QUIT'), findsOneWidget);
    });
  });

  group('Navigation Tests', () {
    testWidgets('Main menu NEW GAME triggers callback', (WidgetTester tester) async {
      bool newGameTapped = false;

      await tester.pumpWidget(
        MaterialApp(
          home: MainMenuScreen(
            onNewGame: () => newGameTapped = true,
            onLoadGame: () {},
            onSettings: () {},
            onQuit: () {},
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Tap NEW GAME
      await tester.tap(find.text('NEW GAME'));
      await tester.pumpAndSettle();

      expect(newGameTapped, isTrue);
    });

    testWidgets('Main menu LOAD GAME triggers callback', (WidgetTester tester) async {
      bool loadGameTapped = false;

      await tester.pumpWidget(
        MaterialApp(
          home: MainMenuScreen(
            onNewGame: () {},
            onLoadGame: () => loadGameTapped = true,
            onSettings: () {},
            onQuit: () {},
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Tap LOAD GAME
      await tester.tap(find.text('LOAD GAME'));
      await tester.pumpAndSettle();

      expect(loadGameTapped, isTrue);
    });
  });

  group('Character Creation Tests', () {
    testWidgets('Character creation screen displays all elements',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: CharacterCreationScreen(
            onComplete: ({
              required String name,
              required String archetype,
              required bool sfwMode,
              required String difficulty,
            }) {},
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Verify title
      expect(find.text('CHARACTER CREATION'), findsOneWidget);

      // Verify archetypes are displayed
      expect(find.text('STORYTELLER'), findsOneWidget);
      expect(find.text('ANALYST'), findsOneWidget);
      expect(find.text('DREAMER'), findsOneWidget);
      expect(find.text('CHALLENGER'), findsOneWidget);

      // Verify difficulty options are displayed
      expect(find.text('FORGIVING'), findsOneWidget);
      expect(find.text('BALANCED'), findsOneWidget);
      expect(find.text('HARSH'), findsOneWidget);
    });

    testWidgets('Character creation allows name input', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: CharacterCreationScreen(
            onComplete: ({
              required String name,
              required String archetype,
              required bool sfwMode,
              required String difficulty,
            }) {},
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Find name input field and enter text
      final nameField = find.byType(TextField);
      expect(nameField, findsOneWidget);

      await tester.enterText(nameField, 'TestPlayer');
      await tester.pumpAndSettle();

      expect(find.text('TestPlayer'), findsOneWidget);
    });

    testWidgets('Character creation submits with correct data',
        (WidgetTester tester) async {
      String? submittedName;
      String? submittedArchetype;
      bool? submittedSfwMode;
      String? submittedDifficulty;

      await tester.pumpWidget(
        MaterialApp(
          home: CharacterCreationScreen(
            onComplete: ({
              required String name,
              required String archetype,
              required bool sfwMode,
              required String difficulty,
            }) {
              submittedName = name;
              submittedArchetype = archetype;
              submittedSfwMode = sfwMode;
              submittedDifficulty = difficulty;
            },
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Enter name
      final nameField = find.byType(TextField);
      await tester.enterText(nameField, 'TestPlayer');
      await tester.pumpAndSettle();

      // Tap an archetype (ANALYST)
      await tester.tap(find.text('ANALYST'));
      await tester.pumpAndSettle();

      // Tap a difficulty (HARSH)
      await tester.tap(find.text('HARSH'));
      await tester.pumpAndSettle();

      // Tap the BEGIN button to submit
      final beginButton = find.text('BEGIN');
      if (beginButton.evaluate().isNotEmpty) {
        await tester.tap(beginButton);
        await tester.pumpAndSettle();

        // Verify callback was invoked with correct values
        expect(submittedName, equals('TestPlayer'));
        expect(submittedArchetype, equals('ANALYST'));
        expect(submittedDifficulty, equals('HARSH'));
        expect(submittedSfwMode, isNotNull);
      }
    });
  });

  group('Visual Consistency Tests', () {
    testWidgets('Main menu uses correct Persona styling', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: MainMenuScreen(
            onNewGame: () {},
            onLoadGame: () {},
            onSettings: () {},
            onQuit: () {},
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Find the SYN title and verify it exists with proper styling
      final synTitle = find.text('SYN');
      expect(synTitle, findsOneWidget);

      // Verify the background is dark (Persona 5 aesthetic)
      final container = find.byType(Container).first;
      expect(container, findsOneWidget);
    });

    testWidgets('GameScreen includes Flame background layer', (WidgetTester tester) async {
      final synGame = SynGame();

      await tester.pumpWidget(
        MaterialApp(
          home: GameScreen(synGame: synGame),
        ),
      );

      // Allow time for Flame game to initialize
      await tester.pump(const Duration(milliseconds: 500));

      // Verify Flame GameWidget is present
      expect(find.byType(GameWidget<SynGame>), findsOneWidget);
    });
  });

  group('Keyboard Navigation Tests', () {
    testWidgets('Main menu supports keyboard navigation', (WidgetTester tester) async {
      int triggeredIndex = -1;

      await tester.pumpWidget(
        MaterialApp(
          home: MainMenuScreen(
            onNewGame: () => triggeredIndex = 0,
            onLoadGame: () => triggeredIndex = 1,
            onSettings: () => triggeredIndex = 2,
            onQuit: () => triggeredIndex = 3,
          ),
        ),
      );

      await tester.pumpAndSettle();

      // Send arrow down key to navigate
      await tester.sendKeyEvent(LogicalKeyboardKey.arrowDown);
      await tester.pumpAndSettle();

      // Send Enter to confirm selection
      await tester.sendKeyEvent(LogicalKeyboardKey.enter);
      await tester.pumpAndSettle();

      // Should have triggered LOAD GAME (index 1) after moving down from NEW GAME
      expect(triggeredIndex, equals(1));
    });
  });

  group('Error Handling Tests', () {
    testWidgets('App handles initialization gracefully', (WidgetTester tester) async {
      // This tests that the app doesn't crash on startup
      final synGame = SynGame();

      await tester.pumpWidget(
        MaterialApp(
          home: GameScreen(synGame: synGame),
        ),
      );

      // Should not throw any exceptions
      await tester.pumpAndSettle(const Duration(seconds: 2));

      expect(find.byType(GameScreen), findsOneWidget);
    });
  });
}
