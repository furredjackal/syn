import 'package:flutter/material.dart';
import '../widgets/particle_system.dart';

class SplashScreen extends StatefulWidget {
  const SplashScreen({Key? key}) : super(key: key);

  @override
  State<SplashScreen> createState() => _SplashScreenState();
}

class _SplashScreenState extends State<SplashScreen>
    with SingleTickerProviderStateMixin {
  late AnimationController _animationController;
  late Animation<double> _scaleAnimation;
  late Animation<double> _fadeAnimation;

  @override
  void initState() {
    super.initState();

    _animationController = AnimationController(
      duration: const Duration(milliseconds: 2000),
      vsync: this,
    );

    _scaleAnimation = Tween<double>(begin: 0.5, end: 1.2).animate(
      CurvedAnimation(parent: _animationController, curve: Curves.easeInOut),
    );

    _fadeAnimation = Tween<double>(begin: 0.0, end: 1.0).animate(
      CurvedAnimation(parent: _animationController, curve: Curves.easeInOut),
    );

    _animationController.forward().then((_) {
      Future.delayed(const Duration(milliseconds: 1000), () {
        if (mounted) {
          Navigator.of(context).pushReplacementNamed('/menu');
        }
      });
    });
  }

  @override
  void dispose() {
    _animationController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF0A0E27),
      body: Stack(
        children: [
          // Background gradient
          Container(
            decoration: const BoxDecoration(
              gradient: LinearGradient(
                begin: Alignment.topLeft,
                end: Alignment.bottomRight,
                colors: [
                  Color(0xFF0A0E27),
                  Color(0xFF1A1F3A),
                  Color(0xFF2D1B4E),
                ],
              ),
            ),
          ),

          // Particle effects
          const Positioned.fill(
            child: ParticleSystemWidget(
              particleCount: 40,
              particleColor: Color(0xFF00D9FF),
              particleSize: 2,
              emitterPosition: Offset(0.5, 0.3),
              emissionRate: 15,
              lifetime: 3.0,
            ),
          ),

          // Center content
          Center(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                // Logo with animation
                ScaleTransition(
                  scale: _scaleAnimation,
                  child: FadeTransition(
                    opacity: _fadeAnimation,
                    child: Column(
                      children: [
                        // SYN Logo (text-based for now)
                        Text(
                          'SYN',
                          style: Theme.of(context)
                              .textTheme
                              .displayLarge
                              ?.copyWith(
                            fontSize: 80,
                            color: const Color(0xFF00D9FF),
                            shadows: [
                              Shadow(
                                blurRadius: 20,
                                color: const Color(0xFF00D9FF).withOpacity(0.6),
                                offset: const Offset(0, 0),
                              ),
                            ],
                          ),
                        ),
                        const SizedBox(height: 16),
                        Text(
                          'Simulate Your Narrative',
                          style:
                              Theme.of(context).textTheme.bodyLarge?.copyWith(
                                    letterSpacing: 2,
                                    fontSize: 18,
                                  ),
                        ),
                      ],
                    ),
                  ),
                ),
                const SizedBox(height: 60),

                // Loading indicator
                FadeTransition(
                  opacity: _fadeAnimation,
                  child: Column(
                    children: [
                      const CircularProgressIndicator(
                        valueColor:
                            AlwaysStoppedAnimation<Color>(Color(0xFF00D9FF)),
                        strokeWidth: 2,
                      ),
                      const SizedBox(height: 20),
                      Text(
                        'Initializing Narrative Engine...',
                        style: Theme.of(context).textTheme.bodySmall?.copyWith(
                              color: Colors.white.withOpacity(0.6),
                            ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),

          // Version number
          Positioned(
            bottom: 20,
            right: 20,
            child: Text(
              'v0.1.0',
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: Colors.white.withOpacity(0.4),
                  ),
            ),
          ),
        ],
      ),
    );
  }
}
