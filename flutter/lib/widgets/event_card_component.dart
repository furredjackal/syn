import 'package:flame/components.dart';
import 'package:flutter/material.dart';
import 'package:syn/flutter/lib/models/game_state.dart';
import 'package:syn/flutter/lib/syn_game.dart';
import 'package:syn/flutter/lib/widgets/choice_button_component.dart';

class EventCardComponent extends PositionComponent with HasGameRef<SynGame> {
  final GameEvent event;
  final Function(int) onChoice;

  EventCardComponent({
    required this.event,
    required this.onChoice,
    Vector2? position,
    Vector2? size,
  }) : super(position: position, size: size);

  @override
  Future<void> onLoad() async {
    final background = RectangleComponent(
      paint: Paint()..color = Colors.black.withOpacity(0.4),
      size: size,
    );
    add(background);

    final title = TextComponent(
      text: event.title.toUpperCase(),
      textRenderer: TextPaint(
        style: const TextStyle(
          color: Color(0xFF00D9FF),
          fontSize: 32,
        ),
      ),
      position: Vector2(24, 24),
    );
    add(title);

    final description = TextComponent(
      text: event.description,
      textRenderer: TextPaint(
        style: const TextStyle(
          color: Colors.white,
          fontSize: 16,
        ),
      ),
      position: Vector2(24, 80),
    );
    add(description);

    double yOffset = 140;
    for (var i = 0; i < event.choices.length; i++) {
      final choice = event.choices[i];
      final choiceButton = ChoiceButtonComponent(
        choice: choice,
        index: i,
        onPressed: () => onChoice(i),
        position: Vector2(24, yOffset),
        size: Vector2(size.x - 48, 80),
      );
      add(choiceButton);
      yOffset += 96;
    }
  }
}