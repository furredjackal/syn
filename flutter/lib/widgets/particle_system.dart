import 'package:flutter/material.dart';
import 'dart:math' as math;

class Particle {
  Offset position;
  Offset velocity;
  double lifetime;
  double maxLifetime;
  Color color;
  double size;

  Particle({
    required this.position,
    required this.velocity,
    required this.lifetime,
    required this.maxLifetime,
    required this.color,
    required this.size,
  });

  void update(double dt) {
    position += velocity * dt;
    lifetime -= dt;
    // Dampen velocity
    velocity *= 0.98;
  }

  double get opacity => math.max(0, lifetime / maxLifetime);
}

class ParticleSystemWidget extends StatefulWidget {
  final int particleCount;
  final Color particleColor;
  final double particleSize;
  final Offset emitterPosition;
  final double emissionRate;
  final double lifetime;
  final Offset? velocityRange;

  const ParticleSystemWidget({
    Key? key,
    this.particleCount = 50,
    this.particleColor = Colors.cyan,
    this.particleSize = 4,
    this.emitterPosition = const Offset(0.5, 0.5),
    this.emissionRate = 20,
    this.lifetime = 2.0,
    this.velocityRange,
  }) : super(key: key);

  @override
  State<ParticleSystemWidget> createState() => _ParticleSystemWidgetState();
}

class _ParticleSystemWidgetState extends State<ParticleSystemWidget>
    with TickerProviderStateMixin {
  late AnimationController _controller;
  late List<Particle> particles;
  final Random = math.Random();

  @override
  void initState() {
    super.initState();
    particles = [];
    _controller = AnimationController(
      vsync: this,
      duration: Duration(
        milliseconds: (widget.lifetime * 1000).toInt(),
      ),
    )..repeat();
    _controller.addListener(_updateParticles);
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  void _updateParticles() {
    setState(() {
      // Remove dead particles
      particles.removeWhere((p) => p.lifetime <= 0);

      // Update remaining particles
      for (var particle in particles) {
        particle.update(0.016); // ~60fps
      }
    });
  }

  Particle _createParticle(Size size) {
    final angle = math.Random().nextDouble() * 2 * math.pi;
    final speed = 50 + math.Random().nextDouble() * 100;
    final velocity = Offset(
      math.cos(angle) * speed,
      math.sin(angle) * speed,
    );

    return Particle(
      position: Offset(
        widget.emitterPosition.dx * size.width,
        widget.emitterPosition.dy * size.height,
      ),
      velocity: velocity,
      lifetime: widget.lifetime,
      maxLifetime: widget.lifetime,
      color: widget.particleColor,
      size: widget.particleSize,
    );
  }

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final size = Size(constraints.maxWidth, constraints.maxHeight);

        // Emit particles
        if (particles.length < widget.particleCount) {
          final particlesToAdd = (widget.emissionRate * 0.016).ceil();
          for (int i = 0; i < particlesToAdd; i++) {
            particles.add(_createParticle(size));
          }
        }

        return CustomPaint(
          painter: ParticlePainter(particles),
          size: Size.infinite,
        );
      },
    );
  }
}

class ParticlePainter extends CustomPainter {
  final List<Particle> particles;

  ParticlePainter(this.particles);

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()..style = PaintingStyle.fill;

    for (var particle in particles) {
      paint
        ..color = particle.color.withOpacity(
          particle.opacity * 0.8,
        );

      canvas.drawCircle(
        particle.position,
        particle.size,
        paint,
      );
    }
  }

  @override
  bool shouldRepaint(ParticlePainter oldDelegate) => true;
}

class FloatingTextWidget extends StatefulWidget {
  final String text;
  final Offset startPosition;
  final Duration duration;
  final Color color;
  final TextStyle? textStyle;
  final double floatDistance;

  const FloatingTextWidget({
    Key? key,
    required this.text,
    required this.startPosition,
    this.duration = const Duration(milliseconds: 800),
    this.color = Colors.cyan,
    this.textStyle,
    this.floatDistance = 50,
  }) : super(key: key);

  @override
  State<FloatingTextWidget> createState() => _FloatingTextWidgetState();
}

class _FloatingTextWidgetState extends State<FloatingTextWidget>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<double> _fadeAnimation;
  late Animation<Offset> _positionAnimation;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      duration: widget.duration,
      vsync: this,
    );

    _fadeAnimation = Tween<double>(begin: 1.0, end: 0.0).animate(
      CurvedAnimation(parent: _controller, curve: Curves.easeOut),
    );

    _positionAnimation = Tween<Offset>(
      begin: Offset.zero,
      end: Offset(0, -widget.floatDistance),
    ).animate(
      CurvedAnimation(parent: _controller, curve: Curves.easeOut),
    );

    _controller.forward().then((_) {
      // Auto-remove after animation completes
    });
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return AnimatedBuilder(
      animation: _controller,
      builder: (context, child) {
        return Positioned(
          left: widget.startPosition.dx + _positionAnimation.value.dx,
          top: widget.startPosition.dy + _positionAnimation.value.dy,
          child: Opacity(
            opacity: _fadeAnimation.value,
            child: Text(
              widget.text,
              style: (widget.textStyle ??
                      Theme.of(context).textTheme.bodyMedium)
                  ?.copyWith(
                    color: widget.color,
                    fontWeight: FontWeight.bold,
                    fontSize: 16,
                  ),
            ),
          ),
        );
      },
    );
  }
}

class StatChangeNotificationWidget extends StatefulWidget {
  final String statName;
  final double statValue;
  final Offset position;
  final Color? color;

  const StatChangeNotificationWidget({
    Key? key,
    required this.statName,
    required this.statValue,
    required this.position,
    this.color,
  }) : super(key: key);

  @override
  State<StatChangeNotificationWidget> createState() =>
      _StatChangeNotificationWidgetState();
}

class _StatChangeNotificationWidgetState
    extends State<StatChangeNotificationWidget>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<double> _scaleAnimation;
  late Animation<double> _opacityAnimation;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      duration: const Duration(milliseconds: 600),
      vsync: this,
    );

    _scaleAnimation = Tween<double>(begin: 0.5, end: 1.2).animate(
      CurvedAnimation(parent: _controller, curve: Curves.elasticOut),
    );

    _opacityAnimation = Tween<double>(begin: 1.0, end: 0.0).animate(
      CurvedAnimation(parent: _controller, curve: Curves.easeOut),
    );

    _controller.forward();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final isPositive = widget.statValue > 0;
    final color = widget.color ??
        (isPositive
            ? Colors.green
            : widget.statValue < 0
                ? Colors.red
                : Colors.amber);

    return AnimatedBuilder(
      animation: _controller,
      builder: (context, child) {
        return Positioned(
          left: widget.position.dx - 30,
          top: widget.position.dy,
          child: Opacity(
            opacity: _opacityAnimation.value,
            child: Transform.scale(
              scale: _scaleAnimation.value,
              child: Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
                decoration: BoxDecoration(
                  color: color.withOpacity(0.2),
                  border: Border.all(color: color, width: 1),
                  borderRadius: BorderRadius.circular(4),
                ),
                child: Text(
                  '${isPositive ? '+' : ''}${widget.statValue.toStringAsFixed(1)}',
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                        color: color,
                        fontWeight: FontWeight.bold,
                      ),
                ),
              ),
            ),
          ),
        );
      },
    );
  }
}
