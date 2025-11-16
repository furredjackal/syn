import 'package:flame/components.dart';
import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';
import 'syn_game.dart';

class SplashScreenComponent extends PositionComponent with HasGameRef<SynGame> {
  @override
  Future<void> onLoad() async {
    super.onLoad();
    
    size = gameRef.size;

    // Add background
    add(RectangleComponent(
      paint: Paint()..color = const Color(0xFF0A0E27),
      size: size,
    ));

    // Add title text
    add(TextComponent(
      text: 'SYN',
      textRenderer: TextPaint(
        style: GoogleFonts.audiowide(
          fontSize: 80,
          color: const Color(0xFF00D9FF),
          fontWeight: FontWeight.bold,
        ),
      ),
      position: size / 2,
      anchor: Anchor.center,
    ));

    // Add subtitle
    add(TextComponent(
      text: 'Simulate Your Narrative',
      textRenderer: TextPaint(
        style: GoogleFonts.roboto(
          fontSize: 18,
          color: Colors.white,
          letterSpacing: 2,
        ),
      ),
      position: Vector2(size.x / 2, size.y / 2 + 60),
      anchor: Anchor.center,
    ));

    // Navigate after delay
    Future.delayed(const Duration(seconds: 3), () {
      // TODO: Navigate to menu when router is available
    });
  }
}

