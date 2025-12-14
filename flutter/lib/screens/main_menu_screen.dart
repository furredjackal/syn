import 'dart:math' as math; // Now we use this!
import 'dart:ui';
import 'package:flutter/material.dart';
import 'package:flutter/scheduler.dart';
import 'package:flutter/services.dart';
import 'package:flutter_animate/flutter_animate.dart';

class MainMenuScreen extends StatefulWidget {
  final VoidCallback onNewGame;
  final VoidCallback onLoadGame;
  final VoidCallback onSettings;
  final VoidCallback onQuit;

  const MainMenuScreen({
    super.key,
    required this.onNewGame,
    required this.onLoadGame,
    required this.onSettings,
    required this.onQuit,
  });

  @override
  State<MainMenuScreen> createState() => _MainMenuScreenState();
}

class _MainMenuScreenState extends State<MainMenuScreen>
    with TickerProviderStateMixin {
  // Mouse position tracking (Normalized -1.0 to 1.0)
  final ValueNotifier<Offset> _mousePos = ValueNotifier(Offset.zero);
  
  // New: Controller for infinite, math-driven background motion (The Sine Wave)
  late AnimationController _oscillationController;
  
  int _selectedIndex = -1;

  final List<String> _menuLabels = [
    'INFILTRATE',
    'MEMORIES',
    'PROTOCOL',
    'ESCAPE'
  ];

  @override
  void initState() {
    super.initState();
    // 1. Set up the oscillation controller (Slow, continuous motion)
    _oscillationController = AnimationController(
      vsync: this,
      duration: const Duration(seconds: 15), // Very slow cycle
    )..repeat();
  }

  @override
  void dispose() {
    _oscillationController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      cursor: SystemMouseCursors.none,
      onHover: (event) {
        final size = MediaQuery.of(context).size;
        final x = (event.position.dx / size.width) * 2 - 1;
        final y = (event.position.dy / size.height) * 2 - 1;
        _mousePos.value = Offset(x, y);
      },
      child: Scaffold(
        backgroundColor: const Color(0xFF0F0F12),
        body: Stack(
          children: [
            // --- PARALLAX & OSCILLATION LAYERS ---
            
            // 1. Deep Background (Moves slowly based on mouse + oscillation)
            _MathLayer(
              mousePos: _mousePos,
              oscillation: _oscillationController,
              mouseSpeed: 15,
              oscillationSpeed: 5, // Small background drift
              child: Container(
                decoration: const BoxDecoration(
                  gradient: RadialGradient(
                    center: Alignment.topRight,
                    radius: 1.5,
                    colors: [Color(0xFF2A2A35), Color(0xFF000000)],
                  ),
                ),
              ),
            ),

            // 2. Geometric Shapes (Moves medium speed based on mouse + oscillation)
            _MathLayer(
              mousePos: _mousePos,
              oscillation: _oscillationController,
              mouseSpeed: 40,
              oscillationSpeed: 10, // Medium geometric drift
              inverse: true,
              child: Stack(
                children: [
                  // ... (Geometric shapes content remains the same) ...
                  Positioned(
                    right: -100,
                    top: -100,
                    child: Transform.rotate(
                      angle: 0.2,
                      child: Container(
                        width: 600,
                        height: 600,
                        decoration: BoxDecoration(
                          border: Border.all(
                            color: Colors.white.withOpacity(0.03), 
                            width: 1
                          ),
                        ),
                      ),
                    ),
                  ),
                  Positioned(
                    left: 100,
                    bottom: 100,
                    child: Container(
                      width: 2,
                      height: 300,
                      color: Colors.cyanAccent.withOpacity(0.2),
                    ),
                  ),
                ],
              ),
            ),

            // --- MAIN MENU CONTENT ---
            
            Positioned(
              left: 100,
              top: 0,
              bottom: 0,
              child: Center(
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: List.generate(_menuLabels.length, (index) {
                    return _buildMathMenuItem(index);
                  }),
                ),
              ),
            ),
            
            // --- SMOOTH CURSOR OVERLAY ---
            _SmoothCursor(mousePos: _mousePos),
          ],
        ),
      ),
    );
  }

  // New menu item builder using oscillation
  Widget _buildMathMenuItem(int index) {
    final isHovered = _selectedIndex == index;

    return MouseRegion(
      onEnter: (_) => setState(() => _selectedIndex = index),
      onExit: (_) => setState(() => _selectedIndex = -1),
      child: GestureDetector(
        onTap: () {
          // Trigger the action based on the index
          switch (index) {
            case 0: widget.onNewGame(); break;
            case 1: widget.onLoadGame(); break;
            case 2: widget.onSettings(); break;
            case 3: widget.onQuit(); break;
          }
        },
        child: AnimatedBuilder(
          animation: _oscillationController,
          builder: (context, child) {
            // MATH: Use Sine wave for a gentle, looping float on the menu item
            // The phase shift (index * 0.5) ensures each item moves independently.
            final wave = math.sin(_oscillationController.value * 2 * math.pi + index * 0.5);
            
            // Horizontal oscillation magnitude (10 pixels left/right)
            final oscillateX = wave * 10; 

            // Combine Hover slide (40px) with Oscillation (up to 10px)
            final combinedX = isHovered ? 40.0 + oscillateX : 0.0 + oscillateX;

            return AnimatedContainer(
              duration: 300.ms,
              curve: Curves.easeOutQuart,
              margin: const EdgeInsets.symmetric(vertical: 12),
              transform: Matrix4.identity()
                ..translate(combinedX, 0.0), // Combined motion
              child: child,
            );
          },
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              // The "Diamond" Bullet
              AnimatedContainer(
                duration: 200.ms,
                width: isHovered ? 12 : 0,
                height: 12,
                margin: const EdgeInsets.only(right: 16),
                transform: Matrix4.rotationZ(0.785), // 45 degrees
                decoration: const BoxDecoration(color: Colors.cyanAccent),
              ),
              
              // The Text
              Text(
                _menuLabels[index],
                style: TextStyle(
                  fontFamily: 'Roboto',
                  fontSize: 56,
                  fontWeight: FontWeight.w900,
                  fontStyle: FontStyle.italic,
                  letterSpacing: 2,
                  color: isHovered ? Colors.white : Colors.white.withOpacity(0.4),
                  shadows: isHovered ? [
                     Shadow(color: Colors.cyanAccent.withOpacity(0.6), blurRadius: 20)
                  ] : [],
                ),
              ).animate(target: isHovered ? 1 : 0)
               .shimmer(duration: 1.seconds, color: Colors.cyanAccent),
            ],
          ),
        ),
      ),
    );
  }
}

// --- HELPERS (Updated to include Oscillation) ---

/// Combines Mouse Parallax and Sine-wave Oscillation (Math Layer)
class _MathLayer extends StatelessWidget {
  final ValueNotifier<Offset> mousePos;
  final AnimationController oscillation;
  final Widget child;
  final double mouseSpeed;
  final double oscillationSpeed;
  final bool inverse;

  const _MathLayer({
    required this.mousePos,
    required this.oscillation,
    required this.child,
    this.mouseSpeed = 20,
    this.oscillationSpeed = 10,
    this.inverse = false,
  });

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: Listenable.merge([mousePos, oscillation]),
      builder: (context, _) {
        // MATH: Parallax based on mouse
        final parallaxX = mousePos.value.dx * mouseSpeed * (inverse ? -1 : 1);
        final parallaxY = mousePos.value.dy * mouseSpeed * (inverse ? -1 : 1);
        
        // MATH: Sine wave for oscillation
        // This causes the background to perpetually drift back and forth
        final driftX = math.sin(oscillation.value * 2 * math.pi) * oscillationSpeed;
        final driftY = math.cos(oscillation.value * 2 * math.pi) * oscillationSpeed; // Use cosine for a circular effect

        // Combine both movements
        return Transform.translate(
          offset: Offset(parallaxX + driftX, parallaxY + driftY),
          child: child,
        );
      },
    );
  }
}

// --- SMOOTH CURSOR (Remains the same as it already uses lerp/physics) ---

class _SmoothCursor extends StatefulWidget {
  final ValueNotifier<Offset> mousePos;

  const _SmoothCursor({required this.mousePos});

  @override
  State<_SmoothCursor> createState() => _SmoothCursorState();
}

class _SmoothCursorState extends State<_SmoothCursor> with SingleTickerProviderStateMixin {
  Offset _currentPos = Offset.zero;
  Offset _targetPos = Offset.zero;
  late Ticker _ticker;

  @override
  void initState() {
    super.initState();
    _ticker = createTicker(_tick)..start();
    
    widget.mousePos.addListener(() {
      final size = MediaQuery.of(context).size;
      final x = (widget.mousePos.value.dx + 1) / 2 * size.width;
      final y = (widget.mousePos.value.dy + 1) / 2 * size.height;
      _targetPos = Offset(x, y);
    });
  }

  void _tick(Duration elapsed) {
    const double friction = 0.15;
    final dx = lerpDouble(_currentPos.dx, _targetPos.dx, friction) ?? 0;
    final dy = lerpDouble(_currentPos.dy, _targetPos.dy, friction) ?? 0;
    
    if ((dx - _currentPos.dx).abs() > 0.1 || (dy - _currentPos.dy).abs() > 0.1) {
        setState(() {
          _currentPos = Offset(dx, dy);
        });
    }
  }

  @override
  void dispose() {
    _ticker.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    // Offset slightly for pointer placement
    const double cursorSize = 40;
    
    return Positioned(
      left: _currentPos.dx,
      top: _currentPos.dy,
      child: IgnorePointer(
        child: SizedBox(
          width: cursorSize,
          height: cursorSize,
          child: CustomPaint(
            painter: _TriangleCursorPainter(),
          ),
        ),
      ),
    );
  }
}

// --- CURSOR PAINTER: Draws the sharp, modern triangle ---

class _TriangleCursorPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    const double strokeWidth = 2.0;
    final double halfWidth = size.width / 2;
    final double height = size.height;
    
    final Path path = Path();
    
    // 1. Draw the main pointing triangle (tip pointing right)
    path.moveTo(0, 0); // Top left
    path.lineTo(size.width, halfWidth); // Tip (Center right)
    path.lineTo(0, height); // Bottom left
    path.close();

    // The Inner Cutout (giving it that aperture look)
    final Path innerPath = Path();
    // Start slightly in from the tip, smaller triangle pointed the same way
    innerPath.moveTo(size.width * 0.2, height * 0.25);
    innerPath.lineTo(size.width * 0.8, halfWidth);
    innerPath.lineTo(size.width * 0.2, height * 0.75);
    innerPath.close();

    // Use Difference to create the outline look without using glow
    final Path finalPath = Path.combine(
      PathOperation.difference,
      path,
      innerPath,
    );
    
    // Paint the Outline (White/Cyan)
    final Paint paint = Paint()
      ..color = Colors.cyanAccent.withOpacity(0.9)
      ..style = PaintingStyle.stroke
      ..strokeWidth = strokeWidth
      ..strokeJoin = StrokeJoin.miter; // Ensures sharp corners

    // Draw the final path
    canvas.drawPath(finalPath, paint);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}