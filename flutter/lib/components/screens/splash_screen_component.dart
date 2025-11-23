
import 'package:flame/components.dart';
import 'package:flutter/material.dart';
import '../../syn_game.dart';

class SplashScreenComponent extends PositionComponent
    with HasGameReference<SynGame> {
  @override
  Future<void> onLoad() async {
    size = game.size;
    add(_SplashBackground()..size = size);

    add(
      TextComponent(
        text: 'S Y n',
        anchor: Anchor.center,
        position: size / 2,
        textRenderer: TextPaint(
          style: const TextStyle(
            fontSize: 96,
            fontWeight: FontWeight.w900,
            letterSpacing: 24,
            color: Color(0xFFFFFFFF),
          ),
        ),
      ),
    );

    add(
      TextComponent(
        text: 'Simulate Your Narrative',
        anchor: Anchor.center,
        position: Vector2(size.x / 2, size.y / 2 + 80),
        textRenderer: TextPaint(
          style: const TextStyle(
            color: Color(0xFFEEEEEE),
            fontSize: 20,
            letterSpacing: 4,
          ),
        ),
      ),
    );

    add(
      TimerComponent(
        period: 2.4,
        onTick: () {
          game.showMainMenu();
        },
      ),
    );
  }

  @override
  void onGameResize(Vector2 size) {
    super.onGameResize(size);
    this.size = size;
  }
}

class _SplashBackground extends PositionComponent {
  @override
  void render(Canvas canvas) {
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    final gradient = Paint()
      ..shader = const LinearGradient(
        colors: [
          Color(0xFF050505),
          Color(0xFF101010),
        ],
        begin: Alignment.topCenter,
        end: Alignment.bottomCenter,
      ).createShader(rect);
    canvas.drawRect(rect, gradient);

    for (int i = 0; i < 30; i++) {
      final y = i * size.y / 30;
      canvas.drawLine(
        Offset(0, y),
        Offset(size.x, y),
        Paint()
          ..color = const Color(0x1100D9FF)
          ..strokeWidth = 1,
      );
    }
  }
}

class SplashScreen extends SplashScreenComponent {}
