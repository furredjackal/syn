import 'package:flutter/material.dart';
import '../models/game_state.dart';

class EventCard extends StatefulWidget {
  final GameEvent event;
  final Function(int) onChoice;

  const EventCard({
    required this.event,
    required this.onChoice,
    Key? key,
  }) : super(key: key);

  @override
  State<EventCard> createState() => _EventCardState();
}

class _EventCardState extends State<EventCard>
    with SingleTickerProviderStateMixin {
  late AnimationController _animationController;
  late Animation<double> _scaleAnimation;
  late Animation<double> _fadeAnimation;

  @override
  void initState() {
    super.initState();
    _animationController = AnimationController(
      duration: const Duration(milliseconds: 400),
      vsync: this,
    );

    _scaleAnimation = Tween<double>(begin: 0.8, end: 1.0).animate(
      CurvedAnimation(parent: _animationController, curve: Curves.easeOut),
    );

    _fadeAnimation = Tween<double>(begin: 0.0, end: 1.0).animate(
      CurvedAnimation(parent: _animationController, curve: Curves.easeOut),
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
    return ScaleTransition(
      scale: _scaleAnimation,
      child: FadeTransition(
        opacity: _fadeAnimation,
        child: Container(
          decoration: BoxDecoration(
            border: Border.all(
              color: const Color(0xFF00D9FF),
              width: 2,
            ),
            color: Colors.black.withOpacity(0.4),
          ),
          clipBehavior: Clip.hardEdge,
          child: SingleChildScrollView(
            padding: const EdgeInsets.all(24),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                // Title
                Text(
                  widget.event.title.toUpperCase(),
                  style: Theme.of(context).textTheme.displaySmall?.copyWith(
                        fontSize: 32,
                        color: const Color(0xFF00D9FF),
                      ),
                ),
                const SizedBox(height: 24),

                // Event description
                Text(
                  widget.event.description,
                  style: Theme.of(context).textTheme.bodyLarge,
                ),
                const SizedBox(height: 32),

                // Choices
                ...List.generate(
                  widget.event.choices.length,
                  (index) => Padding(
                    padding: const EdgeInsets.only(bottom: 16),
                    child: _ChoiceButton(
                      choice: widget.event.choices[index],
                      index: index,
                      onPressed: () {
                        widget.onChoice(index);
                      },
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

class _ChoiceButton extends StatefulWidget {
  final GameChoice choice;
  final int index;
  final VoidCallback onPressed;

  const _ChoiceButton({
    required this.choice,
    required this.index,
    required this.onPressed,
    Key? key,
  }) : super(key: key);

  @override
  State<_ChoiceButton> createState() => _ChoiceButtonState();
}

class _ChoiceButtonState extends State<_ChoiceButton>
    with SingleTickerProviderStateMixin {
  bool _isHovered = false;
  late AnimationController _pressController;

  @override
  void initState() {
    super.initState();
    _pressController = AnimationController(
      duration: const Duration(milliseconds: 300),
      vsync: this,
    );
  }

  @override
  void dispose() {
    _pressController.dispose();
    super.dispose();
  }

  void _handlePress() {
    _pressController.forward().then((_) {
      _pressController.reverse();
      widget.onPressed();
    });
  }

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _isHovered = true),
      onExit: (_) => setState(() => _isHovered = false),
      child: GestureDetector(
        onTap: _handlePress,
        child: AnimatedBuilder(
          animation: _pressController,
          builder: (context, child) {
            return Transform.scale(
              scale: 1.0 - (_pressController.value * 0.05),
              child: AnimatedContainer(
                duration: const Duration(milliseconds: 200),
                padding: const EdgeInsets.all(16),
                decoration: BoxDecoration(
                  border: Border.all(
                    color: _isHovered || _pressController.value > 0
                        ? const Color(0xFF00D9FF)
                        : const Color(0xFF00D9FF).withOpacity(0.3),
                    width: 2,
                  ),
                  color: _isHovered || _pressController.value > 0
                      ? const Color(0xFF00D9FF).withOpacity(0.1)
                      : Colors.transparent,
                  boxShadow: _isHovered
                      ? [
                          BoxShadow(
                            color: const Color(0xFF00D9FF).withOpacity(0.3),
                            blurRadius: 12,
                            spreadRadius: 1,
                          ),
                        ]
                      : [],
                ),
                child: Row(
                  mainAxisAlignment: MainAxisAlignment.spaceBetween,
                  children: [
                    // Choice text
                    Expanded(
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Text(
                            widget.choice.text.toUpperCase(),
                            style:
                                Theme.of(context).textTheme.bodyLarge?.copyWith(
                                      color: _isHovered
                                          ? const Color(0xFF00D9FF)
                                          : Colors.white,
                                    ),
                          ),
                          const SizedBox(height: 8),
                          _StatChangeIndicators(
                              statChanges: widget.choice.statChanges),
                        ],
                      ),
                    ),
                    const SizedBox(width: 16),
                    // Keyboard shortcut
                    Container(
                      width: 32,
                      height: 32,
                      decoration: BoxDecoration(
                        border: Border.all(
                          color: _isHovered || _pressController.value > 0
                              ? const Color(0xFF00D9FF)
                              : const Color(0xFF00D9FF).withOpacity(0.5),
                          width: 2,
                        ),
                        color: Colors.black.withOpacity(0.5),
                      ),
                      child: Center(
                        child: Text(
                          widget.choice.keyboardShortcut.toString(),
                          style:
                              Theme.of(context).textTheme.labelMedium?.copyWith(
                                    color: _isHovered
                                        ? const Color(0xFF00D9FF)
                                        : Colors.white.withOpacity(0.7),
                                  ),
                        ),
                      ),
                    ),
                  ],
                ),
              ),
            );
          },
        ),
      ),
    );
  }
}

class _StatChangeIndicators extends StatelessWidget {
  final Map<String, int> statChanges;

  const _StatChangeIndicators({
    required this.statChanges,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Wrap(
      spacing: 8,
      children: statChanges.entries.map((entry) {
        final isPositive = entry.value > 0;
        return Container(
          padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
          decoration: BoxDecoration(
            border: Border.all(
              color: isPositive
                  ? const Color(0xFF00FF00).withOpacity(0.5)
                  : const Color(0xFFFF0000).withOpacity(0.5),
            ),
            color: isPositive
                ? const Color(0xFF00FF00).withOpacity(0.1)
                : const Color(0xFFFF0000).withOpacity(0.1),
          ),
          child: Text(
            '${isPositive ? '+' : ''}${entry.value} ${entry.key}',
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  color: isPositive
                      ? const Color(0xFF00FF00)
                      : const Color(0xFFFF0000),
                  fontSize: 11,
                ),
          ),
        );
      }).toList(),
    );
  }
}
