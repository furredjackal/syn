import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:syn/models/game_state.dart';
import 'package:syn/widgets/event_card.dart';

void main() {
  group('EventCard Widget Tests', () {
    late GameEvent testEvent;
    int choiceSelected = -1;

    setUp(() {
      choiceSelected = -1;
      testEvent = GameEvent(
        id: 'test_event_1',
        title: 'Test Event',
        description: 'This is a test event description.',
        choices: [
          GameChoice(
            text: 'First choice',
            statChanges: {'Health': 5, 'Mood': 3},
            keyboardShortcut: 1,
          ),
          GameChoice(
            text: 'Second choice',
            statChanges: {'Health': -2, 'Mood': 8},
            keyboardShortcut: 2,
          ),
        ],
        lifeStage: 'adulthood',
        age: 25,
      );
    });

    testWidgets('EventCard renders with title and description',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: EventCard(
              event: testEvent,
              onChoice: (index) {
                choiceSelected = index;
              },
            ),
          ),
        ),
      );

      expect(find.text('TEST EVENT'), findsOneWidget);
      expect(find.text('This is a test event description.'), findsOneWidget);
    });

    testWidgets('EventCard displays all choices', (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: EventCard(
              event: testEvent,
              onChoice: (index) {
                choiceSelected = index;
              },
            ),
          ),
        ),
      );

      expect(find.text('FIRST CHOICE'), findsOneWidget);
      expect(find.text('SECOND CHOICE'), findsOneWidget);
    });

    testWidgets('EventCard shows stat changes for choices',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: EventCard(
              event: testEvent,
              onChoice: (index) {
                choiceSelected = index;
              },
            ),
          ),
        ),
      );

      // Check for stat changes display
      expect(find.text('+5 Health'), findsOneWidget);
      expect(find.text('+3 Mood'), findsOneWidget);
      expect(find.text('-2 Health'), findsOneWidget);
      expect(find.text('+8 Mood'), findsOneWidget);
    });

    testWidgets('EventCard renders keyboard shortcuts',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: EventCard(
              event: testEvent,
              onChoice: (index) {
                choiceSelected = index;
              },
            ),
          ),
        ),
      );

      // Check for keyboard shortcut numbers
      expect(find.text('1'), findsWidgets);
      expect(find.text('2'), findsWidgets);
    });

    testWidgets('EventCard has animation on entry',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: EventCard(
              event: testEvent,
              onChoice: (index) {},
            ),
          ),
        ),
      );

      // Check that animation is applied (scale and fade transitions exist)
      await tester.pumpAndSettle(); // Wait for animation to complete
      expect(find.byType(ScaleTransition), findsWidgets);
      expect(find.byType(FadeTransition), findsWidgets);
    });

    testWidgets('EventCard renders border and container',
        (WidgetTester tester) async {
      await tester.pumpWidget(
        MaterialApp(
          home: Scaffold(
            body: EventCard(
              event: testEvent,
              onChoice: (index) {
                choiceSelected = index;
              },
            ),
          ),
        ),
      );

      // Verify the main container exists
      expect(find.byType(Container), findsWidgets);
      expect(find.byType(SingleChildScrollView), findsOneWidget);
    });
  });
}
