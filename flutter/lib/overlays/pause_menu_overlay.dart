import 'package:flutter/material.dart';
import '../syn_game.dart';
import '../ui/syn_theme.dart';

class PauseMenuOverlay extends StatefulWidget {
  final SynGame game;

  const PauseMenuOverlay({
    Key? key,
    required this.game,
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
    void resumeGame() {
      widget.game.resumeEngine();
      widget.game.overlays.remove('PauseMenuOverlay');
    }

    void openSettings() {
      // TODO: swap to SettingsOverlay when implemented.
      resumeGame();
      widget.game.showSettings();
    }

    void quitToMenu() {
      widget.game.promptQuitToMenu();
    }

    return Stack(
      children: [
        FadeTransition(
          opacity: _fadeAnimation,
          child: GestureDetector(
            onTap: resumeGame,
            child: Container(color: SynColors.bgDark.withOpacity(0.8)),
          ),
        ),
        SlideTransition(
          position: _slideAnimation,
          child: Center(
            child: ClipPath(
              clipper: _PanelClipper(),
              child: Container(
                width: 460,
                padding: const EdgeInsets.all(SynLayout.paddingLarge),
                decoration: BoxDecoration(
                  color: SynColors.bgPanel,
                  border: Border.all(
                    color: SynColors.primaryCyan,
                    width: SynLayout.borderWidthHeavy,
                  ),
                ),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      'PAUSE',
                      style: SynTextStyles.h1Event,
                    ),
                    const SizedBox(height: SynLayout.paddingLarge),
                    _PauseMenuButton(label: 'RESUME', onPressed: resumeGame),
                    _PauseMenuButton(label: 'SETTINGS', onPressed: openSettings),
                    _PauseMenuButton(
                      label: 'QUIT TO MAIN MENU',
                      onPressed: quitToMenu,
                      isDestructive: true,
                    ),
                  ],
                ),
              ),
            ),
          ),
        ),
      ],
    );
  }
}

class _PanelClipper extends CustomClipper<Path> {
  @override
  Path getClip(Size size) {
    return Path()
      ..moveTo(28, 0)
      ..lineTo(size.width, 0)
      ..lineTo(size.width - 28, size.height)
      ..lineTo(0, size.height)
      ..close();
  }

  @override
  bool shouldReclip(covariant CustomClipper<Path> oldClipper) => false;
}

class _PauseMenuButton extends StatefulWidget {
  const _PauseMenuButton({
    required this.label,
    required this.onPressed,
    this.isDestructive = false,
  });

  final String label;
  final VoidCallback onPressed;
  final bool isDestructive;

  @override
  State<_PauseMenuButton> createState() => _PauseMenuButtonState();
}

class _PauseMenuButtonState extends State<_PauseMenuButton> {
  bool _hovering = false;

  @override
  Widget build(BuildContext context) {
    final borderColor = widget.isDestructive
        ? SynColors.accentRed
        : SynColors.primaryCyan;
    final bgColor = _hovering
        ? borderColor.withOpacity(0.2)
        : SynColors.bgPanel.withOpacity(0.6);

    return MouseRegion(
      onEnter: (_) => setState(() => _hovering = true),
      onExit: (_) => setState(() => _hovering = false),
      child: GestureDetector(
        onTap: widget.onPressed,
        child: ClipPath(
          clipper: _ButtonClipper(),
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 150),
            margin: const EdgeInsets.symmetric(vertical: SynLayout.paddingSmall),
            padding: const EdgeInsets.symmetric(
              vertical: SynLayout.paddingSmall,
              horizontal: SynLayout.paddingLarge,
            ),
            decoration: BoxDecoration(
              color: bgColor,
              border: Border.all(color: borderColor, width: SynLayout.borderWidthNormal),
            ),
            child: Center(
              child: Text(
                widget.label,
                style: SynTextStyles.body.copyWith(
                  letterSpacing: 2,
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

class _ButtonClipper extends CustomClipper<Path> {
  @override
  Path getClip(Size size) {
    return Path()
      ..moveTo(12, 0)
      ..lineTo(size.width, 0)
      ..lineTo(size.width - 12, size.height)
      ..lineTo(0, size.height)
      ..close();
  }

  @override
  bool shouldReclip(covariant CustomClipper<Path> oldClipper) => false;
}
