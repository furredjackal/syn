import 'package:flutter/material.dart';

/// Centralized SYN UI theme helpers.
class SynColors {
  SynColors._();

  static const Color primaryCyan = Color(0xFF00D9FF);
  static const Color accentViolet = Color(0xFF7B5CFF);
  static const Color accentRed = Color(0xFFFF4C4C);
  static const Color accentOrange = Color(0xFFFF8F5B);
  static const Color accentGreen = Color(0xFF5CFF90);
  static const Color accentGold = Color(0xFFFFC857);
  static const Color accentCyan = Color(0xFF2FE2FF);
  static const Color accentMagenta = Color(0xFFB45BFF);
  static const Color accentIndigo = Color(0xFF8E9BFF);

  static const Color bgDark = Color(0xFF05070D);
  static const Color bgPanel = Color(0xFF0E111A);
  static const Color bgOverlay = Color(0xCC05070D);

  static const Color textPrimary = Colors.white;
  static const Color textSubtle = Color(0xFFCACED4);
  static const Color textMuted = Color(0xFF6E7382);
}

class SynTextStyles {
  SynTextStyles._();

  static const TextStyle h1Event = TextStyle(
    fontFamily: 'Montserrat',
    fontSize: 36,
    fontWeight: FontWeight.w900,
    letterSpacing: 1.8,
    color: SynColors.textPrimary,
  );

  static const TextStyle h2Strip = TextStyle(
    fontFamily: 'Montserrat',
    fontSize: 14,
    letterSpacing: 2,
    fontWeight: FontWeight.w700,
    color: SynColors.textPrimary,
  );

  static const TextStyle body = TextStyle(
    fontFamily: 'Montserrat',
    fontSize: 14,
    fontWeight: FontWeight.w500,
    color: SynColors.textPrimary,
  );

  static const TextStyle chip = TextStyle(
    fontFamily: 'Montserrat',
    fontSize: 10,
    letterSpacing: 1.4,
    fontWeight: FontWeight.w600,
    color: SynColors.textSubtle,
  );
}

class SynLayout {
  SynLayout._();

  static const double borderWidthHeavy = 3;
  static const double borderWidthNormal = 2;
  static const double borderWidthLight = 1;

  static const double paddingSmall = 8;
  static const double paddingMedium = 16;
  static const double paddingLarge = 24;
}

Color moodToColor(int mood) {
  final clamped = mood.clamp(-10, 10);
  const negative = Color(0xFFFF5A6A);
  const neutral = SynColors.primaryCyan;
  const positive = Color(0xFF54FFCF);
  if (clamped == 0) {
    return neutral;
  } else if (clamped > 0) {
    final t = clamped / 10.0;
    return Color.lerp(neutral, positive, t)!;
  } else {
    final t = clamped.abs() / 10.0;
    return Color.lerp(neutral, negative, t)!;
  }
}
