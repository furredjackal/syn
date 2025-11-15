import 'package:flutter/material.dart';

class TransitionOverlay extends StatefulWidget {
  final String transitionType; // 'slash', 'fade', 'glitch', 'pixelate'
  final Duration duration;
  final VoidCallback onComplete;

  const TransitionOverlay({
    Key? key,
    this.transitionType = 'fade',
    this.duration = const Duration(milliseconds: 600),
    required this.onComplete,
  }) : super(key: key);

  @override
  State<TransitionOverlay> createState() => _TransitionOverlayState();
}

class _TransitionOverlayState extends State<TransitionOverlay> with SingleTickerProviderStateMixin {
  late AnimationController _controller;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      duration: widget.duration,
      vsync: this,
    );
    _controller.forward().then((_) {
      widget.onComplete();
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
        switch (widget.transitionType) {
          case 'slash':
            return _buildSlashTransition();
          case 'glitch':
            return _buildGlitchTransition();
          case 'pixelate':
            return _buildPixelateTransition();
          case 'fade':
          default:
            return _buildFadeTransition();
        }
      },
    );
  }

  Widget _buildFadeTransition() {
    return Container(
      color: Colors.black.withOpacity(1.0 - _controller.value),
    );
  }

  Widget _buildSlashTransition() {
    return CustomPaint(
      painter: SlashPainter(_controller.value),
      size: Size.infinite,
    );
  }

  Widget _buildGlitchTransition() {
    return Stack(
      children: [
        Container(color: Colors.black),
        Transform.translate(
          offset: Offset(_controller.value > 0.5 ? 10 : -10, 0),
          child: Container(
            color: Colors.red.withOpacity((1.0 - _controller.value) * 0.3),
          ),
        ),
        Transform.translate(
          offset: Offset(_controller.value > 0.5 ? -10 : 10, 0),
          child: Container(
            color: Colors.cyan.withOpacity((1.0 - _controller.value) * 0.3),
          ),
        ),
      ],
    );
  }

  Widget _buildPixelateTransition() {
    final pixelSize = 1 + (1.0 - _controller.value) * 50;
    return Container(
      color: Colors.black,
      child: CustomPaint(
        painter: PixelatePainter(_controller.value, pixelSize),
        size: Size.infinite,
      ),
    );
  }
}

class SlashPainter extends CustomPainter {
  final double progress;

  SlashPainter(this.progress);

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = Colors.cyan
      ..strokeWidth = 8
      ..strokeCap = StrokeCap.round;

    final startX = size.width * progress - 100;
    final endX = size.width * progress + 100;
    canvas.drawLine(
      Offset(startX, -100),
      Offset(endX, size.height + 100),
      paint,
    );
  }

  @override
  bool shouldRepaint(SlashPainter oldDelegate) => oldDelegate.progress != progress;
}

class PixelatePainter extends CustomPainter {
  final double progress;
  final double pixelSize;

  PixelatePainter(this.progress, this.pixelSize);

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()..color = Colors.black.withOpacity(progress);
    for (double x = 0; x < size.width; x += pixelSize) {
      for (double y = 0; y < size.height; y += pixelSize) {
        canvas.drawRect(
          Rect.fromLTWH(x, y, pixelSize, pixelSize),
          paint,
        );
      }
    }
  }

  @override
  bool shouldRepaint(PixelatePainter oldDelegate) => oldDelegate.progress != progress || oldDelegate.pixelSize != pixelSize;
}
