import 'package:flame/components.dart';
import 'package:flutter/material.dart';

import '../../../theme/theme.dart';
import '../magnetic_dock_component.dart';
import '../paint/angled_panel.dart';
import 'stat_panel_component.dart';

class StatsDockComponent extends MagneticDockComponent {
  StatsDockComponent({int? priority})
      : super(
          side: DockSide.left,
          collapsedExtent: 40,
          expandedExtent: 300,
          dockThickness: 420,
          initialState: DockState.expanded,
          priority: priority,
        );

  late final StatPanelComponent _panel;

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    _panel = StatPanelComponent(
      gameState: game.gameState,
      onClose: collapse,
      position: Vector2.zero(),
      size: size,
    )
      ..anchor = Anchor.topLeft;
    add(_panel);
  }

  @override
  void onGameResize(Vector2 newSize) {
    super.onGameResize(newSize);
    _panel
      ..size = size
      ..onGameResize(size);
  }

  @override
  void render(Canvas canvas) {
    final mood = game.gameState.mood.toDouble();
    final karma = game.gameState.karma;
    final baseColor = MoodColors.forMood(mood).withValues(alpha: 0.12);
    final borderColor = MoodColors.forMood(mood).withValues(alpha: 0.6);

    drawAngledPanel(
      canvas,
      size.toRect(),
      fill: baseColor,
      border: borderColor,
      borderWidth: 2,
      cutTopRight: true,
      cutBottomRight: true,
      cutTopLeft: false,
      cutBottomLeft: false,
    );

    final karmaOverlay = karma > 60
        ? Colors.white.withValues(alpha: 0.08)
        : karma < -60
            ? const Color(0xFFCC0000).withValues(alpha: 0.12)
            : null;
    if (karmaOverlay != null) {
      drawAngledPanel(
        canvas,
        size.toRect(),
        fill: karmaOverlay,
        border: Colors.transparent,
        borderWidth: 0,
        cutTopRight: true,
        cutBottomRight: true,
      );
    }

    super.render(canvas);
  }
}
