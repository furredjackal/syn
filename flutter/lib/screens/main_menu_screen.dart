import 'package:flutter/material.dart';
import '../widgets/particle_system.dart';

class _MenuOption {
  final String label;
  final VoidCallback onPressed;
  final bool enabled;

  _MenuOption({
    required this.label,
    required this.onPressed,
    this.enabled = true,
  });
}

class MainMenuScreen extends StatefulWidget {
  const MainMenuScreen({Key? key}) : super(key: key);

  @override
  State<MainMenuScreen> createState() => _MainMenuScreenState();
}

class _MainMenuScreenState extends State<MainMenuScreen> {
  late List<_MenuOption> menuOptions;

  @override
  void initState() {
    super.initState();
    menuOptions = [
      _MenuOption(
        label: 'NEW LIFE',
        onPressed: () => Navigator.pushNamed(context, '/character_creation'),
      ),
      _MenuOption(
        label: 'CONTINUE',
        onPressed: () => _handleContinue(),
        enabled: false, // Disabled if no save exists
      ),
      _MenuOption(
        label: 'LOAD GAME',
        onPressed: () => _handleLoadGame(),
      ),
      _MenuOption(
        label: 'SETTINGS',
        onPressed: () => Navigator.pushNamed(context, '/settings'),
      ),
      _MenuOption(
        label: 'CREDITS',
        onPressed: () => _handleCredits(),
      ),
      _MenuOption(
        label: 'EXIT',
        onPressed: () => Navigator.pop(context),
      ),
    ];
  }

  void _handleContinue() {
    Navigator.pushNamed(context, '/game');
  }

  void _handleLoadGame() {
    // TODO: Implement load game dialog
    ScaffoldMessenger.of(context).showSnackBar(
      const SnackBar(content: Text('Load game functionality coming soon')),
    );
  }

  void _handleCredits() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        backgroundColor: const Color(0xFF15192E),
        title: const Text('CREDITS'),
        content: SingleChildScrollView(
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            mainAxisSize: MainAxisSize.min,
            children: [
              _creditSection('DESIGN', ['SYN Design Team']),
              const SizedBox(height: 16),
              _creditSection('DEVELOPMENT', ['Flutter / Rust Team']),
              const SizedBox(height: 16),
              _creditSection('NARRATIVE', ['Content Writers']),
            ],
          ),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context),
            child: const Text('CLOSE'),
          ),
        ],
      ),
    );
  }

  Widget _creditSection(String title, List<String> credits) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          title,
          style: Theme.of(context).textTheme.titleSmall?.copyWith(
                color: const Color(0xFF00D9FF),
              ),
        ),
        const SizedBox(height: 8),
        ...credits.map((credit) => Text(
              credit,
              style: Theme.of(context).textTheme.bodySmall,
            )),
      ],
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF0A0E27),
      body: Stack(
        children: [
          // Background with gradient
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
              particleCount: 50,
              particleColor: Color(0xFF9D4EDD),
              particleSize: 2.5,
              emitterPosition: Offset(0.5, 0.4),
              emissionRate: 12,
              lifetime: 4.0,
            ),
          ),

          // Grid background effect
          Opacity(
            opacity: 0.05,
            child: CustomPaint(
              painter: GridPainter(),
              size: Size.infinite,
            ),
          ),

          // Main menu content
          Center(
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                // Title
                Text(
                  'SYN',
                  style: Theme.of(context).textTheme.displayLarge?.copyWith(
                    fontSize: 72,
                    color: const Color(0xFF00D9FF),
                    shadows: [
                      Shadow(
                        blurRadius: 30,
                        color: const Color(0xFF00D9FF).withOpacity(0.5),
                        offset: const Offset(0, 0),
                      ),
                    ],
                  ),
                ),
                const SizedBox(height: 8),
                Text(
                  'Simulate Your Narrative',
                  style: Theme.of(context).textTheme.bodyLarge?.copyWith(
                        letterSpacing: 3,
                        color: Colors.white.withOpacity(0.7),
                      ),
                ),
                const SizedBox(height: 60),

                // Menu buttons
                ...menuOptions.map((option) => Padding(
                      padding: const EdgeInsets.symmetric(vertical: 12),
                      child: _MenuButton(option: option),
                    )),
              ],
            ),
          ),

          // Version info
          Positioned(
            bottom: 20,
            left: 20,
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

class _MenuButton extends StatefulWidget {
  final _MenuOption option;

  const _MenuButton({required this.option, Key? key}) : super(key: key);

  @override
  State<_MenuButton> createState() => _MenuButtonState();
}

class _MenuButtonState extends State<_MenuButton> {
  bool _isHovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) {
        if (widget.option.enabled) {
          setState(() => _isHovered = true);
        }
      },
      onExit: (_) {
        setState(() => _isHovered = false);
      },
      child: GestureDetector(
        onTap: widget.option.enabled ? widget.option.onPressed : null,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 200),
          padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 12),
          decoration: BoxDecoration(
            border: Border.all(
              color: _isHovered
                  ? const Color(0xFF00D9FF)
                  : const Color(0xFF00D9FF).withOpacity(0.3),
              width: 2,
            ),
            color: _isHovered
                ? const Color(0xFF00D9FF).withOpacity(0.1)
                : Colors.transparent,
          ),
          child: Text(
            widget.option.label,
            style: Theme.of(context).textTheme.titleSmall?.copyWith(
                  color: widget.option.enabled
                      ? const Color(0xFF00D9FF)
                      : Colors.grey,
                  fontSize: 20,
                ),
          ),
        ),
      ),
    );
  }
}

class GridPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = Colors.white
      ..strokeWidth = 1;

    const spacing = 50.0;

    // Vertical lines
    for (double x = 0; x < size.width; x += spacing) {
      canvas.drawLine(Offset(x, 0), Offset(x, size.height), paint);
    }

    // Horizontal lines
    for (double y = 0; y < size.height; y += spacing) {
      canvas.drawLine(Offset(0, y), Offset(size.width, y), paint);
    }
  }

  @override
  bool shouldRepaint(GridPainter oldDelegate) => false;
}
