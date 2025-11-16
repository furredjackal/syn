import 'package:flame/components.dart';
import 'package:flame/effects.dart';
import 'package:flame/input.dart';
import 'package:flutter/material.dart';
import 'package:syn/flutter/lib/models/game_state.dart';
import 'package:syn/flutter/lib/syn_game.dart';
import 'package:syn/flutter/lib/widgets/stat_change_indicators_component.dart';

class ChoiceButtonComponent extends PositionComponent
    with HasGameRef<SynGame>, Tappable {
  final GameChoice choice;
  final int index;
  final VoidCallback onPressed;

  ChoiceButtonComponent({
    required this.choice,
    required this.index,
    required this.onPressed,
    Vector2? position,
    Vector2? size,
  }) : super(position: position, size: size);

  final _background = RectangleComponent();
  final _text = TextComponent();
  final _shortcut = TextComponent();

  @override
  Future<void> onLoad() async {
    _background.paint = Paint()..color = Colors.transparent;
    _background.size = size;
    add(_background);

    _text.text = choice.text.toUpperCase();
    _text.textRenderer = TextPaint(
      style: const TextStyle(
        color: Colors.white,
        fontSize: 16,
      ),
    );
    _text.position = Vector2(16, 16);
    add(_text);

    add(StatChangeIndicatorsComponent(
      statChanges: choice.statChanges,
      position: Vector2(16, 40),
    ));

    _shortcut.text = choice.keyboardShortcut.toString();
    _shortcut.textRenderer = TextPaint(
      style: TextStyle(
        color: Colors.white.withOpacity(0.7),
        fontSize: 14,
      ),
    );
    _shortcut.position = Vector2(size.x - 16, 16);
    _shortcut.anchor = Anchor.topRight;
    add(_shortcut);
  }

  @override
  bool onTapDown(TapDownInfo info) {
    add(ScaleEffect.to(
      Vector2.all(0.95),
      EffectController(duration: 0.1),
    ));
    return true;
  }

  @override
  bool onTapUp(TapUpInfo info) {
    add(ScaleEffect.to(
      Vector2.all(1.0),
      EffectController(duration: 0.1),
    ));
    onPressed();
    return true;
  }
}