import 'package:flame/components.dart';
import 'package:flame/effects.dart';
import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';
import '../../syn_game.dart';

class SplashScreenComponent extends Component with HasGameRef<SynGame> {
  @override
  Future<void> onLoad() async {
    super.onLoad();

    // 1. Add background gradient
    add(_Background());

    // 2. Setup text styles from the theme
    final titleStyle = GoogleFonts.audiowide(
      fontSize: 80,
      color: const Color(0xFF00D9FF),
      shadows: [
        const Shadow(
          blurRadius: 20,
          color: Color(0xFF00D9FF),
          offset: Offset(0, 0),
        ),
      ],
    );

    final subtitleStyle = GoogleFonts.roboto(
      letterSpacing: 2,
      fontSize: 18,
      color: Colors.white.withOpacity(0.9),
    );

    // 3. Create and add animated components
    final title = TextComponent(
      text: 'SYN',
      textRenderer: TextPaint(style: titleStyle),
      anchor: Anchor.center,
    );

    final subtitle = TextComponent(
      text: 'Simulate Your Narrative',
      textRenderer: TextPaint(style: subtitleStyle),
      anchor: Anchor.topCenter,
      position: Vector2(0, 60), // Relative to title's anchor
    );

    // Group logo and subtitle to animate them together
    final logoGroup = PositionComponent(
      position: gameRef.size / 2,
      anchor: Anchor.center,
      children: [title, subtitle],
    );

    // 4. Define and apply animations
    final animationDuration = 2.0;
    final animationCurve = Curves.easeInOut;

    logoGroup.add(
      ScaleEffect.to(
        Vector2.all(1.2),
        EffectController(duration: animationDuration, curve: animationCurve),
      ),
    );
    logoGroup.add(
      OpacityEffect.fadeIn(
        EffectController(duration: animationDuration, curve: animationCurve),
      ),
    );

    // Set initial state for animations
    logoGroup.scale = Vector2.all(0.5);
    logoGroup.children.setAll(0, [OpacityProvider(opacity: 0)]);

    add(logoGroup);

    // 5. Add version text
    add(
      TextComponent(
        text: 'v0.1.0',
        textRenderer: TextPaint(
          style: GoogleFonts.roboto(
            color: Colors.white.withOpacity(0.4),
            fontSize: 14,
          ),
        ),
        anchor: Anchor.bottomRight,
        position: gameRef.size - Vector2.all(20),
      ),
    );

    // 6. Setup timer to navigate after animations
    const navigationDelay = 1.0; // 1 second delay after animation
    add(
      TimerComponent(
        period: animationDuration + navigationDelay,
        onTick: () => game.router.pushNamed('menu'),
        removeOnFinish: true,
      ),
    );
  }
}

class _Background extends Component {
  @override
  void render(Canvas canvas) {
    final gradient = LinearGradient(
      begin: Alignment.topLeft,
      end: Alignment.bottomRight,
      colors: [const Color(0xFF0A0E27), const Color(0xFF2D1B4E)],
    ).createShader(Rect.fromLTWH(0, 0, size.x, size.y));

    canvas.drawRect(Rect.fromLTWH(0, 0, size.x, size.y), Paint()..shader = gradient);
  }
}
