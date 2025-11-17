import 'package:flame/components.dart';
import 'package:flutter/material.dart';
import '../models/game_state.dart';
import '../syn_game.dart';

/// Persona-style choice button rendered as a skewed Flame component.
class ChoiceButtonComponent extends PositionComponent
    with HasGameReference<SynGame> {
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

  // Visual content
  late TextComponent _choiceText;
  late TextComponent _shortcutText;
  TextComponent? _effectText;
  String? _effectSummaryText;
  Color _effectSummaryColor = const Color(0xFF00D9FF);

  static const double _paddingLeft = 28.0;
  static const double _paddingTop = 18.0;
  static const double _shortcutBadgeWidth = 46.0;
  static const double _shortcutBadgeHeight = 30.0;
  static const double _shortcutBadgeRightPadding = 18.0;
  static const double _buttonSkew = 22.0;

  @override
  Future<void> onLoad() async {
    _prepareEffectSummary();

    _choiceText = TextComponent(
      text: choice.text,
      textRenderer: _choiceTextPaint(),
      position: Vector2(_paddingLeft, _paddingTop),
    );
    await add(_choiceText);

    if (_effectSummaryText != null) {
      _effectText = TextComponent(
        text: _effectSummaryText!,
        textRenderer: TextPaint(
          style: TextStyle(
            fontFamily: 'Montserrat',
            fontWeight: FontWeight.w600,
            fontSize: 14,
            color: _effectSummaryColor,
            letterSpacing: 0.4,
          ),
        ),
        position: Vector2(
          _paddingLeft,
          _choiceText.position.y + _choiceText.size.y + 6,
        ),
      );
      await add(_effectText!);
    }

    final badgeRect = _shortcutBadgeRect();

    _shortcutText = TextComponent(
      text: choice.keyboardShortcut.toString(),
      textRenderer: _shortcutPaint(isActive: false),
      position: Vector2(
        badgeRect.left + badgeRect.width / 2,
        badgeRect.top + badgeRect.height / 2,
      ),
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

    _updateVisualState();
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
    _updateVisualState();
  }

  TextPaint _choiceTextPaint() {
    return TextPaint(
      style: TextStyle(
        fontFamily: 'Montserrat',
        fontWeight: FontWeight.w600,
        fontSize: 20,
        letterSpacing: 0.5,
        color: Colors.white.withOpacity(0.95),
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

  void _prepareEffectSummary() {
    if (choice.statChanges.isEmpty) {
      _effectSummaryText = null;
      return;
    }

    final buffer = <String>[];
    var hasPositive = false;
    var hasNegative = false;

    choice.statChanges.forEach((stat, delta) {
      if (delta > 0) {
        hasPositive = true;
      } else if (delta < 0) {
        hasNegative = true;
      }
      final prefix = delta > 0 ? '+' : '';
      final name = stat.isEmpty
          ? stat
          : stat[0].toUpperCase() +
              (stat.length > 1 ? stat.substring(1).toLowerCase() : '');
      buffer.add('$prefix$delta $name');
    });

    if (hasPositive && !hasNegative) {
      _effectSummaryColor = const Color(0xFF4CFF88);
    } else if (hasNegative && !hasPositive) {
      _effectSummaryColor = const Color(0xFFFF6B7C);
    } else {
      _effectSummaryColor = const Color(0xFF00D9FF);
    }

    _effectSummaryText = buffer.join('   ');
  }

  void _updateVisualState() {
    final isActive = _isHovered || _pressAnimationValue > 0.0;
    _shortcutText.textRenderer = _shortcutPaint(isActive: isActive);
  }

  Path _buildButtonPath() {
    final path = Path()
      ..moveTo(0, 0)
      ..lineTo(size.x - _buttonSkew, 0)
      ..lineTo(size.x, size.y)
      ..lineTo(0, size.y)
      ..close();
    return path;
  }

  Rect _shortcutBadgeRect() {
    final left = size.x - _shortcutBadgeWidth - _shortcutBadgeRightPadding;
    const top = 12.0;
    return Rect.fromLTWH(left, top, _shortcutBadgeWidth, _shortcutBadgeHeight);
  }

  Color _buttonFillColor() {
    var fill = const Color(0xFF050608);
    if (_isHovered) {
      fill = const Color(0xFF0E141F);
    }
    if (_pressAnimationValue > 0.0) {
      fill = fill.withOpacity(0.9);
    }
    return fill;
  }

  void _renderShortcutBadge(Canvas canvas) {
    final rect = _shortcutBadgeRect();
    const skew = 10.0;

    final badgePath = Path()
      ..moveTo(rect.left + skew, rect.top)
      ..lineTo(rect.right, rect.top)
      ..lineTo(rect.right - skew, rect.bottom)
      ..lineTo(rect.left, rect.bottom)
      ..close();

    canvas.drawPath(
      badgePath,
      Paint()
        ..color = Colors.black.withOpacity(0.8)
        ..style = PaintingStyle.fill,
    );

    canvas.drawPath(
      badgePath,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = const Color(0xFF00D9FF),
    );
  }

  @override
  void render(Canvas canvas) {
    final buttonPath = _buildButtonPath();

    canvas.drawPath(
      buttonPath,
      Paint()
        ..color = _buttonFillColor()
        ..style = PaintingStyle.fill,
    );

    if (_isHovered) {
      canvas.drawPath(
        buttonPath,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 6
          ..color = const Color(0xFF00D9FF).withOpacity(0.25)
          ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 6),
      );
    }

    canvas.drawPath(
      buttonPath,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2.6
        ..color = const Color(0xFF00D9FF),
    );

    canvas.drawPath(
      buttonPath,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.2
        ..color = const Color(0xFF7AFFFF).withOpacity(0.35),
    );

    _renderShortcutBadge(canvas);

    super.render(canvas);
  }
}
