import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';

import '../ui/widgets/persona_container.dart';

/// Character Creation Screen - Hybrid Architecture Flutter Widget
///
/// Allows player to:
/// - Choose an archetype (STORYTELLER, ANALYST, DREAMER, CHALLENGER)
/// - Enter their name
/// - Toggle SFW mode
/// - Select difficulty (FORGIVING, BALANCED, HARSH)
class CharacterCreationScreen extends StatefulWidget {
  final Function({
    required String name,
    required String archetype,
    required bool sfwMode,
    required String difficulty,
  }) onComplete;

  const CharacterCreationScreen({
    super.key,
    required this.onComplete,
  });

  @override
  State<CharacterCreationScreen> createState() =>
      _CharacterCreationScreenState();
}

class _CharacterCreationScreenState extends State<CharacterCreationScreen> {
  final TextEditingController _nameController = TextEditingController();
  
  int _selectedArchetype = 0;
  int _selectedDifficulty = 1; // BALANCED
  bool _sfwMode = true;

  final List<String> _archetypes = [
    'STORYTELLER',
    'ANALYST',
    'DREAMER',
    'CHALLENGER',
  ];

  final Map<String, String> _archetypeDescriptions = {
    'STORYTELLER': 'Values experiences and relationships above all else',
    'ANALYST': 'Seeks patterns, data, and understanding',
    'DREAMER': 'Creative and idealistic, chases possibilities',
    'CHALLENGER': 'Tests limits and embraces conflict',
  };

  final List<String> _difficultyLevels = [
    'FORGIVING',
    'BALANCED',
    'HARSH',
  ];

  @override
  void dispose() {
    _nameController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      color: Colors.black.withValues(alpha: 0.9),
      padding: const EdgeInsets.all(60),
      child: SingleChildScrollView(
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            // Title
            Text(
              'CHARACTER CREATION',
              textAlign: TextAlign.center,
              style: TextStyle(
                color: Colors.cyanAccent,
                fontSize: 48,
                fontWeight: FontWeight.w900,
                letterSpacing: 6,
              ),
            )
                .animate()
                .fadeIn(duration: 600.ms)
                .slideY(begin: -0.3, duration: 600.ms, curve: Curves.easeOutExpo),

            const SizedBox(height: 60),

            // Name Input
            _buildNameInput()
                .animate()
                .fadeIn(delay: 200.ms, duration: 600.ms)
                .slideX(begin: -0.2, duration: 600.ms, curve: Curves.easeOutExpo),

            const SizedBox(height: 40),

            // Archetype Selection
            Text(
              'SELECT ARCHETYPE',
              style: TextStyle(
                color: Colors.cyanAccent,
                fontSize: 20,
                fontWeight: FontWeight.w700,
                letterSpacing: 2,
              ),
            )
                .animate()
                .fadeIn(delay: 400.ms, duration: 600.ms),

            const SizedBox(height: 20),

            ...List.generate(_archetypes.length, (index) {
              return Padding(
                padding: const EdgeInsets.only(bottom: 16),
                child: _buildArchetypeOption(index)
                    .animate()
                    .fadeIn(delay: (600 + index * 100).ms, duration: 400.ms)
                    .slideX(
                      begin: -0.3,
                      duration: 600.ms,
                      curve: Curves.easeOutExpo,
                    ),
              );
            }),

            const SizedBox(height: 40),

            // Content Mode & Difficulty
            Row(
              children: [
                Expanded(
                  child: _buildContentModeToggle()
                      .animate()
                      .fadeIn(delay: 1000.ms, duration: 600.ms),
                ),
                const SizedBox(width: 20),
                Expanded(
                  child: _buildDifficultySelector()
                      .animate()
                      .fadeIn(delay: 1100.ms, duration: 600.ms),
                ),
              ],
            ),

            const SizedBox(height: 60),

            // Begin Life Button
            Center(
              child: _buildBeginButton()
                  .animate()
                  .fadeIn(delay: 1200.ms, duration: 600.ms)
                  .scale(
                    begin: const Offset(0.8, 0.8),
                    duration: 600.ms,
                    curve: Curves.easeOutBack,
                  ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildNameInput() {
    return PersonaContainer(
      color: Colors.black,
      child: Padding(
        padding: const EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'YOUR NAME',
              style: TextStyle(
                color: Colors.cyanAccent,
                fontSize: 16,
                fontWeight: FontWeight.w700,
                letterSpacing: 2,
              ),
            ),
            const SizedBox(height: 12),
            TextField(
              controller: _nameController,
              style: TextStyle(
                color: Colors.white,
                fontSize: 24,
                fontWeight: FontWeight.w700,
              ),
              decoration: InputDecoration(
                hintText: 'Enter your name...',
                hintStyle: TextStyle(
                  color: Colors.white.withValues(alpha: 0.3),
                  fontSize: 24,
                ),
                border: UnderlineInputBorder(
                  borderSide: BorderSide(color: Colors.cyanAccent, width: 2),
                ),
                enabledBorder: UnderlineInputBorder(
                  borderSide: BorderSide(
                    color: Colors.cyanAccent.withValues(alpha: 0.5),
                    width: 2,
                  ),
                ),
                focusedBorder: UnderlineInputBorder(
                  borderSide: BorderSide(color: Colors.cyanAccent, width: 3),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildArchetypeOption(int index) {
    final archetype = _archetypes[index];
    final isSelected = index == _selectedArchetype;

    return GestureDetector(
      onTap: () {
        setState(() {
          _selectedArchetype = index;
        });
      },
      child: MouseRegion(
        cursor: SystemMouseCursors.click,
        child: PersonaContainer(
          color: isSelected ? Colors.white : Colors.black,
          child: Padding(
            padding: const EdgeInsets.all(20),
            child: Row(
              children: [
                if (isSelected)
                  Icon(
                    Icons.check_circle,
                    color: Colors.black,
                    size: 32,
                  ),
                if (isSelected) const SizedBox(width: 16),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        archetype,
                        style: TextStyle(
                          color: isSelected ? Colors.black : Colors.white,
                          fontSize: 24,
                          fontWeight: FontWeight.w900,
                          letterSpacing: 2,
                        ),
                      ),
                      const SizedBox(height: 8),
                      Text(
                        _archetypeDescriptions[archetype] ?? '',
                        style: TextStyle(
                          color: isSelected
                              ? Colors.black.withValues(alpha: 0.7)
                              : Colors.white.withValues(alpha: 0.7),
                          fontSize: 14,
                        ),
                      ),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildContentModeToggle() {
    return PersonaContainer(
      color: Colors.black,
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'CONTENT MODE',
              style: TextStyle(
                color: Colors.cyanAccent,
                fontSize: 14,
                fontWeight: FontWeight.w700,
                letterSpacing: 2,
              ),
            ),
            const SizedBox(height: 12),
            Row(
              children: [
                Expanded(
                  child: Text(
                    'SFW Mode',
                    style: TextStyle(
                      color: Colors.white,
                      fontSize: 16,
                    ),
                  ),
                ),
                Switch(
                  value: _sfwMode,
                  onChanged: (value) {
                    setState(() {
                      _sfwMode = value;
                    });
                  },
                  activeColor: Colors.cyanAccent,
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildDifficultySelector() {
    return PersonaContainer(
      color: Colors.black,
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              'DIFFICULTY',
              style: TextStyle(
                color: Colors.cyanAccent,
                fontSize: 14,
                fontWeight: FontWeight.w700,
                letterSpacing: 2,
              ),
            ),
            const SizedBox(height: 12),
            ...List.generate(_difficultyLevels.length, (index) {
              final isSelected = index == _selectedDifficulty;
              return GestureDetector(
                onTap: () {
                  setState(() {
                    _selectedDifficulty = index;
                  });
                },
                child: MouseRegion(
                  cursor: SystemMouseCursors.click,
                  child: Padding(
                    padding: const EdgeInsets.symmetric(vertical: 4),
                    child: Row(
                      children: [
                        Icon(
                          isSelected
                              ? Icons.radio_button_checked
                              : Icons.radio_button_unchecked,
                          color: isSelected ? Colors.cyanAccent : Colors.white,
                          size: 20,
                        ),
                        const SizedBox(width: 12),
                        Text(
                          _difficultyLevels[index],
                          style: TextStyle(
                            color: isSelected ? Colors.cyanAccent : Colors.white,
                            fontSize: 16,
                            fontWeight:
                                isSelected ? FontWeight.w700 : FontWeight.normal,
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              );
            }),
          ],
        ),
      ),
    );
  }

  Widget _buildBeginButton() {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        onTap: _handleBeginLife,
        child: PersonaContainer(
          color: Colors.cyanAccent,
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 60, vertical: 20),
            child: Text(
              'BEGIN LIFE',
              style: TextStyle(
                color: Colors.black,
                fontSize: 28,
                fontWeight: FontWeight.w900,
                letterSpacing: 4,
              ),
            ),
          ),
        ),
      ),
    );
  }

  void _handleBeginLife() {
    final name = _nameController.text.trim();
    if (name.isEmpty) {
      // Show error or just use default
      debugPrint('[CharacterCreation] Name is required');
      return;
    }

    widget.onComplete(
      name: name,
      archetype: _archetypes[_selectedArchetype],
      sfwMode: _sfwMode,
      difficulty: _difficultyLevels[_selectedDifficulty],
    );
  }
}
