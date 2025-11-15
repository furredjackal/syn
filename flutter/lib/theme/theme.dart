import 'package:flutter/material.dart';
import 'package:google_fonts/google_fonts.dart';

/// Color scheme based on mood state (-10 to +10)
class MoodColors {
  static const Color despair = Color(0xFF2D0A4E); // Deep purple + black
  static const Color troubled = Color(0xFF1A3A52); // Dark blue
  static const Color neutral = Color(0xFF0F5F6F); // Balanced teal
  static const Color content = Color(0xFF7A5D2F); // Warm yellow
  static const Color euphoric = Color(0xFFFFD700); // Vibrant gold

  static Color forMood(double mood) {
    if (mood <= -6) return despair;
    if (mood <= -1) return troubled;
    if (mood <= 1) return neutral;
    if (mood <= 5) return content;
    return euphoric;
  }

  static Color moodGradientLight(double mood) {
    if (mood <= -6) return Color(0xFF4A1A7A);
    if (mood <= -1) return Color(0xFF2D5A7A);
    if (mood <= 1) return Color(0xFF1A7F8F);
    if (mood <= 5) return Color(0xFFA87F3F);
    return Color(0xFFFFE55C);
  }
}

/// Karma colors
class KarmaColors {
  static const Color highPositive = Color(0xFFF0F0F0); // Ethereal white
  static const Color neutral = Color(0xFF888888); // Standard grey
  static const Color highNegative = Color(0xFFCC0000); // Deep red

  static Color forKarma(int karma) {
    if (karma > 60) return highPositive;
    if (karma < -60) return highNegative;
    return neutral;
  }
}

/// Life stage themes
class LifeStageTheme {
  final String name;
  final Color primaryColor;
  final Color secondaryColor;
  final String badge;

  const LifeStageTheme({
    required this.name,
    required this.primaryColor,
    required this.secondaryColor,
    required this.badge,
  });

  static final child = LifeStageTheme(
    name: 'Child',
    primaryColor: Color(0xFF00D9FF), // Cyan
    secondaryColor: Color(0xFFFFD700), // Yellow
    badge: '👶',
  );

  static final teen = LifeStageTheme(
    name: 'Teen',
    primaryColor: Color(0xFF9D4EDD), // Purple
    secondaryColor: Color(0xFFFF006E), // Pink
    badge: '🎓',
  );

  static final adult = LifeStageTheme(
    name: 'Adult',
    primaryColor: Color(0xFF3A86FF), // Blue
    secondaryColor: Color(0xFF8338EC), // Purple
    badge: '💼',
  );

  static final elder = LifeStageTheme(
    name: 'Elder',
    primaryColor: Color(0xFFB8860B), // Gold
    secondaryColor: Color(0xFF8B7355), // Brown
    badge: '🧙',
  );

  static final digital = LifeStageTheme(
    name: 'Digital',
    primaryColor: Color(0xFF00D9FF), // Cyan
    secondaryColor: Color(0xFFFFFFFF), // White
    badge: '🤖',
  );

  static LifeStageTheme fromStage(String stage) {
    switch (stage.toLowerCase()) {
      case 'child':
        return child;
      case 'teen':
        return teen;
      case 'elder':
        return elder;
      case 'digital':
        return digital;
      case 'adult':
      default:
        return adult;
    }
  }
}

/// SYN Typography
class SynTypography {
  static TextStyle get titleLarge {
    return GoogleFonts.audiowide(
      fontSize: 48,
      fontWeight: FontWeight.w900,
      letterSpacing: 4,
      color: Colors.white,
    );
  }

  static TextStyle get titleMedium {
    return GoogleFonts.audiowide(
      fontSize: 32,
      fontWeight: FontWeight.w900,
      letterSpacing: 3,
      color: Colors.white,
    );
  }

  static TextStyle get titleSmall {
    return GoogleFonts.audiowide(
      fontSize: 24,
      fontWeight: FontWeight.w700,
      letterSpacing: 2,
      color: Colors.white,
    );
  }

  static TextStyle get bodyLarge {
    return GoogleFonts.roboto(
      fontSize: 18,
      fontWeight: FontWeight.w400,
      color: Colors.white.withOpacity(0.9),
    );
  }

  static TextStyle get bodyMedium {
    return GoogleFonts.roboto(
      fontSize: 16,
      fontWeight: FontWeight.w400,
      color: Colors.white.withOpacity(0.85),
    );
  }

  static TextStyle get bodySmall {
    return GoogleFonts.roboto(
      fontSize: 14,
      fontWeight: FontWeight.w400,
      color: Colors.white.withOpacity(0.75),
    );
  }

  static TextStyle get label {
    return GoogleFonts.roboto(
      fontSize: 12,
      fontWeight: FontWeight.w600,
      letterSpacing: 1,
      color: Colors.white.withOpacity(0.9),
    );
  }

  static TextStyle get labelMedium {
    return label;
  }
}

/// SYN Theme
class SynTheme {
  static final ThemeData dark = ThemeData(
    useMaterial3: true,
    brightness: Brightness.dark,
    scaffoldBackgroundColor: Color(0xFF0A0E27),
    primaryColor: Color(0xFF00D9FF),
    secondaryHeaderColor: Color(0xFF9D4EDD),
    fontFamily: GoogleFonts.roboto().fontFamily,
    textTheme: TextTheme(
      displayLarge: SynTypography.titleLarge,
      displayMedium: SynTypography.titleMedium,
      displaySmall: SynTypography.titleSmall,
      bodyLarge: SynTypography.bodyLarge,
      bodyMedium: SynTypography.bodyMedium,
      bodySmall: SynTypography.bodySmall,
      labelMedium: SynTypography.label,
      labelSmall: SynTypography.label,
    ),
    canvasColor: Color(0xFF1A1F3A),
    cardColor: Color(0xFF15192E),
    dividerColor: Color(0xFF00D9FF).withOpacity(0.3),
  );
}

/// Extension to provide .label getter for backward compatibility
extension TextThemeExt on TextTheme {
  TextStyle? get label => labelMedium;
}
