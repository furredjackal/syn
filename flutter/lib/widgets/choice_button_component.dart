import 'package:flame/components.dart';
import 'package:flutter/material.dart';
import '../models/game_state.dart';
import '../syn_game.dart';
import 'stat_change_indicators_component.dart';

/// Flame component for displaying a choice button with interactive feedback.
///
/// Features:
/// - Cyan border styling that brightens on hover/press
/// - Semi-transparent background that appears on hover/press
/// - Scale animation on press (1.0 → 0.95 → 1.0 over 300ms)
/// - Keyboard shortcut display in top-right box
/// - Stat change indicators below choice text
class ChoiceButtonComponent extends PositionComponent with HasGameReference<SynGame> {
  final GameChoice choice;
  final int index;
  final VoidCallback onPressed;

  ChoiceButtonComponent({
    required this.choice,
    required this.index,
    required this.onPressed,
    Vector2? position,
    Vector2? size,
  }) : super(position: position, size: size);

  // State for interactivity
  bool _isHovered = false;
  double _pressAnimationValue = 0.0; // 0.0 to 1.0
  double _pressAnimationDirection = 0.0; // For animating press feedback

  late RectangleComponent _background;
  late RectangleComponent _borderComponent;
  late RectangleComponent _shortcutBox;
  late TextComponent _choiceText;
  late TextComponent _shortcutText;
  late StatChangeIndicatorsComponent _statChanges;

  @override
  Future<void> onLoad() async {
    _background = RectangleComponent(
      paint: Paint()..color = Colors.black.withValues(alpha: 0.3),
      size: size,
    );
    add(_background);

    _borderComponent = RectangleComponent(
      paint: Paint()
        ..color = Colors.transparent
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2,
      size: size,
    );
    add(_borderComponent);

    _choiceText = TextComponent(
      text: choice.text,
      textRenderer: _choiceTextPaint(isActive: false),
      position: Vector2(24, 14),
    );
    await add(_choiceText);

    _statChanges = StatChangeIndicatorsComponent(
      statChanges: choice.statChanges,
      position: Vector2(
        _choiceText.position.x + 12,
        _choiceText.position.y + _choiceText.size.y + 6,
      ),
    );
    add(_statChanges);

    _shortcutBox = RectangleComponent(
      paint: Paint()
        ..color = Colors.black.withValues(alpha: 0.5)
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2,
      size: Vector2(32, 32),
      position: Vector2(size.x - 48, 10),
    );
    add(_shortcutBox);

    _shortcutText = TextComponent(
      text: choice.keyboardShortcut.toString(),
      textRenderer: _shortcutPaint(isActive: false),
      position: Vector2(size.x - 32, 26),
      anchor: Anchor.center,
    );
    add(_shortcutText);
  }

  @override
  void update(double dt) {
    super.update(dt);

    // Animate press feedback (scale effect)
    if (_pressAnimationValue > 0.0 && _pressAnimationDirection < 0.0) {
      // Animating back to normal after press
      _pressAnimationValue -= dt * 3.0; // 3.0 units/sec = ~333ms animation
      if (_pressAnimationValue <= 0.0) {
        _pressAnimationValue = 0.0;
        _pressAnimationDirection = 0.0;
      }
    }

    // Update scale based on press animation
    final pressScale = 1.0 - (_pressAnimationValue * 0.05);
    scale.setValues(pressScale, pressScale);

    // Update visual feedback colors based on hover/press state
    _updateHoverColors();
  }

  /// Update button colors based on hover and press state
  void _updateHoverColors() {
    final isActive = _isHovered || _pressAnimationValue > 0.0;
    final borderColor = isActive
        ? const Color(0xFF00D9FF)
        : const Color(0xFF00D9FF).withValues(alpha: 0.3);
    final backgroundColor = isActive
        ? const Color(0xFF00D9FF).withValues(alpha: 0.1)
        : Colors.black.withValues(alpha: 0.3);

    // Update border
    _borderComponent.paint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2
      ..color = borderColor;

    // Update background
    _background.paint = Paint()..color = backgroundColor;

    // Update text color
    _choiceText.textRenderer = _choiceTextPaint(isActive: isActive);

    // Update shortcut box and text colors
    _shortcutBox.paint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2
      ..color = borderColor;

    _shortcutText.textRenderer = _shortcutPaint(isActive: isActive);
  }

  /// Trigger press animation when user selects this choice
  void simulateTap() {
    // Start press animation
    _pressAnimationValue = 1.0;
    _pressAnimationDirection = -1.0; // Animating downward (releasing)

    // Fire the callback after a short delay to show animation
    Future.delayed(const Duration(milliseconds: 150), onPressed);
  }

  /// Check if a point is within this component's bounds
  @override
  bool containsPoint(Vector2 point) {
    return point.x >= 0 &&
        point.x <= size.x &&
        point.y >= 0 &&
        point.y <= size.y;
  }

  /// Handle tap at a local coordinate
  void handleTap(Vector2 tapPosition) {
    if (containsPoint(tapPosition)) {
      simulateTap();
    }
  }

  /// Set hover state (can be called from parent during collision detection)
  void setHovered(bool hovered) {
    _isHovered = hovered;
  }

  TextPaint _choiceTextPaint({required bool isActive}) {
    return TextPaint(
      style: TextStyle(
        fontFamily: 'Montserrat',
        fontWeight: FontWeight.w500,
        fontSize: 20,
        color: isActive
            ? const Color(0xFF00D9FF)
            : Colors.white.withValues(alpha: 0.9),
      ),
    );
  }

  TextPaint _shortcutPaint({required bool isActive}) {
    return TextPaint(
      style: TextStyle(
        fontFamily: 'Montserrat',
        fontWeight: FontWeight.w600,
        fontSize: 14,
        color:
            isActive ? const Color(0xFF00D9FF) : Colors.white.withValues(alpha: 0.7),
      ),
    );
  }
}
