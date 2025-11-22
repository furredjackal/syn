import 'package:flame/components.dart';
import 'package:flame/effects.dart';
import 'package:flame/events.dart';

import '../../syn_game.dart';
import '../../ui/ui_signal_bus.dart';

enum DockSide { left, right, bottom }
enum DockState { collapsed, expanded }

abstract class MagneticDockComponent extends PositionComponent
    with HasGameReference<SynGame>, HoverCallbacks, TapCallbacks, UiSignalListener {
  MagneticDockComponent({
    required this.side,
    required this.collapsedExtent,
    required this.expandedExtent,
    this.dockThickness = 320,
    this.collapseOnExitHover = true,
    this.initialState = DockState.collapsed,
    int? priority,
  }) : super(priority: priority ?? 20) {
    _state = initialState;
  }

  final DockSide side;
  final double collapsedExtent;
  final double expandedExtent;
  final double dockThickness;
  final bool collapseOnExitHover;
  final DockState initialState;

  late DockState _state;
  bool _isAnimating = false;

  DockState get state => _state;

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    game.uiSignals.register(this);
    _layoutForState(initialState, instant: true);
  }

  @override
  void onRemove() {
    game.uiSignals.unregister(this);
    super.onRemove();
  }

  @override
  void onUiSignal(UiSignal signal) {
    // Default no-op; override in subclasses.
  }

  void expand() {
    if (_state == DockState.expanded || _isAnimating) return;
    _state = DockState.expanded;
    _animateToState(_state);
  }

  void collapse() {
    if (_state == DockState.collapsed || _isAnimating) return;
    _state = DockState.collapsed;
    _animateToState(_state);
  }

  void toggle() => _state == DockState.expanded ? collapse() : expand();

  void _layoutForState(DockState state, {bool instant = false}) {
    if (instant) {
      _isAnimating = false;
    }

    final viewport = game.size;
    final centerY = viewport.y / 2;

    switch (side) {
      case DockSide.left:
        size = Vector2(expandedExtent, dockThickness);
        final targetX = state == DockState.expanded
            ? 0.0
            : -(expandedExtent - collapsedExtent);
        position = Vector2(targetX, centerY - dockThickness / 2);
        break;
      case DockSide.right:
        size = Vector2(expandedExtent, dockThickness);
        final targetX = state == DockState.expanded
            ? viewport.x - expandedExtent
            : viewport.x - collapsedExtent;
        position = Vector2(targetX, centerY - dockThickness / 2);
        break;
      case DockSide.bottom:
        size = Vector2(viewport.x, expandedExtent);
        final targetY = state == DockState.expanded
            ? viewport.y - expandedExtent
            : viewport.y - collapsedExtent;
        position = Vector2(0.0, targetY);
        break;
    }
  }

  void _animateToState(DockState state) {
    _isAnimating = true;
    final viewport = game.size;
    Vector2 targetPosition;

    switch (side) {
      case DockSide.left:
        final x = state == DockState.expanded
            ? 0.0
            : -(expandedExtent - collapsedExtent);
        targetPosition = Vector2(x, position.y);
        break;
      case DockSide.right:
        final x = state == DockState.expanded
            ? viewport.x - expandedExtent
            : viewport.x - collapsedExtent;
        targetPosition = Vector2(x, position.y);
        break;
      case DockSide.bottom:
        final y = state == DockState.expanded
            ? viewport.y - expandedExtent
            : viewport.y - collapsedExtent;
        targetPosition = Vector2(position.x, y);
        break;
    }

    final controller = CurvedEffectController(
      game.uiTheme.motion.dockSnapDuration,
      game.uiTheme.motion.dockSnapCurve,
    );

    add(
      MoveEffect.to(
        targetPosition,
        controller,
        onComplete: () => _isAnimating = false,
      ),
    );
  }

  @override
  void onTapDown(TapDownEvent event) {
    toggle();
    event.handled = true;
  }

  @override
  void onHoverEnter() => expand();

  @override
  void onHoverExit() {
    if (collapseOnExitHover) collapse();
  }

  @override
  void onGameResize(Vector2 newSize) {
    super.onGameResize(newSize);
    _layoutForState(_state, instant: true);
  }
}
