import 'package:flutter/material.dart'
    show Color, TextStyle, FontStyle, FontWeight;

/// Defines the visual and behavioral profile for each life stage.
/// Customizes geometry, animation speed, particle behavior, and aesthetic elements.
class LifeStageProfile {
  final String stage;
  final double cornerRadius;
  final double skewAngle;
  final Color primaryColor;
  final Color accentColor;
  final double saturation;
  final double animationSpeedMultiplier;
  final double bounceAmount;
  final String particleType;
  final double particleEmissionRate;
  final TextStyle headingStyle;
  final TextStyle bodyStyle;
  final double iconScale;

  LifeStageProfile({
    required this.stage,
    required this.cornerRadius,
    required this.skewAngle,
    required this.primaryColor,
    required this.accentColor,
    required this.saturation,
    required this.animationSpeedMultiplier,
    required this.bounceAmount,
    required this.particleType,
    required this.particleEmissionRate,
    required this.headingStyle,
    required this.bodyStyle,
    required this.iconScale,
  });

  static LifeStageProfile child() {
    return LifeStageProfile(
      stage: 'Child',
      cornerRadius: 4.0,
      skewAngle: 0.0,
      primaryColor: const Color(0xFF00D9FF),
      accentColor: const Color(0xFFFFD700),
      saturation: 1.3,
      animationSpeedMultiplier: 1.1,
      bounceAmount: 0.2,
      particleType: 'sparkles',
      particleEmissionRate: 12.0,
      headingStyle: TextStyle(
        fontSize: 28,
        fontWeight: FontWeight.w900,
        letterSpacing: 2,
      ),
      bodyStyle: TextStyle(
        fontSize: 14,
        letterSpacing: 1,
      ),
      iconScale: 1.1,
    );
  }

  static LifeStageProfile teen() {
    return LifeStageProfile(
      stage: 'Teen',
      cornerRadius: 1.0,
      skewAngle: 12.0,
      primaryColor: const Color(0xFF9D4EDD),
      accentColor: const Color(0xFF3A86FF),
      saturation: 1.2,
      animationSpeedMultiplier: 1.15,
      bounceAmount: 0.15,
      particleType: 'lightning',
      particleEmissionRate: 15.0,
      headingStyle: TextStyle(
        fontSize: 32,
        fontWeight: FontWeight.w900,
        letterSpacing: 3,
        fontStyle: FontStyle.italic,
      ),
      bodyStyle: TextStyle(
        fontSize: 15,
        letterSpacing: 1.5,
        fontStyle: FontStyle.italic,
      ),
      iconScale: 1.0,
    );
  }

  static LifeStageProfile adult() {
    return LifeStageProfile(
      stage: 'Adult',
      cornerRadius: 6.0,
      skewAngle: 0.0,
      primaryColor: const Color(0xFF0099CC),
      accentColor: const Color(0xFF00D9FF),
      saturation: 1.0,
      animationSpeedMultiplier: 1.0,
      bounceAmount: 0.05,
      particleType: 'coins',
      particleEmissionRate: 5.0,
      headingStyle: TextStyle(
        fontSize: 26,
        fontWeight: FontWeight.w700,
        letterSpacing: 2,
      ),
      bodyStyle: TextStyle(
        fontSize: 14,
        letterSpacing: 0.5,
      ),
      iconScale: 1.0,
    );
  }

  static LifeStageProfile elder() {
    return LifeStageProfile(
      stage: 'Elder',
      cornerRadius: 12.0,
      skewAngle: 0.0,
      primaryColor: const Color(0xFFD4A574),
      accentColor: const Color(0xFFF4A460),
      saturation: 0.9,
      animationSpeedMultiplier: 0.8,
      bounceAmount: 0.0,
      particleType: 'leaves',
      particleEmissionRate: 3.0,
      headingStyle: TextStyle(
        fontSize: 24,
        fontWeight: FontWeight.w700,
        letterSpacing: 2,
      ),
      bodyStyle: TextStyle(
        fontSize: 16,
        letterSpacing: 0.5,
      ),
      iconScale: 1.1,
    );
  }

  static LifeStageProfile digital() {
    return LifeStageProfile(
      stage: 'Digital',
      cornerRadius: 0.0,
      skewAngle: 0.0,
      primaryColor: const Color(0xFF00FF00),
      accentColor: const Color(0xFFFFFFFF),
      saturation: 0.7,
      animationSpeedMultiplier: 1.25,
      bounceAmount: 0.1,
      particleType: 'data_streams',
      particleEmissionRate: 20.0,
      headingStyle: TextStyle(
        fontSize: 20,
        fontWeight: FontWeight.w900,
        letterSpacing: 4,
        fontFamily: 'monospace',
      ),
      bodyStyle: TextStyle(
        fontSize: 12,
        letterSpacing: 2,
        fontFamily: 'monospace',
      ),
      iconScale: 0.9,
    );
  }

  static LifeStageProfile forStage(String stageName) {
    switch (stageName.toLowerCase()) {
      case 'child':
        return child();
      case 'teen':
        return teen();
      case 'adult':
        return adult();
      case 'elder':
        return elder();
      case 'digital':
        return digital();
      default:
        return adult();
    }
  }
}
