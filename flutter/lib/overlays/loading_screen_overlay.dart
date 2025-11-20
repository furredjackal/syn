import 'package:flutter/material.dart';

Widget buildLoadingOverlay(BuildContext context) {
  return const LoadingScreenOverlay();
}

class LoadingScreenOverlay extends StatefulWidget {
  final double progress;
  final String message;
  final bool isFullScreen;

  const LoadingScreenOverlay({
    Key? key,
    this.progress = 0.5,
    this.message = 'Loading...',
    this.isFullScreen = true,
  }) : super(key: key);

  @override
  State<LoadingScreenOverlay> createState() => _LoadingScreenOverlayState();
}

class _LoadingScreenOverlayState extends State<LoadingScreenOverlay> with SingleTickerProviderStateMixin {
  late AnimationController _animationController;

  @override
  void initState() {
    super.initState();
    _animationController = AnimationController(
      duration: const Duration(seconds: 2),
      vsync: this,
    )..repeat();
  }

  @override
  void dispose() {
    _animationController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      color: Colors.black.withOpacity(0.85),
      child: Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            RotationTransition(
              turns: _animationController,
              child: Container(
                width: 80,
                height: 80,
                decoration: BoxDecoration(
                  border: Border.all(color: Colors.cyan, width: 3),
                  borderRadius: BorderRadius.circular(40),
                ),
                child: const SizedBox(),
              ),
            ),
            const SizedBox(height: 32),
            Text(widget.message, style: Theme.of(context).textTheme.titleMedium?.copyWith(color: Colors.cyan)),
            const SizedBox(height: 16),
            SizedBox(
              width: 200,
              child: LinearProgressIndicator(
                value: widget.progress,
                backgroundColor: Colors.grey.shade800,
                valueColor: const AlwaysStoppedAnimation(Colors.cyan),
                minHeight: 4,
              ),
            ),
            const SizedBox(height: 8),
            Text(
              '${(widget.progress * 100).toStringAsFixed(0)}%',
              style: Theme.of(context).textTheme.bodySmall?.copyWith(color: Colors.grey),
            ),
          ],
        ),
      ),
    );
  }
}
