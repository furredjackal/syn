import 'package:flame/components.dart';
import 'package:flame/events.dart';
import 'package:flutter/material.dart';
import '../../../models/game_state.dart';
import '../../../syn_game.dart';
import '../syn_theme.dart';

class ChoiceButtonComponent extends PositionComponent
    with HasGameReference<SynGame>, TapCallbacks, HoverCallbacks {
  
  final GameChoice choice;
  final int index;
  final VoidCallback onPressed;

  bool _isHovered = false;
  bool _isPressed = false;
  double _animationProgress = 0.0; 
  double _opacity = 1.0; 

  final Paint _fillPaint = Paint()..style = PaintingStyle.fill;
  final Paint _strokePaint = Paint()
    ..style = PaintingStyle.stroke
    ..strokeWidth = 2.0
    ..color = SynColors.primaryCyan;
  
  final Paint _highlightPaint = Paint()
    ..style = PaintingStyle.stroke
    ..strokeWidth = 1.0
    ..color = SynColors.accentCyan.withValues(alpha: 0.5);

  final Path _bgPath = Path();

  late TextComponent _textComponent;
  late TextComponent _shortcutComponent;

  ChoiceButtonComponent({
    required this.choice,
    required this.index,
    required this.onPressed,
    super.position,
    super.size,
  });

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    
    _textComponent = TextComponent(
      text: choice.text.toUpperCase(),
      textRenderer: TextPaint(
        style: SynTextStyles.body.copyWith(
          fontWeight: FontWeight.w600, 
          letterSpacing: 1.0,
          fontSize: 16,
          color: Colors.white, 
        ),
      ),
      anchor: Anchor.centerLeft, 
      position: Vector2(40, size.y / 2), 
    );
    add(_textComponent);

    _shortcutComponent = TextComponent(
      text: '[${choice.keyboardShortcut}]',
      textRenderer: TextPaint(
        style: SynTextStyles.chip.copyWith(
            color: SynColors.accentCyan, 
            fontWeight: FontWeight.bold
        ),
      ),
      anchor: Anchor.centerRight,
      position: Vector2(size.x - 30, size.y / 2),
    );
    add(_shortcutComponent);

    _updatePath();
  }

  @override
  void onGameResize(Vector2 newSize) {
    super.onGameResize(newSize);
    _updatePath();
    if (isLoaded) {
       _textComponent.position = Vector2(40, size.y / 2);
       _shortcutComponent.position = Vector2(size.x - 30, size.y / 2);
    }
  }

  void _updatePath() {
    _bgPath.reset();
    const skew = 12.0;
    _bgPath
      ..moveTo(0, 0)
      ..lineTo(size.x - skew, 0)
      ..lineTo(size.x, size.y)
      ..lineTo(skew, size.y)
      ..close();
  }

  void setOpacity(double val) {
    _opacity = val.clamp(0.0, 1.0);
    
    if (_textComponent.textRenderer is TextPaint) {
        final tp = _textComponent.textRenderer as TextPaint;
        _textComponent.textRenderer = TextPaint(
            style: tp.style.copyWith(
                color: tp.style.color?.withValues(alpha: _opacity)
            )
        );
    }
     if (_shortcutComponent.textRenderer is TextPaint) {
        final tp = _shortcutComponent.textRenderer as TextPaint;
        _shortcutComponent.textRenderer = TextPaint(
            style: tp.style.copyWith(
                color: tp.style.color?.withValues(alpha: _opacity)
            )
        );
    }
  }

  @override
  void update(double dt) {
    super.update(dt);
    if (_isPressed) {
      _animationProgress = (_animationProgress + dt * 5).clamp(0.0, 1.0);
    } else {
      _animationProgress = (_animationProgress - dt * 5).clamp(0.0, 1.0);
    }
    
    final baseColor = const Color(0xFF1A1F2E); 
    final hoverColor = SynColors.primaryCyan.withValues(alpha: 0.25);

    _fillPaint.color = _isHovered 
        ? Color.lerp(baseColor, hoverColor, 0.5 + (_animationProgress * 0.5))!
              .withValues(alpha: _opacity)
        : baseColor.withValues(alpha: 0.8 * _opacity);
        
    _strokePaint.color = (_isHovered ? SynColors.primaryCyan : SynColors.textSubtle)
        .withValues(alpha: (_isHovered ? 1.0 : 0.3) * _opacity);
        
    _highlightPaint.color = SynColors.accentCyan.withValues(alpha: 0.5 * _opacity);
  }

  @override
  void render(Canvas canvas) {
    if (_opacity <= 0) return;

    canvas.drawPath(_bgPath, _fillPaint);
    canvas.drawPath(_bgPath, _strokePaint);
    
    if (_isHovered) {
      canvas.drawLine(
        Offset(15, size.y - 2), 
        Offset(size.x - 15, size.y - 2), 
        _highlightPaint
      );
    }
  }

  @override
  void onHoverEnter() {
    _isHovered = true;
    game.mouseCursor = SystemMouseCursors.click;
  }

  @override
  void onHoverExit() {
    _isHovered = false;
    _isPressed = false;
    game.mouseCursor = SystemMouseCursors.basic;
  }

  @override
  void onTapDown(TapDownEvent event) {
    _isPressed = true;
  }

  @override
  void onTapUp(TapUpEvent event) {
    _isPressed = false;
    simulateTap();
  }

  @override
  void onTapCancel(TapCancelEvent event) {
    _isPressed = false;
  }

  void simulateTap() {
    onPressed();
  }
}