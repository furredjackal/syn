import 'package:flame/components.dart';
import 'package:flame/effects.dart';
import 'package:flame/events.dart';
import 'package:flutter/material.dart';

import '../../models/game_state.dart';
import '../../syn_game.dart';
import '../ui/cards/event_card_component.dart';
import '../ui/panels/quick_panel_component.dart';
import '../ui/panels/relationship_panel_component.dart';
import '../ui/panels/stat_panel_component.dart';
import '../ui/panels/top_bar_component.dart';
import '../ui/system/background_layer_component.dart';

/// Magnetic Dock System: Edge panels that expand on hover/click
/// Event card dominates center, stats/relationships magnetically dock to edges
class MagneticDockHubComponent extends PositionComponent
    with HasGameReference<SynGame>, HoverCallbacks {
  
  // -- Child Components --
  late final BackgroundLayerComponent _background;
  late final TopBarComponent _topBar;
  late final MagneticDock _leftDock;   // Stats
  late final MagneticDock _rightDock;  // Relationships
  late final QuickMenuBarComponent _quickMenu;
  late final EventCardComponent _eventCard;

  // -- Layout Constants --
  static const double _dockCollapsedWidth = 48.0;
  static const double _dockExpandedWidth = 420.0;
  static const double _dockAnimDuration = 0.35;
  static const Curve _dockCurve = Curves.easeOutCubic;
  
  static const double _eventCardWidth = 650.0;
  static const double _eventCardMinWidth = 500.0;

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    size = game.size.clone();

    // 1. Background
    _background = BackgroundLayerComponent()..priority = 0;

    // 2. Top Bar
    _topBar = TopBarComponent(
      gameState: game.gameState,
      onStats: () => _leftDock.toggle(),
      onSettings: game.showSettings,
      onSave: game.showSaveLoad,
      onPause: game.togglePauseOverlay,
      onNotifications: () {},
    )..priority = 20;

    // 3. Left Dock (Stats)
    _leftDock = MagneticDock(
      side: DockSide.left,
      collapsedWidth: _dockCollapsedWidth,
      expandedWidth: _dockExpandedWidth,
      child: StatPanelComponent(
        gameState: game.gameState,
        onClose: () => _leftDock.collapse(),
      ),
      onExpansionChanged: (expanded) => _reLayoutEventCard(),
    )..priority = 15;

    // 4. Right Dock (Relationships)
    _rightDock = MagneticDock(
      side: DockSide.right,
      collapsedWidth: _dockCollapsedWidth,
      expandedWidth: _dockExpandedWidth,
      child: RelationshipPanelComponent(
        relationships: game.gameState.relationships,
        onOpenNetwork: () => _rightDock.toggle(),
      ),
      onExpansionChanged: (expanded) => _reLayoutEventCard(),
    )..priority = 15;

    // 5. Event Card (center stage, dynamically sized)
    _eventCard = EventCardComponent(
      event: game.gameState.currentEvent ?? _placeholderEvent(),
      onChoice: (i) => {},
    )..priority = 10;

    // 6. Quick Menu (bottom center)
    _quickMenu = QuickMenuBarComponent(
      onMemory: game.showMemoryJournal,
      onMap: game.showWorldMap,
      onPossessions: game.showPossessions,
    )..priority = 15;

    addAll([
      _background,
      _topBar,
      _leftDock,
      _rightDock,
      _eventCard,
      _quickMenu,
    ]);

    _applyLayout(animated: false);
  }

  @override
  void onGameResize(Vector2 newSize) {
    super.onGameResize(newSize);
    size = newSize;
    _background.size = newSize;
    _topBar.size = Vector2(newSize.x, 92.0);
    _applyLayout(animated: false);
  }

  void _applyLayout({required bool animated}) {
    final w = size.x;
    final h = size.y;
    final topBarHeight = 92.0;

    // 1. Position docks at edges
    final leftDockX = 0.0;
    final rightDockX = w - _rightDock.currentWidth;
    
    _morphPosition(_leftDock, 
      Vector2(leftDockX, topBarHeight), 
      animated: animated);
    _morphPosition(_rightDock, 
      Vector2(rightDockX, topBarHeight), 
      animated: animated);
    
    _leftDock.size = Vector2(_leftDock.currentWidth, h - topBarHeight);
    _rightDock.size = Vector2(_rightDock.currentWidth, h - topBarHeight);

    // 2. Event card in remaining center space
    _reLayoutEventCard(animated: animated);

    // 3. Quick menu at bottom center
    final availableWidth = w - _leftDock.currentWidth - _rightDock.currentWidth;
    _quickMenu.size = Vector2(availableWidth.clamp(400, 800).toDouble(), 80);
    _morphPosition(_quickMenu,
      Vector2(
        _leftDock.currentWidth + (availableWidth - _quickMenu.width) / 2,
        h - 100,
      ),
      animated: animated);
  }

  void _reLayoutEventCard({bool animated = true}) {
    final w = size.x;
    final h = size.y;
    final topBarHeight = 92.0;

    final availableWidth = w - _leftDock.currentWidth - _rightDock.currentWidth;
    final cardWidth = availableWidth.clamp(_eventCardMinWidth, _eventCardWidth).toDouble();
    final cardX = _leftDock.currentWidth + (availableWidth - cardWidth) / 2;

    _eventCard.size = Vector2(cardWidth, h * 0.55);
    _morphPosition(_eventCard,
      Vector2(cardX, topBarHeight + (h - topBarHeight) * 0.15),
      animated: animated);
  }

  void _morphPosition(PositionComponent c, Vector2 pos, {required bool animated}) {
    c.removeWhere((component) => component is MoveEffect);
    
    if (!animated) {
      c.position = pos;
      return;
    }

    c.add(MoveEffect.to(
      pos,
      EffectController(duration: _dockAnimDuration, curve: _dockCurve),
    ));
  }

  GameEvent _placeholderEvent() => GameEvent(
    id: 'xx',
    title: 'LOADING',
    description: '...',
    choices: [],
    lifeStage: 'Child',
    age: 6,
  );
}

// ============================================================================
// MAGNETIC DOCK - Self-contained collapsible panel
// ============================================================================

enum DockSide { left, right }

class MagneticDock extends PositionComponent with HoverCallbacks {
  final DockSide side;
  final double collapsedWidth;
  final double expandedWidth;
  final PositionComponent child;
  final void Function(bool expanded)? onExpansionChanged;

  bool _isExpanded = false;
  double _currentWidth;
  bool _isHovering = false;

  MagneticDock({
    required this.side,
    required this.collapsedWidth,
    required this.expandedWidth,
    required this.child,
    this.onExpansionChanged,
  }) : _currentWidth = collapsedWidth;

  double get currentWidth => _currentWidth;
  bool get isExpanded => _isExpanded;

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    add(child);
    child.size = Vector2(collapsedWidth, size.y);
  }

  @override
  void onHoverEnter() {
    _isHovering = true;
    if (!_isExpanded) {
      _showPreview();
    }
  }

  @override
  void onHoverExit() {
    _isHovering = false;
    if (!_isExpanded) {
      _hidePreview();
    }
  }

  void toggle() {
    if (_isExpanded) {
      collapse();
    } else {
      expand();
    }
  }

  void expand() {
    if (_isExpanded) return;
    _isExpanded = true;
    _animateWidth(expandedWidth);
    onExpansionChanged?.call(true);
  }

  void collapse() {
    if (!_isExpanded) return;
    _isExpanded = false;
    _animateWidth(collapsedWidth);
    onExpansionChanged?.call(false);
  }

  void _showPreview() {
    // Subtle peek on hover (20% expansion)
    _animateWidth(collapsedWidth + (expandedWidth - collapsedWidth) * 0.2, 
      duration: 0.2);
  }

  void _hidePreview() {
    if (!_isExpanded) {
      _animateWidth(collapsedWidth, duration: 0.2);
    }
  }

  void _animateWidth(double targetWidth, {double duration = 0.35}) {
    final startWidth = _currentWidth;
    final delta = targetWidth - startWidth;

    removeWhere((component) => component is _WidthTween);
    
    add(_WidthTween(
      duration: duration,
      onUpdate: (progress) {
        _currentWidth = startWidth + (delta * progress);
        child.size = Vector2(_currentWidth, size.y);
      },
    ));
  }
}

// Simple tween helper for width animation
class _WidthTween extends Component {
  final double duration;
  final void Function(double progress) onUpdate;
  double _elapsed = 0;

  _WidthTween({required this.duration, required this.onUpdate});

  @override
  void update(double dt) {
    _elapsed += dt;
    final progress = (_elapsed / duration).clamp(0.0, 1.0);
    
    // Ease out cubic
    final eased = 1 - pow(1 - progress, 3);
    onUpdate(eased);

    if (progress >= 1.0) {
      removeFromParent();
    }
  }
}

double pow(double x, int exp) {
  double result = 1;
  for (int i = 0; i < exp; i++) {
    result *= x;
  }
  return result;
}