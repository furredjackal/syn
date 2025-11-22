import 'package:flame/components.dart';
import 'package:flutter/material.dart';

import '../../../theme/theme.dart';
import '../magnetic_dock_component.dart';
import '../paint/angled_panel.dart';
import 'relationship_panel_component.dart';

class RelationshipsDockComponent extends MagneticDockComponent {
  RelationshipsDockComponent({int? priority})
      : super(
          side: DockSide.right,
          collapsedExtent: 40,
          expandedExtent: 320,
          dockThickness: 420,
          initialState: DockState.collapsed,
          priority: priority,
        );

  late final RelationshipPanelComponent _panel;

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    _panel = RelationshipPanelComponent(
      relationships: game.gameState.relationships,
      onOpenNetwork: toggle,
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
    final baseColor = MoodColors.forMood(mood).withValues(alpha: 0.1);
    final borderColor = MoodColors.forMood(mood).withValues(alpha: 0.55);

    drawAngledPanel(
      canvas,
      size.toRect(),
      fill: baseColor,
      border: borderColor,
      borderWidth: 2,
      cutTopLeft: true,
      cutBottomLeft: true,
      cutTopRight: false,
      cutBottomRight: false,
    );

    final karmaOverlay = karma > 60
        ? Colors.white.withValues(alpha: 0.07)
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
        cutBottomLeft: true,
      );
    }

    super.render(canvas);
  }
}
