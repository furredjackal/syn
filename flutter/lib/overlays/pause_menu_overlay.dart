import 'package:flutter/material.dart';
import '../syn_game.dart';
import '../components/ui/syn_theme.dart';
import '../components/ui/buttons/menu_button.dart';

Widget buildPauseMenuOverlay(BuildContext context, SynGame game) {
  return PauseMenuOverlay(game: game);
}

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
      widget.game.overlays.remove('pause_menu');
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
            child: Container(color: SynColors.bgDark.withValues(alpha: 0.8)),
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
                    MenuButton(label: 'RESUME', onPressed: resumeGame),
                    MenuButton(label: 'SETTINGS', onPressed: openSettings),
                    MenuButton(
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
