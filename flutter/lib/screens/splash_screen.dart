import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';

/// Splash Screen - Hybrid Architecture Flutter Widget
///
/// Animated logo screen with:
/// - Large "S Y N" title
/// - Subtitle "Simulate Your Narrative"
/// - Gradient background with scan lines
/// - Auto-transitions after delay
class SplashScreen extends StatefulWidget {
  final VoidCallback onFinish;
  final Duration duration;

  const SplashScreen({
    super.key,
    required this.onFinish,
    this.duration = const Duration(milliseconds: 2400),
  });

  @override
  State<SplashScreen> createState() => _SplashScreenState();
}

class _SplashScreenState extends State<SplashScreen> {
  @override
  void initState() {
    super.initState();
    // Auto-transition after duration
    Future.delayed(widget.duration, () {
      if (mounted) {
        widget.onFinish();
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: const BoxDecoration(
        gradient: LinearGradient(
          colors: [
            Color(0xFF050505),
            Color(0xFF101010),
          ],
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
        ),
      ),
      child: Stack(
        children: [
          // Scan lines background effect
          _ScanLinesEffect(),

          // Centered content
          Center(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                // Main logo "S Y N"
                Text(
                  'S Y N',
                  style: TextStyle(
                    fontSize: 96,
                    fontWeight: FontWeight.w900,
                    letterSpacing: 24,
                    color: Colors.white,
                    shadows: [
                      Shadow(
                        color: Colors.cyanAccent.withValues(alpha: 0.6),
                        blurRadius: 30,
                        offset: const Offset(0, 0),
                      ),
                    ],
                  ),
                )
                    .animate()
                    .fadeIn(duration: 800.ms, curve: Curves.easeOut)
                    .scale(
                      begin: const Offset(0.8, 0.8),
                      duration: 800.ms,
                      curve: Curves.easeOutBack,
                    ),

                const SizedBox(height: 32),

                // Subtitle
                Text(
                  'Simulate Your Narrative',
                  style: TextStyle(
                    color: const Color(0xFFEEEEEE),
                    fontSize: 20,
                    letterSpacing: 4,
                    fontWeight: FontWeight.w300,
                  ),
                )
                    .animate()
                    .fadeIn(
                      delay: 400.ms,
                      duration: 600.ms,
                      curve: Curves.easeOut,
                    )
                    .slideY(
                      begin: 0.2,
                      duration: 600.ms,
                      curve: Curves.easeOut,
                    ),

                const SizedBox(height: 60),

                // Loading indicator
                SizedBox(
                  width: 200,
                  height: 3,
                  child: LinearProgressIndicator(
                    backgroundColor: Colors.white.withValues(alpha: 0.1),
                    valueColor: AlwaysStoppedAnimation(Colors.cyanAccent),
                  ),
                )
                    .animate()
                    .fadeIn(delay: 800.ms, duration: 400.ms)
                    .slideY(begin: 0.5, duration: 400.ms),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

/// Scan lines background effect widget
class _ScanLinesEffect extends StatelessWidget {
  @override
  Widget build(BuildContext context) {
    return CustomPaint(
      painter: _ScanLinesPainter(),
      child: Container(),
    );
  }
}

/// Custom painter for scan lines effect
class _ScanLinesPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = Colors.white.withValues(alpha: 0.02)
      ..strokeWidth = 1;

    // Draw horizontal scan lines
    for (int i = 0; i < 30; i++) {
      final y = i * size.height / 30;
      canvas.drawLine(
        Offset(0, y),
        Offset(size.width, y),
        paint,
      );
    }
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}
