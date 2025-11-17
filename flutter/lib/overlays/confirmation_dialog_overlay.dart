import 'package:flutter/material.dart';
import '../syn_game.dart';

class ConfirmationDialogOverlay extends StatefulWidget {
  final SynGame game;

  const ConfirmationDialogOverlay({
    Key? key,
    required this.game,
  }) : super(key: key);

  @override
  State<ConfirmationDialogOverlay> createState() => _ConfirmationDialogOverlayState();
}

class _ConfirmationDialogOverlayState extends State<ConfirmationDialogOverlay> with SingleTickerProviderStateMixin {
  late AnimationController _animationController;
  late Animation<double> _fadeAnimation;
  late Animation<Offset> _slideAnimation;

  @override
  void initState() {
    super.initState();
    _animationController = AnimationController(
      duration: const Duration(milliseconds: 300),
      vsync: this,
    );
    _fadeAnimation = Tween<double>(begin: 0.0, end: 1.0).animate(_animationController);
    _slideAnimation = Tween<Offset>(begin: const Offset(0, 0.3), end: Offset.zero).animate(
      CurvedAnimation(parent: _animationController, curve: Curves.easeOutCubic),
    );
    _animationController.forward();
  }

  @override
  void dispose() {
    _animationController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final request = widget.game.pendingConfirmation;
    if (request == null) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        widget.game.cancelCurrentDialog();
      });
      return const SizedBox.shrink();
    }

    return FadeTransition(
      opacity: _fadeAnimation,
      child: Container(
        color: Colors.black.withOpacity(0.5),
        child: Center(
          child: SlideTransition(
            position: _slideAnimation,
            child: Container(
              width: 400,
              padding: const EdgeInsets.all(24),
              decoration: BoxDecoration(
                color: const Color(0xFF1a1f3a),
                border: Border.all(color: Colors.cyan, width: 2),
                borderRadius: BorderRadius.circular(8),
              ),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text(
                    request.title,
                    style: Theme.of(context).textTheme.titleLarge?.copyWith(
                          color: Colors.cyan,
                        ),
                  ),
                  const SizedBox(height: 16),
                  Text(
                    request.message,
                    style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                          color: Colors.grey[300],
                        ),
                    textAlign: TextAlign.center,
                  ),
                  const SizedBox(height: 32),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      ElevatedButton(
                        onPressed: widget.game.cancelCurrentDialog,
                        style: ElevatedButton.styleFrom(
                          backgroundColor: Colors.grey.shade800,
                          padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 12),
                        ),
                        child: Text(request.cancelLabel),
                      ),
                      const SizedBox(width: 16),
                      ElevatedButton(
                        onPressed: widget.game.confirmCurrentDialog,
                        style: ElevatedButton.styleFrom(
                          backgroundColor: Colors.cyan,
                          foregroundColor: Colors.black,
                          padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 12),
                        ),
                        child: Text(request.confirmLabel),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }
}
