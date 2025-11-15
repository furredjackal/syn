import 'package:flutter/material.dart';

class PauseMenuOverlay extends StatefulWidget {
  final VoidCallback onResume;
  final VoidCallback onSave;
  final VoidCallback onLoad;
  final VoidCallback onSettings;
  final VoidCallback onQuit;

  const PauseMenuOverlay({
    Key? key,
    required this.onResume,
    required this.onSave,
    required this.onLoad,
    required this.onSettings,
    required this.onQuit,
  }) : super(key: key);

  @override
  State<PauseMenuOverlay> createState() => _PauseMenuOverlayState();
}

class _PauseMenuOverlayState extends State<PauseMenuOverlay> with SingleTickerProviderStateMixin {
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
    _fadeAnimation = Tween<double>(begin: 0, end: 1).animate(_animationController);
    _slideAnimation = Tween<Offset>(begin: const Offset(0, -0.1), end: Offset.zero).animate(_animationController);
    _animationController.forward();
  }

  @override
  void dispose() {
    _animationController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: [
        FadeTransition(
          opacity: _fadeAnimation,
          child: GestureDetector(
            onTap: widget.onResume,
            child: Container(color: Colors.black.withOpacity(0.7)),
          ),
        ),
        SlideTransition(
          position: _slideAnimation,
          child: Center(
            child: Container(
              decoration: BoxDecoration(
                border: Border.all(color: Colors.cyan.withOpacity(0.5), width: 1),
                borderRadius: BorderRadius.circular(4),
                color: const Color(0xFF0A0E27),
              ),
              padding: const EdgeInsets.all(32),
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Text('GAME PAUSED', style: Theme.of(context).textTheme.titleLarge?.copyWith(color: Colors.cyan)),
                  const SizedBox(height: 32),
                  _buildMenuButton(context, 'RESUME', Icons.play_arrow, widget.onResume),
                  _buildMenuButton(context, 'SAVE GAME', Icons.save, widget.onSave),
                  _buildMenuButton(context, 'LOAD GAME', Icons.upload, widget.onLoad),
                  _buildMenuButton(context, 'SETTINGS', Icons.settings, widget.onSettings),
                  const SizedBox(height: 16),
                  _buildMenuButton(context, 'QUIT TO MENU', Icons.exit_to_app, widget.onQuit, isDestructive: true),
                ],
              ),
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildMenuButton(
    BuildContext context,
    String label,
    IconData icon,
    VoidCallback onPressed, {
    bool isDestructive = false,
  }) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 8),
      child: SizedBox(
        width: 200,
        child: ElevatedButton.icon(
          onPressed: onPressed,
          icon: Icon(icon),
          label: Text(label),
          style: ElevatedButton.styleFrom(
            backgroundColor: isDestructive ? Colors.red.shade700 : Colors.cyan,
            padding: const EdgeInsets.symmetric(vertical: 12),
          ),
        ),
      ),
    );
  }
}
