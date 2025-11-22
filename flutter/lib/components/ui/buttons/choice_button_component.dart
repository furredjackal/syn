import 'package:flame/components.dart';
import 'package:flutter/material.dart';
import '../../../models/game_state.dart';
import '../../../syn_game.dart';
import '../syn_theme.dart'; // Ensure you import your theme

/// Choice button rendered as a skewed Flame component.
class ChoiceButtonComponent extends PositionComponent
    with HasGameReference<SynGame> {
  final GameChoice choice;
  final int index;
  final VoidCallback onPressed;

  ChoiceButtonComponent({
    required this.choice,
    required this.index,
    required this.onPressed,
    super.position,
    super.size,
  });

  // State for interactivity
  bool _isHovered = false;
  double _pressAnimationValue = 0.0;
  double _pressAnimationDirection = 0.0;

  // Visual content
  late TextComponent _choiceText;
  late TextComponent _shortcutText;
  TextComponent? _effectText;

  // Cached Paints (Optimization)
  final Paint _fillPaint = Paint()..style = PaintingStyle.fill;
  final Paint _strokePaint = Paint()
    ..style = PaintingStyle.stroke
    ..strokeWidth = 2.6
    ..color = const Color(0xFF00D9FF);
  
  final Paint _glowPaint = Paint()
    ..style = PaintingStyle.stroke
    ..strokeWidth = 6
    ..color = const Color(0xFF00D9FF).withValues(alpha: 0.25)
    ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 6);

  final Paint _highlightPaint = Paint()
    ..style = PaintingStyle.stroke
    ..strokeWidth = 1.2
    ..color = const Color(0xFF7AFFFF).withValues(alpha: 0.35);

  final Paint _badgeFillPaint = Paint()
    ..color = Colors.black.withValues(alpha: 0.8)
    ..style = PaintingStyle.fill;

  final Paint _badgeStrokePaint = Paint()
    ..style = PaintingStyle.stroke
    ..strokeWidth = 2
    ..color = const Color(0xFF00D9FF);
    
  // Cached Path
  final Path _buttonPath = Path();
  final Path _badgePath = Path();

  static const double _paddingLeft = 28.0;
  static const double _paddingTop = 18.0;
  static const double _shortcutBadgeWidth = 46.0;
  static const double _shortcutBadgeHeight = 30.0;
  static const double _shortcutBadgeRightPadding = 18.0;
  static const double _buttonSkew = 22.0;

  @override
  Future<void> onLoad() async {
    // 1. Prepare Text
    _choiceText = TextComponent(
      text: choice.text,
      textRenderer: _choiceTextPaint(),
      position: Vector2(_paddingLeft, _paddingTop),
    );
    add(_choiceText);

    // 2. Prepare Effects Text (if any)
    final effectSummary = _prepareEffectSummary();
    if (effectSummary != null) {
      _effectText = TextComponent(
        text: effectSummary.$1,
        textRenderer: TextPaint(
          style: TextStyle(
            fontFamily: 'Montserrat',
            fontWeight: FontWeight.w600,
            fontSize: 14,
            color: effectSummary.$2,
            letterSpacing: 0.4,
          ),
        ),
        position: Vector2(
          _paddingLeft,
          _choiceText.position.y + _choiceText.size.y + 6,
        ),
      );
      add(_effectText!);
    }

    // 3. Prepare Shortcut Badge
    _updateBadgeGeometry(); // Pre-calculate badge path
    _shortcutText = TextComponent(
      text: choice.keyboardShortcut.toString(),
      textRenderer: _shortcutPaint(isActive: false),
      anchor: Anchor.center,
      position: _getBadgeCenter(), 
    );
    add(_shortcutText);
    
    // 4. Pre-calculate button path
    _updateButtonPath();
  }

  @override
  void onGameResize(Vector2 newSize) {
    super.onGameResize(newSize);
    _updateButtonPath();
    _updateBadgeGeometry();
    _shortcutText.position = _getBadgeCenter();
  }

  void _updateButtonPath() {
    _buttonPath.reset();
    _buttonPath
      ..moveTo(0, 0)
      ..lineTo(size.x - _buttonSkew, 0)
      ..lineTo(size.x, size.y)
      ..lineTo(0, size.y)
      ..close();
  }
  
  void _updateBadgeGeometry() {
    final left = size.x - _shortcutBadgeWidth - _shortcutBadgeRightPadding;
    const top = 12.0;
    const skew = 10.0;
    
    // We don't need a full Rect object, just the path points
    final right = left + _shortcutBadgeWidth;
    final bottom = top + _shortcutBadgeHeight;

    _badgePath.reset();
    _badgePath
      ..moveTo(left + skew, top)
      ..lineTo(right, top)
      ..lineTo(right - skew, bottom)
      ..lineTo(left, bottom)
      ..close();
  }

  Vector2 _getBadgeCenter() {
    final left = size.x - _shortcutBadgeWidth - _shortcutBadgeRightPadding;
    const top = 12.0;
    return Vector2(
      left + _shortcutBadgeWidth / 2,
      top + _shortcutBadgeHeight / 2,
    );
  }

  @override
  void update(double dt) {
    super.update(dt);

    // Animate press feedback
    if (_pressAnimationValue > 0.0 && _pressAnimationDirection < 0.0) {
      _pressAnimationValue -= dt * 3.0;
      if (_pressAnimationValue <= 0.0) {
        _pressAnimationValue = 0.0;
        _pressAnimationDirection = 0.0;
      }
    }

    final pressScale = 1.0 - (_pressAnimationValue * 0.05);
    scale.setValues(pressScale, pressScale);
    
    // Update fill color only (much cheaper than re-creating paint)
    _fillPaint.color = _calculateFillColor();
    
    // Update text style only if state changed
    _updateVisualState();
  }

  Color _calculateFillColor() {
    var fill = const Color(0xFF050608);
    if (_isHovered) {
      fill = const Color(0xFF0E141F);
    }
    if (_pressAnimationValue > 0.0) {
      fill = fill.withValues(alpha: 0.9);
    }
    return fill;
  }

  void _updateVisualState() {
    final isActive = _isHovered || _pressAnimationValue > 0.0;
    // Only update the renderer if needed (optimization)
    // Note: TextRenderer update is somewhat heavy, do it only on state change if possible
    _shortcutText.textRenderer = _shortcutPaint(isActive: isActive);
  }

  void simulateTap() {
    _pressAnimationValue = 1.0;
    _pressAnimationDirection = -1.0;
    Future.delayed(const Duration(milliseconds: 150), onPressed);
  }

  void setHovered(bool hovered) {
    if (_isHovered != hovered) {
      _isHovered = hovered;
      // Trigger visual update
    }
  }

  TextPaint _choiceTextPaint() {
    return TextPaint(
      style: TextStyle(
        fontFamily: 'Montserrat',
        fontWeight: FontWeight.w600,
        fontSize: 20,
        letterSpacing: 0.5,
        color: Colors.white.withValues(alpha: 0.95),
      ),
    );
  }

  TextPaint _shortcutPaint({required bool isActive}) {
    return TextPaint(
      style: TextStyle(
        fontFamily: 'Montserrat',
        fontWeight: FontWeight.w600,
        fontSize: 14,
        color: isActive ? const Color(0xFF00D9FF) : Colors.white.withValues(alpha: 0.7),
      ),
    );
  }

  (String, Color)? _prepareEffectSummary() {
    if (choice.statChanges.isEmpty) return null;

    final buffer = <String>[];
    var hasPositive = false;
    var hasNegative = false;

    choice.statChanges.forEach((stat, delta) {
      if (delta > 0) hasPositive = true;
      if (delta < 0) hasNegative = true;
      
      final prefix = delta > 0 ? '+' : '';
      final name = stat.isEmpty
          ? stat
          : stat[0].toUpperCase() + (stat.length > 1 ? stat.substring(1).toLowerCase() : '');
      buffer.add('$prefix$delta $name');
    });

    final color = (hasPositive && !hasNegative) 
        ? const Color(0xFF4CFF88) 
        : (hasNegative && !hasPositive) ? const Color(0xFFFF6B7C) : const Color(0xFF00D9FF);

    return (buffer.join('   '), color);
  }

  @override
  void render(Canvas canvas) {
    // 1. Draw Button Body
    canvas.drawPath(_buttonPath, _fillPaint);

    if (_isHovered) {
      canvas.drawPath(_buttonPath, _glowPaint);
    }

    canvas.drawPath(_buttonPath, _strokePaint);
    canvas.drawPath(_buttonPath, _highlightPaint);

    // 2. Draw Badge
    canvas.drawPath(_badgePath, _badgeFillPaint);
    canvas.drawPath(_badgePath, _badgeStrokePaint);

    // 3. Children (Text) render automatically via super.render
  }
}