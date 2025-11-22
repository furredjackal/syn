import 'package:flame/components.dart';
import 'package:flutter/material.dart';

import '../../../models/game_state.dart';
import '../../../syn_game.dart';
import '../../../ui/ui_signal_bus.dart';
import '../buttons/icon_button_component.dart';
import '../display/stat_bar_component.dart';
import '../paint/angled_panel.dart';
import '../syn_theme.dart';

enum PanelMode { compact, detailed }

class StatPanelComponent extends PositionComponent 
    with HasGameReference<SynGame>, HasPaint { 
    
  StatPanelComponent({
    required this.gameState,
    this.onClose,
    super.position,
    super.size,
  });

  final GameState gameState;
  final VoidCallback? onClose;
  
  PanelMode _mode = PanelMode.compact;
  
  final Paint _bgPaint = Paint();
  final Paint _borderPaint = Paint()
    ..style = PaintingStyle.stroke
    ..strokeWidth = SynLayout.borderWidthHeavy
    ..color = SynHudChrome.topBarBorderColorPrimary;
    
  final Path _clipPath = Path();

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    _buildCompactLayout();
  }

  void setMode(PanelMode mode, {Vector2? availableSize}) {
    if (_mode == mode) return;
    _mode = mode;

    if (mode == PanelMode.detailed && availableSize != null) {
      size = availableSize;
      _buildDetailedLayout();
    } else {
      size = Vector2(300, 400); 
      _buildCompactLayout();
    }
    
    _updateBackgroundPath();
  }

  void _buildCompactLayout() {
    removeAll(children);
    final stats = _buildStats();
    
    var y = 24.0;
    for (final stat in stats) {
      add(_StatRow(label: stat.$1, value: stat.$2, rawValue: stat.$3)
        ..position = Vector2(20, y)
        ..size = Vector2(size.x - 40, 52));
      y += 58;
    }
    _updateBackgroundPath();
  }

  // This method must stay INSIDE StatPanelComponent to access 'onClose'
  void _buildDetailedLayout() {
    removeAll(children);
    
    // 1. Close Button (Top Right)
    add(IconButtonComponent(
      materialIcon: Icons.close, 
      iconColor: SynColors.primaryCyan,
      onTap: () => onClose?.call(), // Accesses the class variable
    )
      ..size = Vector2(32, 32)
      ..anchor = Anchor.topRight
      ..position = Vector2(size.x - 20, 20));

    // 2. Title
    add(TextComponent(
      text: 'NEURAL METRICS',
      textRenderer: TextPaint(style: SynTextStyles.h1Event.copyWith(fontSize: 24)),
    )..position = Vector2(40, 30));

    // 3. Grid Layout for Stats
    final stats = _buildStats();
    const colCount = 2;
    final colWidth = (size.x - 80) / colCount;
    const rowHeight = 80.0;
    const startY = 100.0;

    for (int i = 0; i < stats.length; i++) {
      final col = i % colCount;
      final row = i ~/ colCount;
      
      final x = 40 + (col * colWidth);
      final y = startY + (row * rowHeight);

      add(_DetailedStatModule(
        label: stats[i].$1,
        value: stats[i].$2,
        rawValue: stats[i].$3,
      )
        ..position = Vector2(x, y)
        ..size = Vector2(colWidth - 20, 60)
      );
    }
    _updateBackgroundPath();
  }

  List<(String, double, int)> _buildStats() {
    return [
      ('HEALTH', gameState.health / 100, gameState.health),
      ('WEALTH', gameState.wealth / 100, gameState.wealth),
      ('CHARISMA', gameState.charisma / 100, gameState.charisma),
      ('INTELLECT', gameState.intelligence / 100, gameState.intelligence),
      ('STABILITY', gameState.stability / 100, gameState.stability),
      ('CREATIVITY', gameState.wisdom / 100, gameState.wisdom),
    ];
  }

  @override
  void onGameResize(Vector2 newSize) {
    super.onGameResize(newSize);
    if (_mode == PanelMode.compact) {
        _updateBackgroundPath();
    }
  }
  
  void _updateBackgroundPath() {
    _clipPath.reset();
    if (_mode == PanelMode.compact) {
        _clipPath
          ..moveTo(16, 0)
          ..lineTo(size.x - 10, 6)
          ..lineTo(size.x, size.y * 0.25)
          ..lineTo(size.x, size.y - 10)
          ..lineTo(size.x - 16, size.y)
          ..lineTo(10, size.y - 6)
          ..lineTo(0, size.y * 0.15)
          ..close();
    } else {
        _clipPath.addRect(size.toRect());
    }

    _bgPaint.shader = LinearGradient(
      colors: [SynColors.bgDark.withOpacity(0.9), SynColors.bgPanel.withOpacity(0.95)],
      begin: Alignment.topLeft,
      end: Alignment.bottomRight,
    ).createShader(size.toRect());
  }

  @override
  void render(Canvas canvas) {
    drawAngledPanel(
      canvas,
      size.toRect(),
      fill: SynColors.bgDark.withValues(alpha: 0.85),
      border: SynHudChrome.topBarBorderColorPrimary,
      borderWidth: SynLayout.borderWidthHeavy,
      cutTopRight: true,
      cutBottomLeft: true,
      cutTopLeft: false,
      cutBottomRight: false,
    );
    canvas.drawPath(_clipPath, _bgPaint);
    canvas.drawPath(_clipPath, _borderPaint);
  }
} // End of StatPanelComponent

// Helper classes are defined OUTSIDE the main class

class _StatRow extends PositionComponent {
  _StatRow({required this.label, required this.value, required this.rawValue});
  final String label;
  final double value;
  final int rawValue;

  @override
  Future<void> onLoad() async {
    add(TextComponent(
      text: label,
      textRenderer: TextPaint(style: SynTextStyles.h2Strip.copyWith(fontSize: 13)),
    ));

    add(StatBarComponent(
      value: value,
      position: Vector2(0, 24),
      size: Vector2(size.x, 10),
    ));

    add(TextComponent(
      text: rawValue.toString(),
      textRenderer: TextPaint(
        style: SynTextStyles.body.copyWith(
          color: SynColors.textSubtle, 
          fontSize: 14, 
          fontWeight: FontWeight.w600
        ),
      ),
      anchor: Anchor.topRight,
      position: Vector2(size.x, 0),
    ));
  }
}

class _DetailedStatModule extends PositionComponent {
   _DetailedStatModule({required this.label, required this.value, required this.rawValue});
   final String label;
   final double value;
   final int rawValue;
   
   @override
   Future<void> onLoad() async {
      add(TextComponent(text: label, textRenderer: TextPaint(style: SynTextStyles.h2Strip)));
      add(StatBarComponent(value: value)..position = Vector2(0, 25)..size = Vector2(size.x, 16));
      add(TextComponent(
          text: '$rawValue / 100', 
          textRenderer: TextPaint(style: SynTextStyles.chip.copyWith(color: SynColors.primaryCyan, fontSize: 16)),
          anchor: Anchor.topRight
      )..position = Vector2(size.x, 0));
   }
}
