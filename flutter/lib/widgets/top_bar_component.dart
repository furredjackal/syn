import 'package:flame/components.dart';
import 'package:flutter/material.dart';
import 'package:syn/flutter/lib/syn_game.dart';
import 'package:syn/flutter/lib/theme/theme.dart';

class TopBarComponent extends PositionComponent with HasGameRef<SynGame> {
  @override
  Future<void> onLoad() async {
    final gameState = game.gameState;
    final lifeStageTheme = LifeStageTheme.fromStage(gameState.lifeStage);
    final moodColor = MoodColors.forMood(gameState.mood.toDouble());

    final background = RectangleComponent(
      paint: Paint()..color = Colors.black.withOpacity(0.3),
      size: size,
    );
    add(background);

    final lifeStageBadge = TextComponent(
      text: lifeStageTheme.badge,
      textRenderer: TextPaint(style: const TextStyle(fontSize: 20)),
      position: Vector2(16, 12),
    );
    add(lifeStageBadge);

    final lifeStageText = TextComponent(
      text: gameState.lifeStage.toUpperCase(),
      textRenderer: TextPaint(
        style: TextStyle(color: lifeStageTheme.primaryColor),
      ),
      position: Vector2(48, 16),
    );
    add(lifeStageText);

    final ageLabel = TextComponent(
      text: 'AGE',
      textRenderer: TextPaint(
        style: TextStyle(fontSize: 10, color: Colors.white.withOpacity(0.6)),
      ),
      position: Vector2(size.x / 2 - 50, 12),
    );
    add(ageLabel);

    final ageText = TextComponent(
      text: gameState.age.toString(),
      textRenderer: TextPaint(style: const TextStyle(fontSize: 32)),
      position: Vector2(size.x / 2 - 50, 28),
    );
    add(ageText);

    final moodLabel = TextComponent(
      text: 'MOOD',
      textRenderer: TextPaint(
        style: TextStyle(fontSize: 10, color: Colors.white.withOpacity(0.6)),
      ),
      position: Vector2(size.x / 2 + 50, 12),
    );
    add(moodLabel);

    final moodCircle = CircleComponent(
      radius: 30,
      position: Vector2(size.x / 2 + 50, 40),
      paint: Paint()..color = moodColor.withOpacity(0.1),
      children: [
        CircleComponent(
          radius: 30,
          paint: Paint()
            ..color = moodColor
            ..style = PaintingStyle.stroke
            ..strokeWidth = 2,
        ),
        TextComponent(
          text: gameState.mood.toString(),
          textRenderer: TextPaint(
            style: TextStyle(color: moodColor, fontSize: 20),
          ),
          anchor: Anchor.center,
          position: Vector2(30, 30),
        ),
      ],
    );
    add(moodCircle);

    // TODO: Add menu button
  }
}