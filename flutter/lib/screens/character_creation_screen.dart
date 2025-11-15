import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../models/game_state.dart';

class CharacterCreationScreen extends StatefulWidget {
  const CharacterCreationScreen({Key? key}) : super(key: key);

  @override
  State<CharacterCreationScreen> createState() =>
      _CharacterCreationScreenState();
}

class _CharacterCreationScreenState extends State<CharacterCreationScreen> {
  final _nameController = TextEditingController();
  bool _sfwMode = true;
  String _difficulty = 'Balanced';

  @override
  void dispose() {
    _nameController.dispose();
    super.dispose();
  }

  void _handleBeginLife() {
    if (_nameController.text.isEmpty) {
      ScaffoldMessenger.of(context).showSnackBar(
        const SnackBar(content: Text('Please enter a name')),
      );
      return;
    }

    final gameState = context.read<GameState>();
    gameState.setPlayerName(_nameController.text);
    gameState.sfwMode = _sfwMode;

    Navigator.pushNamed(context, '/game');
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF0A0E27),
      body: Stack(
        children: [
          Container(
            decoration: BoxDecoration(
              gradient: LinearGradient(
                begin: Alignment.topLeft,
                end: Alignment.bottomRight,
                colors: [
                  const Color(0xFF0A0E27),
                  const Color(0xFF1A1F3A),
                  const Color(0xFF2D1B4E),
                ],
              ),
            ),
          ),
          SafeArea(
            child: Center(
              child: SingleChildScrollView(
                padding: const EdgeInsets.all(24),
                child: Column(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    // Title
                    Text(
                      'CHARACTER CREATION',
                      style: Theme.of(context).textTheme.displaySmall?.copyWith(
                            color: const Color(0xFF00D9FF),
                          ),
                    ),
                    const SizedBox(height: 48),

                    // Name input
                    Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          'NAME',
                          style: Theme.of(context).textTheme.labelMedium,
                        ),
                        const SizedBox(height: 8),
                        TextFormField(
                          controller: _nameController,
                          style: Theme.of(context).textTheme.bodyMedium,
                          decoration: InputDecoration(
                            hintText: 'Enter your character name',
                            hintStyle: Theme.of(context)
                                .textTheme
                                .bodyMedium
                                ?.copyWith(
                                  color: Colors.white.withOpacity(0.3),
                                ),
                            border: OutlineInputBorder(
                              borderSide: const BorderSide(
                                color: Color(0xFF00D9FF),
                              ),
                            ),
                            enabledBorder: OutlineInputBorder(
                              borderSide: BorderSide(
                                color: const Color(0xFF00D9FF).withOpacity(0.5),
                              ),
                            ),
                            focusedBorder: const OutlineInputBorder(
                              borderSide: BorderSide(
                                color: Color(0xFF00D9FF),
                              ),
                            ),
                          ),
                        ),
                      ],
                    ),
                    const SizedBox(height: 32),

                    // Content mode
                    Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          'CONTENT MODE',
                          style: Theme.of(context).textTheme.labelMedium,
                        ),
                        const SizedBox(height: 12),
                        Row(
                          children: [
                            Expanded(
                              child: _ToggleButton(
                                label: 'SFW',
                                isSelected: _sfwMode,
                                onPressed: () {
                                  setState(() => _sfwMode = true);
                                },
                              ),
                            ),
                            const SizedBox(width: 16),
                            Expanded(
                              child: _ToggleButton(
                                label: 'NSFW',
                                isSelected: !_sfwMode,
                                onPressed: () {
                                  setState(() => _sfwMode = false);
                                },
                              ),
                            ),
                          ],
                        ),
                      ],
                    ),
                    const SizedBox(height: 32),

                    // Difficulty
                    Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(
                          'DIFFICULTY',
                          style: Theme.of(context).textTheme.labelMedium,
                        ),
                        const SizedBox(height: 12),
                        Row(
                          children: [
                            Expanded(
                              child: _ToggleButton(
                                label: 'FORGIVING',
                                isSelected: _difficulty == 'Forgiving',
                                onPressed: () {
                                  setState(() => _difficulty = 'Forgiving');
                                },
                              ),
                            ),
                            const SizedBox(width: 12),
                            Expanded(
                              child: _ToggleButton(
                                label: 'BALANCED',
                                isSelected: _difficulty == 'Balanced',
                                onPressed: () {
                                  setState(() => _difficulty = 'Balanced');
                                },
                              ),
                            ),
                            const SizedBox(width: 12),
                            Expanded(
                              child: _ToggleButton(
                                label: 'HARSH',
                                isSelected: _difficulty == 'Harsh',
                                onPressed: () {
                                  setState(() => _difficulty = 'Harsh');
                                },
                              ),
                            ),
                          ],
                        ),
                      ],
                    ),
                    const SizedBox(height: 48),

                    // Begin Life button
                    ElevatedButton(
                      onPressed: _handleBeginLife,
                      style: ElevatedButton.styleFrom(
                        backgroundColor: const Color(0xFF00D9FF),
                        foregroundColor: const Color(0xFF0A0E27),
                        padding: const EdgeInsets.symmetric(
                          horizontal: 48,
                          vertical: 16,
                        ),
                      ),
                      child: Text(
                        'BEGIN LIFE',
                        style: Theme.of(context).textTheme.titleSmall?.copyWith(
                              color: const Color(0xFF0A0E27),
                            ),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _ToggleButton extends StatefulWidget {
  final String label;
  final bool isSelected;
  final VoidCallback onPressed;

  const _ToggleButton({
    required this.label,
    required this.isSelected,
    required this.onPressed,
    Key? key,
  }) : super(key: key);

  @override
  State<_ToggleButton> createState() => _ToggleButtonState();
}

class _ToggleButtonState extends State<_ToggleButton> {
  bool _isHovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _isHovered = true),
      onExit: (_) => setState(() => _isHovered = false),
      child: GestureDetector(
        onTap: widget.onPressed,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 150),
          padding: const EdgeInsets.symmetric(vertical: 12),
          decoration: BoxDecoration(
            border: Border.all(
              color: widget.isSelected
                  ? const Color(0xFF00D9FF)
                  : Colors.white.withOpacity(0.2),
              width: 2,
            ),
            color: widget.isSelected
                ? const Color(0xFF00D9FF).withOpacity(0.1)
                : (_isHovered
                    ? const Color(0xFF00D9FF).withOpacity(0.05)
                    : Colors.transparent),
          ),
          child: Center(
            child: Text(
              widget.label,
              style: Theme.of(context).textTheme.labelMedium?.copyWith(
                    color: widget.isSelected
                        ? const Color(0xFF00D9FF)
                        : Colors.white.withOpacity(0.7),
                  ),
            ),
          ),
        ),
      ),
    );
  }
}
