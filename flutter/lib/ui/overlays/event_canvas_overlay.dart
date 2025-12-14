import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';
import '../../dev_tools/inspectable_mixin.dart';
import '../widgets/persona_container.dart';
import 'package:syn/models/game_state.dart';

/// Event Canvas Overlay - displays storylet events (Floating Canvas system)
///
/// Props:
/// - gameState: Live game state from Rust backend
/// - onChoiceSelected: Callback when a choice is made (storyletId, choiceId)
class EventCanvasOverlay extends StatefulWidget {
  final GameState gameState;
  final Future<void> Function(String storyletId, String choiceId) onChoiceSelected;

  const EventCanvasOverlay({
    super.key,
    required this.gameState,
    required this.onChoiceSelected,
  });

  @override
  State<EventCanvasOverlay> createState() => _EventCanvasOverlayState();
}

class _EventCanvasOverlayState extends State<EventCanvasOverlay> {
  int _selectedChoiceIndex = 0;
  final _o = InspectorOverrides.instance;

  @override
  void initState() {
    super.initState();
    _o.register('EventCanvasOverlay', {
      'backdropOpacity': 0.7,
      'widthFactor': 0.6,
      'heightFactor': 0.75,
      'padding': 40.0,
      'titleFontSize': 32.0,
      'descriptionFontSize': 16.0,
      'skew': -0.1,
      'borderWidth': 3.0,
      'choiceSpacing': 12.0,
    }, onUpdate: () => setState(() {}));
  }

  @override
  void dispose() {
    _o.unregister('EventCanvasOverlay');
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final currentEvent = widget.gameState.currentEvent;

    // Show nothing if no active event - let other UI be interactive
    if (currentEvent == null) {
      return const SizedBox.shrink();
    }

    final screenWidth = MediaQuery.of(context).size.width;
    final screenHeight = MediaQuery.of(context).size.height;
    
    // Read from overrides
    final backdropOpacity = _o.get('EventCanvasOverlay.backdropOpacity', 0.7);
    final widthFactor = _o.get('EventCanvasOverlay.widthFactor', 0.6);
    final heightFactor = _o.get('EventCanvasOverlay.heightFactor', 0.75);
    final padding = _o.get('EventCanvasOverlay.padding', 40.0);
    final titleFontSize = _o.get('EventCanvasOverlay.titleFontSize', 32.0);
    final descriptionFontSize = _o.get('EventCanvasOverlay.descriptionFontSize', 16.0);
    final skew = _o.get('EventCanvasOverlay.skew', -0.1);
    final borderWidth = _o.get('EventCanvasOverlay.borderWidth', 3.0);

    return Container(
      color: Colors.black.withValues(alpha: backdropOpacity),
      child: Center(
        child: Container(
          constraints: BoxConstraints(
            maxWidth: screenWidth * widthFactor,
            maxHeight: screenHeight * heightFactor,
          ),
          child: PersonaContainer(
            color: Colors.black.withValues(alpha: 0.95),
            borderColor: Colors.cyanAccent,
            borderWidth: borderWidth,
            skew: skew,
            child: Container(
              padding: EdgeInsets.all(padding),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                mainAxisSize: MainAxisSize.min,
                children: [
                  // Event Title
                  Text(
                    currentEvent.title,
                    style: TextStyle(
                      color: Colors.white,
                      fontSize: titleFontSize,
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
                        currentEvent.description,
                        style: TextStyle(
                          color: Colors.white.withValues(alpha: 0.9),
                          fontSize: descriptionFontSize,
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
                      currentEvent.choices.length,
                      (index) => Padding(
                        padding: const EdgeInsets.only(bottom: 12),
                        child: _buildChoiceButton(
                          eventId: currentEvent.id,
                          choice: currentEvent.choices[index],
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
    required String eventId,
    required GameChoice choice,
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
          // Call backend with storylet ID and choice text (used as ID)
          widget.onChoiceSelected(eventId, choice.text);
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
                    ],
                  ),
                ),

                // Stat impacts (if any)
                if (choice.statChanges.isNotEmpty)
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
                      choice.statChanges.entries
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

// Data models moved to game_state.dart (GameEvent and GameChoice)
