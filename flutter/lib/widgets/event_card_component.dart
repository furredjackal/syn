import 'package:flame/components.dart';
import 'package:flutter/material.dart';
import '../models/game_state.dart';
import '../syn_game.dart';
import 'choice_button_component.dart';

class EventCardComponent extends PositionComponent with HasGameRef<SynGame> {
  final GameEvent event;
  final Function(int) onChoice;
  late List<ChoiceButtonComponent> choiceButtons;
  double elapsedTime = 0;

  EventCardComponent({
    required this.event,
    required this.onChoice,
    Vector2? position,
    Vector2? size,
  }) : super(position: position, size: size);

  @override
  Future<void> onLoad() async {
    choiceButtons = [];

    // Add background
    final background = RectangleComponent(
      paint: Paint()..color = Colors.black.withOpacity(0.4),
      size: size,
    );
    add(background);

    // Add border
    add(_SlashBorderComponent(size: size));

    // Add slash transition
    add(_SlashTransition(
      size: size,
      duration: 0.4,
      isEntrance: true,
    ));

    // Title
    final title = TextComponent(
      text: event.title.toUpperCase(),
      textRenderer: TextPaint(
        style: const TextStyle(
          color: Color(0xFF00D9FF),
          fontSize: 32,
          fontWeight: FontWeight.w900,
          letterSpacing: 2,
        ),
      ),
      position: Vector2(24, 24),
    );
    add(title);

    // Description
    final description = TextComponent(
      text: event.description,
      textRenderer: TextPaint(
        style: const TextStyle(
          color: Colors.white,
          fontSize: 16,
        ),
      ),
      position: Vector2(24, 80),
    );
    add(description);

    // Add choice buttons (will animate in via update())
    double yOffset = 140;
    for (var i = 0; i < event.choices.length; i++) {
      final choice = event.choices[i];
      final choiceButton = ChoiceButtonComponent(
        choice: choice,
        index: i,
        onPressed: () => onChoice(i),
        position: Vector2(24, yOffset),
        size: Vector2(size.x - 48, 80),
      );

      // Create wrapper to control opacity via custom component
      final buttonWrapper = _AnimatingButtonWrapper(
        child: choiceButton,
        staggerDelay: 0.2 + (i * 0.1),
      );
      add(buttonWrapper);
      choiceButtons.add(choiceButton);

      yOffset += 96;
    }
  }

  @override
  void update(double dt) {
    super.update(dt);
    elapsedTime += dt;

    // Scale + fade entrance animation for the card itself
    if (elapsedTime < 0.3) {
      final progress = elapsedTime / 0.3;
      scale.setValues(0.9 + (progress * 0.1), 0.9 + (progress * 0.1));
    }
  }
}

/// Wrapper component to animate button entrance with stagger
class _AnimatingButtonWrapper extends PositionComponent {
  final ChoiceButtonComponent child;
  final double staggerDelay;
  double elapsedTime = 0;
  double fadeOpacity = 0;

  _AnimatingButtonWrapper({
    required this.child,
    required this.staggerDelay,
  }) : super(size: child.size, position: child.position);

  @override
  Future<void> onLoad() async {
    add(child);
  }

  @override
  void update(double dt) {
    super.update(dt);
    elapsedTime += dt;

    // Fade in after stagger delay
    if (elapsedTime < staggerDelay) {
      fadeOpacity = 0;
      child.scale.x = 0.8;
      child.scale.y = 0.8;
    } else if (elapsedTime < staggerDelay + 0.2) {
      // Animate fade-in over 0.2s with scale
      final fadeProgress = (elapsedTime - staggerDelay) / 0.2;
      fadeOpacity = fadeProgress.clamp(0.0, 1.0);
      child.scale.x = 0.8 + (fadeProgress * 0.2);
      child.scale.y = 0.8 + (fadeProgress * 0.2);
    } else {
      fadeOpacity = 1;
      child.scale.x = 1;
      child.scale.y = 1;
    }
  }

  @override
  void render(Canvas canvas) {
    canvas.saveLayer(
      Rect.fromLTWH(0, 0, size.x, size.y),
      Paint()..color = Colors.white.withOpacity(fadeOpacity),
    );
    super.render(canvas);
    canvas.restore();
  }
}

/// Diagonal slash border component (Persona-style)
class _SlashBorderComponent extends PositionComponent {
  _SlashBorderComponent({required Vector2 size}) : super(size: size);

  @override
  void render(Canvas canvas) {
    // Draw angular border (parallelogram)
    final path = Path()
      ..moveTo(8, 0)
      ..lineTo(size.x, 0)
      ..lineTo(size.x - 8, size.y)
      ..lineTo(0, size.y)
      ..close();

    canvas.drawPath(
      path,
      Paint()
        ..color = const Color(0xFF00D9FF)
        ..style = PaintingStyle.stroke
        ..strokeWidth = 3,
    );
  }
}

/// Slash transition effect - diagonal wipe from top-right
class _SlashTransition extends PositionComponent {
  final double duration;
  final bool isEntrance;
  double elapsedTime = 0;

  _SlashTransition({
    required Vector2 size,
    required this.duration,
    required this.isEntrance,
  }) : super(size: size);

  @override
  void update(double dt) {
    super.update(dt);
    elapsedTime += dt;

    if (elapsedTime >= duration) {
      removeFromParent();
    }
  }

  @override
  void render(Canvas canvas) {
    final progress = (elapsedTime / duration).clamp(0.0, 1.0);

    // Diagonal slash from right to left (or left to right on exit)
    final startX = isEntrance ? size.x : 0;
    final endX = isEntrance ? 0 : size.x;
    final currentX = startX + (endX - startX) * progress;

    // Create diagonal path (slash)
    final slashWidth = 40.0;
    final path = Path()
      ..moveTo(currentX - slashWidth, -size.y)
      ..lineTo(currentX + slashWidth, size.y * 2)
      ..lineTo(currentX, size.y * 2)
      ..lineTo(currentX - slashWidth * 0.5, -size.y)
      ..close();

    // Paint with gradient for smooth wipe effect
    canvas.drawPath(
      path,
      Paint()
        ..color = const Color(0xFF00D9FF).withOpacity(0.6)
        ..style = PaintingStyle.fill,
    );

    // Add glowing edge
    final edgePath = Path()
      ..moveTo(currentX, -size.y)
      ..lineTo(currentX, size.y * 2);

    canvas.drawPath(
      edgePath,
      Paint()
        ..color = const Color(0xFF00D9FF)
        ..strokeWidth = 3
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 8),
    );
  }
}
