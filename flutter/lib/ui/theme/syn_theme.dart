import 'package:flutter/material.dart';

/// SYN Design System - Persona 5 × Destiny 2 inspired
///
/// Core principles:
/// - BOLD: High contrast, aggressive typography, dramatic reveals
/// - FLUID: Buttery smooth 60fps animations, physics-based motion
/// - KINETIC: Everything responds to interaction, nothing is static
/// - LAYERED: Depth through parallax, shadows, and blur

class SynTheme {
  SynTheme._();

  // ==================== COLORS ====================

  /// Primary accent - electric cyan
  static const Color accent = Color(0xFF00E6FF);

  /// Secondary accent - hot magenta (for warnings/heat)
  static const Color accentHot = Color(0xFFFF0066);

  /// Tertiary accent - electric yellow (for highlights)
  static const Color accentWarm = Color(0xFFFFE600);

  /// Pure black background
  static const Color bgBlack = Color(0xFF000000);

  /// Elevated surface
  static const Color bgSurface = Color(0xFF0A0A0A);

  /// Card background
  static const Color bgCard = Color(0xFF121212);

  /// Muted text
  static const Color textMuted = Color(0xFF666666);

  /// Secondary text
  static const Color textSecondary = Color(0xFFAAAAAA);

  /// Primary text
  static const Color textPrimary = Color(0xFFFFFFFF);

  // ==================== GRADIENTS ====================

  /// Accent glow gradient
  static const LinearGradient accentGlow = LinearGradient(
    colors: [Color(0xFF00E6FF), Color(0xFF0080FF)],
    begin: Alignment.topLeft,
    end: Alignment.bottomRight,
  );

  /// Hot/danger gradient
  static const LinearGradient hotGlow = LinearGradient(
    colors: [Color(0xFFFF0066), Color(0xFFFF6600)],
    begin: Alignment.topLeft,
    end: Alignment.bottomRight,
  );

  /// Dark vignette for depth
  static RadialGradient get vignette => RadialGradient(
        colors: [Colors.transparent, Colors.black.withOpacity(0.7)],
        stops: const [0.5, 1.0],
      );

  // ==================== SHADOWS ====================

  /// Subtle glow shadow
  static List<BoxShadow> glowShadow(Color color, {double intensity = 0.4}) => [
        BoxShadow(
          color: color.withOpacity(intensity),
          blurRadius: 20,
          spreadRadius: -2,
        ),
      ];

  /// Hard drop shadow (Persona style)
  static List<BoxShadow> get hardShadow => [
        const BoxShadow(
          color: Colors.black,
          offset: Offset(6, 6),
          blurRadius: 0,
        ),
      ];

  /// Combined glow + hard shadow
  static List<BoxShadow> dramaticShadow(Color glowColor) => [
        BoxShadow(
          color: glowColor.withOpacity(0.3),
          blurRadius: 15,
          spreadRadius: -3,
        ),
        const BoxShadow(
          color: Colors.black,
          offset: Offset(4, 4),
          blurRadius: 0,
        ),
      ];

  // ==================== TYPOGRAPHY ====================

  /// Display - huge impact text
  static TextStyle display({Color? color}) => TextStyle(
        fontSize: 64,
        fontWeight: FontWeight.w900,
        color: color ?? textPrimary,
        letterSpacing: 8,
        height: 0.9,
      );

  /// Headline - section headers
  static TextStyle headline({Color? color}) => TextStyle(
        fontSize: 32,
        fontWeight: FontWeight.w900,
        color: color ?? textPrimary,
        letterSpacing: 4,
      );

  /// Title - card titles
  static TextStyle title({Color? color}) => TextStyle(
        fontSize: 20,
        fontWeight: FontWeight.w700,
        color: color ?? textPrimary,
        letterSpacing: 2,
      );

  /// Label - buttons, tabs
  static TextStyle label({Color? color}) => TextStyle(
        fontSize: 14,
        fontWeight: FontWeight.w700,
        color: color ?? textPrimary,
        letterSpacing: 1.5,
      );

  /// Body - readable text
  static TextStyle body({Color? color}) => TextStyle(
        fontSize: 16,
        fontWeight: FontWeight.w400,
        color: color ?? textSecondary,
        height: 1.5,
      );

  /// Caption - small metadata
  static TextStyle caption({Color? color}) => TextStyle(
        fontSize: 12,
        fontWeight: FontWeight.w500,
        color: color ?? textMuted,
        letterSpacing: 1,
      );

  // ==================== ANIMATION CURVES ====================

  /// Snappy entrance (Destiny 2 style)
  static const Curve snapIn = Curves.easeOutExpo;

  /// Smooth exit
  static const Curve snapOut = Curves.easeInExpo;

  /// Bouncy overshoot
  static const Curve bounce = Curves.elasticOut;

  /// Dramatic reveal
  static const Curve dramatic = Curves.easeOutBack;

  // ==================== ANIMATION DURATIONS ====================

  /// Instant micro-interaction
  static const Duration instant = Duration(milliseconds: 100);

  /// Fast response
  static const Duration fast = Duration(milliseconds: 200);

  /// Standard transition
  static const Duration normal = Duration(milliseconds: 350);

  /// Dramatic reveal
  static const Duration slow = Duration(milliseconds: 500);

  /// Epic entrance
  static const Duration epic = Duration(milliseconds: 800);

  // ==================== GEOMETRY ====================

  /// Standard skew angle for Persona aesthetic
  static const double skewAngle = -0.12;

  /// Aggressive skew for emphasis
  static const double skewHeavy = -0.18;

  /// Subtle skew for balance
  static const double skewLight = -0.06;

  /// Standard border radius
  static const double radius = 0; // Sharp corners are more Persona

  /// Border width
  static const double borderWidth = 2.0;

  /// Heavy border
  static const double borderHeavy = 3.0;
}

/// Extension for easy color manipulation
extension SynColorExt on Color {
  /// Create a glow version of this color
  Color get glow => withOpacity(0.4);

  /// Create a dim version of this color
  Color get dim => withOpacity(0.2);

  /// Create a muted version
  Color get muted => Color.lerp(this, Colors.grey, 0.5)!;
}

// =============================================================================
// UI GRAMMAR SYSTEM
// =============================================================================
// "Once this grammar exists, the UI immediately feels intentional—even if sparse."

/// Shape grammar for panel hierarchy
enum SynShapeGrammar {
  /// Primary panels: hard rectangles, clipped corners
  primary,

  /// Secondary panels: slanted edge or asymmetric notch
  secondary,

  /// Focus elements: brackets, diamonds, or ticks (not glows)
  focus,
}

/// Line grammar rules
/// - Lines never end flush—always overshoot or break
/// - All dividers are either 1px solid OR 2px dashed (never mix)
class SynLineGrammar {
  SynLineGrammar._();

  /// Solid line width (primary)
  static const double solidWidth = 1.0;

  /// Dashed line width (secondary)
  static const double dashedWidth = 2.0;

  /// Overshoot amount for line terminals
  static const double overshoot = 4.0;

  /// Dash length for dashed lines
  static const double dashLength = 4.0;

  /// Gap length for dashed lines
  static const double dashGap = 4.0;

  /// Bracket arm length
  static const double bracketSize = 20.0;

  /// Tracking line opacity
  static const double trackingOpacity = 0.15;
}

/// Color grammar rules
/// - One accent color only (SYN cyan)
/// - All alerts derive from opacity, not hue shifts
/// - Red is rare and contextual, not decorative
class SynColorGrammar {
  SynColorGrammar._();

  /// Primary accent (the ONLY accent color for most UI)
  static const Color accent = SynTheme.accent;

  /// Alert color (use sparingly, contextual only)
  static const Color alert = SynTheme.accentHot;

  /// Highlight color (temporary emphasis)
  static const Color highlight = SynTheme.accentWarm;

  /// Standard opacity levels
  static const double opacityFull = 1.0;
  static const double opacityStrong = 0.8;
  static const double opacityMedium = 0.5;
  static const double opacitySubtle = 0.3;
  static const double opacityGhost = 0.1;
  static const double opacityInstrumentation = 0.03;

  /// Derive alert state from opacity rather than color
  static Color withAlertLevel(double level) {
    // level 0.0 = normal, 1.0 = critical
    if (level < 0.5) {
      return accent.withOpacity(opacityStrong);
    } else if (level < 0.8) {
      return accent.withOpacity(opacityFull);
    } else {
      // Only at critical levels do we shift to alert color
      return alert.withOpacity(opacityFull);
    }
  }
}

/// UI Station types for contextual styling
/// "Stop thinking 'screen' — think 'stations'"
enum SynStation {
  /// Observation Station – read-only world state
  observation,

  /// Intervention Station – make changes
  intervention,

  /// Reflection Station – memory, legacy, stats
  reflection,

  /// Transit State – transitions, time passing
  transit,
}

/// Station-specific styling
extension SynStationStyle on SynStation {
  /// Get the emphasis color for this station
  Color get emphasisColor {
    switch (this) {
      case SynStation.observation:
        return SynTheme.accent; // Calm cyan
      case SynStation.intervention:
        return SynTheme.accentWarm; // Active yellow
      case SynStation.reflection:
        return SynTheme.accent.withOpacity(0.7); // Muted cyan
      case SynStation.transit:
        return SynTheme.accent.withOpacity(0.5); // Faded
    }
  }

  /// Get the border style for this station
  double get borderWidth {
    switch (this) {
      case SynStation.observation:
        return SynLineGrammar.solidWidth;
      case SynStation.intervention:
        return SynLineGrammar.dashedWidth;
      case SynStation.reflection:
        return SynLineGrammar.solidWidth;
      case SynStation.transit:
        return 0.0; // No border during transit
    }
  }

  /// Whether to show brackets on focus
  bool get showBrackets {
    switch (this) {
      case SynStation.observation:
        return true;
      case SynStation.intervention:
        return true;
      case SynStation.reflection:
        return false;
      case SynStation.transit:
        return false;
    }
  }

  /// Get the skew angle for this station
  double get skewAngle {
    switch (this) {
      case SynStation.observation:
        return SynTheme.skewLight;
      case SynStation.intervention:
        return SynTheme.skewAngle;
      case SynStation.reflection:
        return SynTheme.skewHeavy;
      case SynStation.transit:
        return 0.0; // No skew during transit
    }
  }
}

/// Instrumentation layer constants
class SynInstrumentation {
  SynInstrumentation._();

  /// Scanline sweep duration
  static const Duration scanlineDuration = Duration(seconds: 12);

  /// Scanline opacity
  static const double scanlineOpacity = 0.03;

  /// Grid cell size
  static const double gridCellSize = 50.0;

  /// Grid base opacity
  static const double gridOpacity = 0.02;

  /// Grid hover multiplier
  static const double gridHoverMultiplier = 2.0;

  /// Bracket pulse duration
  static const Duration bracketPulseDuration = Duration(seconds: 3);

  /// Idle motion cycle
  static const Duration idleCycleDuration = Duration(seconds: 10);

  /// Maximum idle drift in pixels
  static const double idleMaxDrift = 3.0;

  /// Micro-labels for instrumentation HUD
  static const List<String> microLabels = ['SIM', 'MEM', 'REL', 'LOD', 'STATE'];
}
