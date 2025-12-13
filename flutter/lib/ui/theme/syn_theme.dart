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
