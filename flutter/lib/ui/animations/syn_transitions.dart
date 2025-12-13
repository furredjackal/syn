import 'package:flutter/material.dart';
import '../theme/syn_theme.dart';
import '../helpers/animated_builder.dart';

/// Staggered entrance animation wrapper for list items.
///
/// Animates children in sequence with Persona-style slam effect.
class SynStaggeredEntrance extends StatefulWidget {
  final int index;
  final int totalItems;
  final Widget child;
  final Duration staggerDelay;
  final Duration animationDuration;
  final Offset slideFrom;
  final bool enabled;

  const SynStaggeredEntrance({
    super.key,
    required this.index,
    this.totalItems = 10,
    required this.child,
    this.staggerDelay = const Duration(milliseconds: 50),
    this.animationDuration = const Duration(milliseconds: 400),
    this.slideFrom = const Offset(-0.3, 0),
    this.enabled = true,
  });

  @override
  State<SynStaggeredEntrance> createState() => _SynStaggeredEntranceState();
}

class _SynStaggeredEntranceState extends State<SynStaggeredEntrance>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<double> _opacity;
  late Animation<Offset> _slide;
  late Animation<double> _scale;

  @override
  void initState() {
    super.initState();

    _controller = AnimationController(
      duration: widget.animationDuration,
      vsync: this,
    );

    _opacity = Tween<double>(begin: 0, end: 1).animate(
      CurvedAnimation(
        parent: _controller,
        curve: const Interval(0, 0.6, curve: Curves.easeOut),
      ),
    );

    _slide = Tween<Offset>(
      begin: widget.slideFrom,
      end: Offset.zero,
    ).animate(
      CurvedAnimation(
        parent: _controller,
        curve: SynTheme.snapIn,
      ),
    );

    _scale = TweenSequence<double>([
      TweenSequenceItem(tween: Tween(begin: 0.8, end: 1.05), weight: 70),
      TweenSequenceItem(tween: Tween(begin: 1.05, end: 1.0), weight: 30),
    ]).animate(
      CurvedAnimation(
        parent: _controller,
        curve: Curves.easeOut,
      ),
    );

    if (widget.enabled) {
      final delay = widget.staggerDelay * widget.index;
      Future.delayed(delay, () {
        if (mounted) _controller.forward();
      });
    } else {
      _controller.value = 1.0;
    }
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
      builder: (context, _) => Opacity(
        opacity: _opacity.value,
        child: SlideTransition(
          position: _slide,
          child: ScaleTransition(
            scale: _scale,
            alignment: Alignment.centerLeft,
            child: widget.child,
          ),
        ),
      ),
    );
  }
}

/// Hero-style page transition with dramatic entrance.
///
/// Use for panels that slam in from the side with impact.
class SynPanelEntrance extends StatefulWidget {
  final Widget child;
  final Duration duration;
  final SlideDirection direction;
  final bool enabled;
  final VoidCallback? onComplete;

  const SynPanelEntrance({
    super.key,
    required this.child,
    this.duration = const Duration(milliseconds: 500),
    this.direction = SlideDirection.left,
    this.enabled = true,
    this.onComplete,
  });

  @override
  State<SynPanelEntrance> createState() => _SynPanelEntranceState();
}

enum SlideDirection { left, right, top, bottom }

class _SynPanelEntranceState extends State<SynPanelEntrance>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<Offset> _slideAnimation;
  late Animation<double> _opacityAnimation;
  late Animation<double> _scaleAnimation;
  late Animation<double> _skewAnimation;

  @override
  void initState() {
    super.initState();

    _controller = AnimationController(
      duration: widget.duration,
      vsync: this,
    );

    final beginOffset = _getBeginOffset(widget.direction);

    _slideAnimation = Tween<Offset>(
      begin: beginOffset,
      end: Offset.zero,
    ).animate(CurvedAnimation(
      parent: _controller,
      curve: SynTheme.snapIn,
    ));

    _opacityAnimation = Tween<double>(begin: 0, end: 1).animate(
      CurvedAnimation(
        parent: _controller,
        curve: const Interval(0, 0.5, curve: Curves.easeOut),
      ),
    );

    _scaleAnimation = TweenSequence<double>([
      TweenSequenceItem(tween: Tween(begin: 0.9, end: 1.02), weight: 80),
      TweenSequenceItem(tween: Tween(begin: 1.02, end: 1.0), weight: 20),
    ]).animate(CurvedAnimation(
      parent: _controller,
      curve: Curves.easeOut,
    ));

    // Dramatic skew that settles
    _skewAnimation = TweenSequence<double>([
      TweenSequenceItem(tween: Tween(begin: -0.15, end: 0.03), weight: 70),
      TweenSequenceItem(tween: Tween(begin: 0.03, end: 0.0), weight: 30),
    ]).animate(CurvedAnimation(
      parent: _controller,
      curve: Curves.easeOut,
    ));

    if (widget.enabled) {
      _controller.forward().then((_) {
        widget.onComplete?.call();
      });
    } else {
      _controller.value = 1.0;
    }
  }

  Offset _getBeginOffset(SlideDirection direction) {
    switch (direction) {
      case SlideDirection.left:
        return const Offset(-1.0, 0);
      case SlideDirection.right:
        return const Offset(1.0, 0);
      case SlideDirection.top:
        return const Offset(0, -1.0);
      case SlideDirection.bottom:
        return const Offset(0, 1.0);
    }
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
      builder: (context, _) => Opacity(
        opacity: _opacityAnimation.value,
        child: SlideTransition(
          position: _slideAnimation,
          child: Transform(
            alignment: Alignment.center,
            transform: Matrix4.identity()
              ..scale(_scaleAnimation.value)
              ..rotateZ(_skewAnimation.value * 0.1),
            child: widget.child,
          ),
        ),
      ),
    );
  }
}

/// Scanline reveal effect - text/content revealed line by line.
class SynScanlineReveal extends StatefulWidget {
  final Widget child;
  final Duration duration;
  final bool enabled;
  final Color scanlineColor;

  const SynScanlineReveal({
    super.key,
    required this.child,
    this.duration = const Duration(milliseconds: 800),
    this.enabled = true,
    this.scanlineColor = SynTheme.accent,
  });

  @override
  State<SynScanlineReveal> createState() => _SynScanlineRevealState();
}

class _SynScanlineRevealState extends State<SynScanlineReveal>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      duration: widget.duration,
      vsync: this,
    );

    if (widget.enabled) {
      _controller.forward();
    } else {
      _controller.value = 1.0;
    }
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
      builder: (context, _) {
        return ClipRect(
          child: ShaderMask(
            shaderCallback: (bounds) {
              final revealPoint = _controller.value * bounds.height;
              return LinearGradient(
                begin: Alignment.topCenter,
                end: Alignment.bottomCenter,
                colors: [
                  Colors.white,
                  Colors.white,
                  Colors.transparent,
                ],
                stops: [
                  0,
                  (revealPoint / bounds.height).clamp(0, 1),
                  ((revealPoint + 20) / bounds.height).clamp(0, 1),
                ],
              ).createShader(bounds);
            },
            blendMode: BlendMode.dstIn,
            child: Stack(
              children: [
                widget.child,
                // Scanline
                if (_controller.value < 1.0)
                  Positioned(
                    left: 0,
                    right: 0,
                    top: _controller.value *
                        MediaQuery.of(context).size.height *
                        0.3,
                    child: Container(
                      height: 2,
                      decoration: BoxDecoration(
                        gradient: LinearGradient(
                          colors: [
                            widget.scanlineColor.withOpacity(0),
                            widget.scanlineColor,
                            widget.scanlineColor.withOpacity(0),
                          ],
                        ),
                        boxShadow: [
                          BoxShadow(
                            color: widget.scanlineColor.withOpacity(0.5),
                            blurRadius: 10,
                          ),
                        ],
                      ),
                    ),
                  ),
              ],
            ),
          ),
        );
      },
    );
  }
}

/// Flip card reveal with 3D rotation.
class SynFlipReveal extends StatefulWidget {
  final Widget child;
  final Duration duration;
  final bool enabled;
  final Axis axis;

  const SynFlipReveal({
    super.key,
    required this.child,
    this.duration = const Duration(milliseconds: 600),
    this.enabled = true,
    this.axis = Axis.vertical,
  });

  @override
  State<SynFlipReveal> createState() => _SynFlipRevealState();
}

class _SynFlipRevealState extends State<SynFlipReveal>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<double> _flipAnimation;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      duration: widget.duration,
      vsync: this,
    );

    _flipAnimation = Tween<double>(
      begin: 1.57, // 90 degrees in radians
      end: 0,
    ).animate(CurvedAnimation(
      parent: _controller,
      curve: SynTheme.snapIn,
    ));

    if (widget.enabled) {
      _controller.forward();
    } else {
      _controller.value = 1.0;
    }
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
      builder: (context, _) {
        final transform = Matrix4.identity()
          ..setEntry(3, 2, 0.001); // Perspective

        if (widget.axis == Axis.vertical) {
          transform.rotateY(_flipAnimation.value);
        } else {
          transform.rotateX(_flipAnimation.value);
        }

        return Opacity(
          opacity: _controller.value,
          child: Transform(
            alignment: Alignment.center,
            transform: transform,
            child: widget.child,
          ),
        );
      },
    );
  }
}
