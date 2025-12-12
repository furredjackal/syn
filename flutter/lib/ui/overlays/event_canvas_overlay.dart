import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';
import '../widgets/persona_container.dart';

/// Event Canvas Overlay - displays storylet events (Floating Canvas system)
///
/// Props:
/// - event: Current event data
/// - onChoiceSelect: Callback when a choice is made
/// - onDismiss: Callback to dismiss the event
class EventCanvasOverlay extends StatefulWidget {
  final StoryletEvent? event;
  final Function(int choiceIndex)? onChoiceSelect;
  final VoidCallback? onDismiss;

  const EventCanvasOverlay({
    super.key,
    this.event,
    this.onChoiceSelect,
    this.onDismiss,
  });

  @override
  State<EventCanvasOverlay> createState() => _EventCanvasOverlayState();
}

class _EventCanvasOverlayState extends State<EventCanvasOverlay> {
  int _selectedChoiceIndex = 0;

  @override
  Widget build(BuildContext context) {
    if (widget.event == null) return const SizedBox.shrink();

    final screenWidth = MediaQuery.of(context).size.width;
    final screenHeight = MediaQuery.of(context).size.height;

    return Container(
      color: Colors.black.withValues(alpha: 0.7),
      child: Center(
        child: Container(
          constraints: BoxConstraints(
            maxWidth: screenWidth * 0.6,
            maxHeight: screenHeight * 0.75,
          ),
          child: PersonaContainer(
            color: Colors.black.withValues(alpha: 0.95),
            borderColor: Colors.cyanAccent,
            borderWidth: 3,
            skew: -0.1,
            child: Container(
              padding: const EdgeInsets.all(40),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: [
                  // Event Title
                  Text(
                    widget.event!.title,
                    style: const TextStyle(
                      color: Colors.white,
                      fontSize: 32,
                      fontWeight: FontWeight.w900,
                      letterSpacing: 2,
                    ),
                  )
                      .animate()
                      .fadeIn(duration: 400.ms)
                      .slideX(
                          begin: -0.2,
                          duration: 400.ms,
                          curve: Curves.easeOut),

                  const SizedBox(height: 24),

                  // Event Description
                  Expanded(
                    child: SingleChildScrollView(
                      child: Text(
                        widget.event!.description,
                        style: TextStyle(
                          color: Colors.white.withValues(alpha: 0.9),
                          fontSize: 16,
                          height: 1.6,
                          letterSpacing: 0.5,
                        ),
                      )
                          .animate()
                          .fadeIn(delay: 200.ms, duration: 600.ms),
                    ),
                  ),

                  const SizedBox(height: 32),

                  // Choices
                  Column(
                    crossAxisAlignment: CrossAxisAlignment.stretch,
                    children: List.generate(
                      widget.event!.choices.length,
                      (index) => Padding(
                        padding: const EdgeInsets.only(bottom: 12),
                        child: _buildChoiceButton(
                          choice: widget.event!.choices[index],
                          index: index,
                          isSelected: index == _selectedChoiceIndex,
                        )
                            .animate(delay: (300 + index * 100).ms)
                            .fadeIn(duration: 400.ms)
                            .slideX(
                                begin: -0.1,
                                duration: 400.ms,
                                curve: Curves.easeOut),
                      ),
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildChoiceButton({
    required EventChoice choice,
    required int index,
    required bool isSelected,
  }) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) {
        setState(() {
          _selectedChoiceIndex = index;
        });
      },
      child: GestureDetector(
        onTap: () {
          widget.onChoiceSelect?.call(index);
        },
        child: PersonaContainer(
          color: isSelected ? Colors.white : Colors.black,
          borderColor: isSelected
              ? Colors.cyanAccent
              : Colors.white.withValues(alpha: 0.3),
          borderWidth: isSelected ? 3 : 2,
          skew: -0.15,
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 32, vertical: 20),
            child: Row(
              children: [
                // Choice indicator
                if (isSelected)
                  Container(
                    width: 8,
                    height: 8,
                    margin: const EdgeInsets.only(right: 16),
                    decoration: const BoxDecoration(
                      color: Colors.cyanAccent,
                      shape: BoxShape.circle,
                    ),
                  )
                else
                  const SizedBox(width: 24),

                // Choice text
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        choice.text,
                        style: TextStyle(
                          color: isSelected ? Colors.black : Colors.white,
                          fontSize: 16,
                          fontWeight: FontWeight.w700,
                          letterSpacing: 0.5,
                        ),
                      ),
                      if (choice.consequence.isNotEmpty) ...[
                        const SizedBox(height: 6),
                        Text(
                          choice.consequence,
                          style: TextStyle(
                            color: isSelected
                                ? Colors.black.withValues(alpha: 0.6)
                                : Colors.white.withValues(alpha: 0.5),
                            fontSize: 13,
                            fontStyle: FontStyle.italic,
                          ),
                        ),
                      ],
                    ],
                  ),
                ),

                // Stat impacts (if any)
                if (choice.statImpacts.isNotEmpty)
                  Container(
                    margin: const EdgeInsets.only(left: 16),
                    padding: const EdgeInsets.symmetric(
                      horizontal: 12,
                      vertical: 6,
                    ),
                    decoration: BoxDecoration(
                      color: isSelected
                          ? Colors.black.withValues(alpha: 0.2)
                          : Colors.white.withValues(alpha: 0.1),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    child: Text(
                      choice.statImpacts.entries
                          .map((e) => '${e.key}: ${e.value > 0 ? '+' : ''}${e.value}')
                          .join(', '),
                      style: TextStyle(
                        color: isSelected
                            ? Colors.black.withValues(alpha: 0.7)
                            : Colors.cyanAccent,
                        fontSize: 12,
                        fontWeight: FontWeight.w600,
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

/// Data model for storylet events
class StoryletEvent {
  final String id;
  final String title;
  final String description;
  final List<EventChoice> choices;

  const StoryletEvent({
    required this.id,
    required this.title,
    required this.description,
    required this.choices,
  });
}

/// Data model for event choices
class EventChoice {
  final String text;
  final String consequence;
  final Map<String, int> statImpacts;

  const EventChoice({
    required this.text,
    this.consequence = '',
    this.statImpacts = const {},
  });
}
