import 'package:flame/components.dart';
import 'package:flutter/material.dart';
import '../syn_game.dart';
import '../theme/theme.dart';

class TopBarComponent extends PositionComponent with HasGameReference<SynGame> {
  late TextComponent _lifeStageBadge;
  late TextComponent _lifeStageText;
  late TextComponent _ageText;
  late CircleComponent _moodFill;
  late CircleComponent _moodOutline;
  late TextComponent _moodValue;

  @override
  Future<void> onLoad() async {
    final gameState = game.gameState;
    final lifeStageTheme = LifeStageTheme.fromStage(gameState.lifeStage);
    final moodColor = MoodColors.forMood(gameState.mood.toDouble());

    add(RectangleComponent(
      paint: Paint()..color = Colors.black.withOpacity(0.3),
      size: size,
    ));

    _lifeStageBadge = TextComponent(
      text: lifeStageTheme.badge,
      textRenderer: TextPaint(style: const TextStyle(fontSize: 20)),
      position: Vector2(16, 12),
    );
    add(_lifeStageBadge);

    _lifeStageText = TextComponent(
      text: gameState.lifeStage.toUpperCase(),
      textRenderer: TextPaint(
        style: TextStyle(color: lifeStageTheme.primaryColor),
      ),
      position: Vector2(48, 16),
    );
    add(_lifeStageText);

    add(TextComponent(
      text: 'AGE',
      textRenderer: TextPaint(
        style: TextStyle(fontSize: 10, color: Colors.white.withOpacity(0.6)),
      ),
      position: Vector2(size.x / 2 - 50, 12),
    ));

    _ageText = TextComponent(
      text: gameState.age.toString(),
      textRenderer: TextPaint(style: const TextStyle(fontSize: 32)),
      position: Vector2(size.x / 2 - 50, 28),
    );
    add(_ageText);

    add(TextComponent(
      text: 'MOOD',
      textRenderer: TextPaint(
        style: TextStyle(fontSize: 10, color: Colors.white.withOpacity(0.6)),
      ),
      position: Vector2(size.x / 2 + 50, 12),
    ));

    final moodCenter = Vector2(size.x / 2 + 50, 40);
    _moodFill = CircleComponent(
      radius: 30,
      anchor: Anchor.center,
      position: moodCenter,
      paint: Paint()..color = moodColor.withOpacity(0.1),
    );
    _moodOutline = CircleComponent(
      radius: 30,
      anchor: Anchor.center,
      position: moodCenter,
      paint: Paint()
        ..color = moodColor
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2,
    );
    _moodValue = TextComponent(
      text: gameState.mood.toString(),
      textRenderer: TextPaint(
        style: TextStyle(color: moodColor, fontSize: 20),
      ),
      anchor: Anchor.center,
      position: moodCenter,
    );

    add(_moodFill);
    add(_moodOutline);
    add(_moodValue);

    // TODO: Add menu button
  }

  @override
  void update(double dt) {
    super.update(dt);
    final gameState = game.gameState;
    final lifeStageTheme = LifeStageTheme.fromStage(gameState.lifeStage);
    final moodColor = MoodColors.forMood(gameState.mood.toDouble());

    _lifeStageBadge.text = lifeStageTheme.badge;
    _lifeStageText
      ..text = gameState.lifeStage.toUpperCase()
      ..textRenderer = TextPaint(
        style: TextStyle(color: lifeStageTheme.primaryColor),
      );

    _ageText.text = gameState.age.toString();

    _moodFill.paint.color = moodColor.withOpacity(0.1);
    _moodOutline.paint
      ..color = moodColor
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2;

    _moodValue
      ..text = gameState.mood.toString()
      ..textRenderer = TextPaint(
        style: TextStyle(color: moodColor, fontSize: 20),
      );
  }
}
