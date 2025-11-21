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

/// HUD chrome tokens for bars and overlays.
class SynHudChrome {
  SynHudChrome._();

  static const LinearGradient topBarBackgroundGradient = LinearGradient(
    colors: [Color(0xFF05050A), Color(0xFF0A0F1F)],
    begin: Alignment.topLeft,
    end: Alignment.bottomRight,
  );

  static const Color topBarBorderColorPrimary = Color(0xFF00E6FF);
  static const Color topBarBorderFlashColor = Color(0xFF9A27FF);
  static const double topBarBorderWidth = 3.0;
  static const Color topBarInnerStrokeColor = Color(0x5500E6FF);
  static const double topBarInnerStrokeWidth = 1.2;

  static const Color topBarShadowColor = Color(0x99000000);
  static const double topBarShadowBlur = 14;

  static const Color topBarSlashCyanColor = Color(0xFF00E6FF);
  static const double topBarSlashCyanOpacity = 0.20;
  static const Color topBarSlashPurpleColor = Color(0xFF9A27FF);
  static const double topBarSlashPurpleOpacity = 0.16;

  static const TextStyle topBarLabelTextStyle = TextStyle(
    fontFamily: 'Montserrat',
    fontSize: 12,
    letterSpacing: 1.5,
    fontWeight: FontWeight.w800,
    color: Color(0xFFF6F6F6),
  );

  static const double topBarCornerInset = 18.0;
  static const double topBarEdgeBend = 0.62;

  static const Color topBarButtonFill = Color(0xFF10192A);
  static const Color topBarButtonHoverFill = Color(0xFF142138);
  static const Color topBarButtonPressedFill = Color(0xFF1C2B45);
  static const Color topBarButtonStrokeColor = Color(0x8000E6FF);
  static const double topBarButtonStrokeWidth = 1.6;
  static const double topBarButtonHoverScale = 1.04;
  static const double topBarButtonHoverDuration = 0.12;
}

/// Top bar specific tokens for the HUD strip.
class SynTopBar {
  SynTopBar._();

  static const Color backgroundColor = Color(0xFF05060A);
  static const Color backgroundSheenColor = Color(0xFF0A0C14);
  static const Color slashOverlayColor = Color(0xFF00E6FF);
  static const double slashOverlayOpacity = 0.08;
  static const Color ambientGlowColor = Color(0x3300E6FF);
  static const Color shadowColor = Color(0x99000000);
  static const double shadowBlur = 14;

  static const double height = 92.0;
  static const double heightFraction = 0.12;

  static const TextStyle textPrimaryStyle = TextStyle(
    fontFamily: 'Montserrat',
    fontSize: 13,
    letterSpacing: 1.6,
    fontWeight: FontWeight.w800,
    color: Color(0xFFF6F6F6),
  );
  static const Color textHoverColor = Color(0xFF00E6FF);
  static const Color textActiveColor = Color(0xFF9A27FF);
  static const double textGlowBlur = 8.0;

  static const double actionHoverScale = 1.05;
  static const double actionHoverDuration = 0.12;
  static const double actionPressScale = 0.98;

  static const double stageFlashOpacity = 0.45;
  static const double stageFlashBlur = 18.0;

  static const Color ageLabelColor = Color(0xFFB8C2D6);
  static const Color ageValueColor = Color(0xFFF6F6F6);
  static const TextStyle ageLabelStyle = TextStyle(
    fontFamily: 'Montserrat',
    fontSize: 10,
    letterSpacing: 1.3,
    fontWeight: FontWeight.w700,
    color: ageLabelColor,
  );
  static const TextStyle ageValueStyle = TextStyle(
    fontFamily: 'Montserrat',
    fontSize: 18,
    letterSpacing: 1.0,
    fontWeight: FontWeight.w800,
    color: ageValueColor,
  );
  static const TextStyle ageInlineStyle = TextStyle(
    fontFamily: 'Montserrat',
    fontSize: 14,
    letterSpacing: 1.1,
    fontWeight: FontWeight.w700,
    color: ageValueColor,
  );
  static const Color ageAccentColor = Color(0xFF00E6FF);

  static const TextStyle lifeStageTextStyle = TextStyle(
    fontFamily: 'Montserrat',
    fontSize: 14,
    letterSpacing: 1.3,
    fontWeight: FontWeight.w800,
    color: Color(0xFFF6F6F6),
  );
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
