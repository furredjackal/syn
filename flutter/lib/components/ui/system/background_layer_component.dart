import 'dart:math';
import 'package:flame/components.dart';
import 'package:flutter/material.dart' hide Image;

/// Cyberpunk city background with procedural skyline
/// 
/// Architecture:
/// - Rendered directly on canvas (no parallax assets)
/// - Procedurally generated jagged skylines using Path
/// - Deep purple/black gradient sky
/// - Cyan/Magenta neon outlines
/// - Scrolling effect with multiple layers
class BackgroundLayerComponent extends PositionComponent {
  late final List<_CityLayer> _layers;
  
  @override
  Future<void> onLoad() async {
    priority = -10; // Behind everything
    
    // Initialize city layers with different speeds
    _layers = [
      _CityLayer(
        buildingHeightRange: (80, 150),
        buildingWidthRange: (40, 80),
        buildingCount: 15,
        baselineY: 0.7,
        outlineColor: const Color(0xFF6B4FBB).withValues(alpha: 0.6),
        fillColor: const Color(0xFF1A0F2E).withValues(alpha: 0.8),
        glowColor: const Color(0xFF9D6FFF),
        scrollSpeed: 5.0,
        seed: 42,
      ),
      _CityLayer(
        buildingHeightRange: (120, 220),
        buildingWidthRange: (50, 100),
        buildingCount: 12,
        baselineY: 0.75,
        outlineColor: const Color(0xFF00E6FF).withValues(alpha: 0.8),
        fillColor: const Color(0xFF0A0520).withValues(alpha: 0.9),
        glowColor: const Color(0xFF00E6FF),
        scrollSpeed: 10.0,
        seed: 137,
      ),
      _CityLayer(
        buildingHeightRange: (180, 300),
        buildingWidthRange: (60, 120),
        buildingCount: 10,
        baselineY: 0.8,
        outlineColor: const Color(0xFFFF006E).withValues(alpha: 0.9),
        fillColor: const Color(0xFF050014).withValues(alpha: 0.95),
        glowColor: const Color(0xFFFF006E),
        scrollSpeed: 15.0,
        seed: 999,
      ),
    ];
    
    // Generate building data for each layer
    for (final layer in _layers) {
      layer.generateBuildings();
    }
  }

  @override
  void onGameResize(Vector2 size) {
    super.onGameResize(size);
    this.size = size;
  }

  @override
  void update(double dt) {
    super.update(dt);
    for (final layer in _layers) {
      layer.update(dt);
    }
  }

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    
    // Draw gradient sky
    _drawSky(canvas);
    
    // Draw city layers
    for (final layer in _layers) {
      layer.render(canvas, size);
    }
  }

  void _drawSky(Canvas canvas) {
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    
    final gradient = LinearGradient(
      begin: Alignment.topCenter,
      end: Alignment.bottomCenter,
      colors: [
        const Color(0xFF000000), // Pure black at top
        const Color(0xFF0A0520), // Deep purple-black
        const Color(0xFF1A0F2E), // Lighter purple at horizon
      ],
      stops: const [0.0, 0.6, 1.0],
    );

    final paint = Paint()..shader = gradient.createShader(rect);
    canvas.drawRect(rect, paint);
  }
}

/// Represents one parallax layer of city buildings
class _CityLayer {
  final (double, double) buildingHeightRange;
  final (double, double) buildingWidthRange;
  final int buildingCount;
  final double baselineY;
  final Color outlineColor;
  final Color fillColor;
  final Color glowColor;
  final double scrollSpeed;
  final int seed;
  
  List<_Building> _buildings = [];
  double _offset = 0;
  double _totalWidth = 0;

  _CityLayer({
    required this.buildingHeightRange,
    required this.buildingWidthRange,
    required this.buildingCount,
    required this.baselineY,
    required this.outlineColor,
    required this.fillColor,
    required this.glowColor,
    required this.scrollSpeed,
    required this.seed,
  });

  void generateBuildings() {
    final random = Random(seed);
    double currentX = 0;

    for (int i = 0; i < buildingCount; i++) {
      final width = buildingWidthRange.$1 + 
          random.nextDouble() * (buildingWidthRange.$2 - buildingWidthRange.$1);
      final height = buildingHeightRange.$1 + 
          random.nextDouble() * (buildingHeightRange.$2 - buildingHeightRange.$1);
      
      _buildings.add(_Building(
        x: currentX,
        width: width,
        height: height,
        hasAntenna: random.nextBool(),
        windowRows: random.nextInt(8) + 3,
      ));
      
      currentX += width;
    }

    _totalWidth = currentX;
  }

  void update(double dt) {
    _offset += scrollSpeed * dt;
    // Loop when scrolled one full width
    if (_offset >= _totalWidth) {
      _offset -= _totalWidth;
    }
  }

  void render(Canvas canvas, Vector2 size) {
    final baselineYPos = size.y * baselineY;
    
    // Draw buildings, wrapping around for seamless scroll
    canvas.save();
    canvas.translate(-_offset, 0);
    
    // Draw twice to ensure seamless wrapping
    for (int pass = 0; pass < 2; pass++) {
      final offsetX = pass * _totalWidth;
      for (final building in _buildings) {
        _drawBuilding(canvas, building, offsetX, baselineYPos, size.y);
      }
    }
    
    canvas.restore();
  }

  void _drawBuilding(
    Canvas canvas,
    _Building building,
    double offsetX,
    double baselineY,
    double screenHeight,
  ) {
    final path = Path();
    final left = building.x + offsetX;
    final right = left + building.width;
    final top = baselineY - building.height;
    final bottom = screenHeight;

    // Building body (jagged top)
    path.moveTo(left, bottom);
    path.lineTo(left, top + 10);
    
    // Jagged roofline
    final roofSegments = 3;
    final segmentWidth = building.width / roofSegments;
    for (int i = 0; i < roofSegments; i++) {
      final x = left + (i + 0.5) * segmentWidth;
      final y = top + (i % 2 == 0 ? 0 : 8);
      path.lineTo(x, y);
    }
    
    path.lineTo(right, top + 10);
    path.lineTo(right, bottom);
    path.close();

    // Draw fill
    final fillPaint = Paint()
      ..color = fillColor
      ..style = PaintingStyle.fill;
    canvas.drawPath(path, fillPaint);

    // Draw glow (outer outline)
    final glowPaint = Paint()
      ..color = glowColor.withValues(alpha: 0.2)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 3
      ..maskFilter = const MaskFilter.blur(BlurStyle.outer, 6);
    canvas.drawPath(path, glowPaint);

    // Draw outline
    final outlinePaint = Paint()
      ..color = outlineColor
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.5;
    canvas.drawPath(path, outlinePaint);

    // Draw antenna if present
    if (building.hasAntenna) {
      final antennaX = left + building.width / 2;
      final antennaHeight = 20.0;
      final antennaPaint = Paint()
        ..color = outlineColor
        ..strokeWidth = 1.5;
      
      canvas.drawLine(
        Offset(antennaX, top),
        Offset(antennaX, top - antennaHeight),
        antennaPaint,
      );
      
      // Antenna light
      final lightPaint = Paint()
        ..color = glowColor
        ..style = PaintingStyle.fill
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 4);
      canvas.drawCircle(
        Offset(antennaX, top - antennaHeight),
        3,
        lightPaint,
      );
    }

    // Draw windows
    _drawWindows(canvas, building, left, top, baselineY);
  }

  void _drawWindows(
    Canvas canvas,
    _Building building,
    double left,
    double top,
    double baselineY,
  ) {
    final windowWidth = 4.0;
    final windowHeight = 6.0;
    final windowSpacingX = 12.0;
    final windowSpacingY = 15.0;
    
    final windowPaint = Paint()
      ..color = glowColor.withValues(alpha: 0.6)
      ..style = PaintingStyle.fill;

    final cols = (building.width / windowSpacingX).floor();
    final rows = building.windowRows;

    for (int row = 0; row < rows; row++) {
      for (int col = 0; col < cols; col++) {
        // Randomly skip some windows
        if (Random(seed + row * 100 + col).nextDouble() > 0.7) continue;
        
        final x = left + (col + 1) * windowSpacingX;
        final y = top + 20 + row * windowSpacingY;
        
        if (y < baselineY - 10) {
          canvas.drawRect(
            Rect.fromLTWH(x, y, windowWidth, windowHeight),
            windowPaint,
          );
        }
      }
    }
  }
}

/// Building data structure
class _Building {
  final double x;
  final double width;
  final double height;
  final bool hasAntenna;
  final int windowRows;

  _Building({
    required this.x,
    required this.width,
    required this.height,
    required this.hasAntenna,
    required this.windowRows,
  });
}
