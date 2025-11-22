import 'package:flame/components.dart';
import 'package:flutter/material.dart';

import '../../../theme/theme.dart';
import '../magnetic_dock_component.dart';
import '../paint/angled_panel.dart';
import 'quick_panel_component.dart';

class QuickActionsDockComponent extends MagneticDockComponent {
  QuickActionsDockComponent({int? priority})
      : super(
          side: DockSide.bottom,
          collapsedExtent: 56,
          expandedExtent: 120,
          dockThickness: 120,
          initialState: DockState.expanded,
          collapseOnExitHover: false,
          priority: priority,
        );

  late final QuickMenuBarComponent _quickMenu;

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    _quickMenu = QuickMenuBarComponent(
      onMemory: game.showMemoryJournal,
      onMap: game.showWorldMap,
      onPossessions: game.showPossessions,
      position: Vector2.zero(),
      size: size,
    )
      ..anchor = Anchor.topLeft;
    add(_quickMenu);
  }

  @override
  void onGameResize(Vector2 newSize) {
    super.onGameResize(newSize);
    _quickMenu
      ..size = size
      ..onGameResize(size);
  }

  @override
  void render(Canvas canvas) {
    final mood = game.gameState.mood.toDouble();
    final karma = game.gameState.karma;
    final baseColor = MoodColors.forMood(mood).withValues(alpha: 0.1);
    final borderColor = MoodColors.forMood(mood).withValues(alpha: 0.5);

    drawAngledPanel(
      canvas,
      size.toRect(),
      fill: baseColor,
      border: borderColor,
      borderWidth: 2,
      cutTopLeft: true,
      cutTopRight: true,
      cutBottomLeft: false,
      cutBottomRight: false,
    );

    final karmaOverlay = karma > 60
        ? Colors.white.withValues(alpha: 0.08)
        : karma < -60
            ? const Color(0xFFCC0000).withValues(alpha: 0.1)
            : null;
    if (karmaOverlay != null) {
      drawAngledPanel(
        canvas,
        size.toRect(),
        fill: karmaOverlay,
        border: Colors.transparent,
        borderWidth: 0,
        cutTopLeft: true,
        cutTopRight: true,
      );
    }

    super.render(canvas);
  }
}
