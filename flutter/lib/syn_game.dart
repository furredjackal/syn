import 'dart:async';
import 'dart:ui';

import 'package:flame/camera.dart';
import 'package:flame/components.dart';
import 'package:flame/events.dart';
import 'package:flame/game.dart';
import 'package:flutter/painting.dart'
    show TextDirection, TextPainter, TextSpan, TextStyle;
import 'theme/theme.dart';
import 'ui/ui_signal_bus.dart';
import 'models/game_state.dart';
import 'components/ui/system/background_layer_component.dart';
import 'components/ui/effects/particle_system_component.dart'
    as effects;

/// SynGame - Flame layer for visual effects only
///
/// Hybrid Architecture:
/// - This handles ONLY the Flame background visuals (particles, background layers)
/// - All UI is handled by Flutter widgets in lib/screens/game_screen.dart
/// - No more RouterComponent - navigation is handled by GamePhase in GameScreen
class SynGame extends FlameGame
    with HasKeyboardHandlerComponents, MouseMovementDetector {
  SynGame({GameState? initialGameState})
      : gameState = initialGameState ?? GameState();

  final GameState gameState;
  final SynUiTheme uiTheme = SynUiTheme.defaultTheme;
  final UiSignalBus uiSignals = UiSignalBus();
  Vector2? _mousePosition;

  Vector2? get mousePosition => _mousePosition;

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    camera.viewport = FixedResolutionViewport(resolution: Vector2(1280, 720));

    // Add cyberpunk visual background (Tier 1 visuals)
    final background = BackgroundLayerComponent()
      ..priority = -10; // Behind everything
    add(background);

    // Add floating data particles
    final particles = effects.ParticleSystemComponent()
      ..priority = -5; // Above background, below UI
    add(particles);

    // No more RouterComponent - UI is handled by Flutter GameScreen
  }

  @override
  void onGameResize(Vector2 size) {
    super.onGameResize(size);
  }

  @override
  void onMouseMove(PointerHoverInfo info) {
    super.onMouseMove(info);
    final pointer = info.eventPosition.widget;
    if (pointer.x < 0 ||
        pointer.y < 0 ||
        pointer.x > size.x ||
        pointer.y > size.y) {
      _mousePosition = null;
    } else {
      _mousePosition = pointer;
    }
  }

  // Navigation methods are now deprecated - UI is handled by Flutter GameScreen
  // These are kept as stubs for compatibility with old code
  
  @deprecated
  void showSplash() {
    // No-op: Navigation handled by GameScreen
  }

  @deprecated
  void showMainMenu() {
    // No-op: Navigation handled by GameScreen
  }

  @deprecated
  void showCharacterCreation() {
    // No-op: Navigation handled by GameScreen
  }

  @deprecated
  Future<void> startGameplay() async {
    // No-op: Gameplay is now handled by Flutter GameScreen
  }

  @deprecated
  Future<void> startGameplayWithCharacter({
    required String name,
    required String archetype,
    required bool sfwMode,
    required String difficulty,
  }) async {
    // Save character data to game state (still needed for backend)
    gameState.setPlayerName(name);
    gameState.setArchetype(archetype);
    gameState.sfwMode = sfwMode;
    gameState.setDifficulty(difficulty);
  }

  @deprecated
  void returnToTitle() {
    // No-op: Navigation handled by GameScreen
  }

  @deprecated
  void showSettings() {
    // No-op: Navigation handled by GameScreen
  }

  @deprecated
  Future<void> showMemoryJournal() async {
    // No-op: Navigation handled by GameScreen
  }

  @deprecated
  Future<void> showDetailedStats() async {
    // No-op: Navigation handled by GameScreen
  }

  @deprecated
  Future<void> showRelationshipNetwork() async {
    // No-op: Navigation handled by GameScreen
  }

  @deprecated
  Future<void> showWorldMap() async {
    // No-op: Navigation handled by GameScreen
  }

  @deprecated
  Future<void> showPossessions() async {
    // No-op: Navigation handled by GameScreen
  }

  @deprecated
  Future<void> showSaveLoad() async {
    // No-op: Navigation handled by GameScreen
  }

  @deprecated
  void closeSettings() {
    // No-op: Navigation handled by GameScreen
  }

  void showComingSoon(String label) {
    final banner = _NotificationBanner(label)
      ..position = Vector2(size.x / 2, size.y - 40);
    add(banner);
  }

  void togglePauseOverlay() {
    if (overlays.isActive('pause_menu')) {
      overlays.remove('pause_menu');
      resumeEngine();
    } else {
      pauseEngine();
      overlays.add('pause_menu');
    }
  }

  void showPauseOverlay() {
    togglePauseOverlay();
  }

  ConfirmationRequest? get pendingConfirmation => _pendingConfirmation;

  // Placeholder handlers for Flutter overlays.
  void handleTextInput(String value) {}
  void executeDebugCommand(String command) {}

  void showConfirmationDialog({
    required String title,
    required String message,
    String confirmLabel = 'Confirm',
    String cancelLabel = 'Cancel',
    required VoidCallback onConfirm,
    VoidCallback? onCancel,
  }) {
    _pendingConfirmation = ConfirmationRequest(
      title: title,
      message: message,
      confirmLabel: confirmLabel,
      cancelLabel: cancelLabel,
      onConfirm: onConfirm,
      onCancel: onCancel,
    );
    overlays.add('confirm_dialog');
  }

  void confirmCurrentDialog() {
    final request = _pendingConfirmation;
    if (request == null) {
      overlays.remove('confirm_dialog');
      return;
    }
    _pendingConfirmation = null;
    overlays.remove('confirm_dialog');
    request.onConfirm();
  }

  void cancelCurrentDialog() {
    final request = _pendingConfirmation;
    _pendingConfirmation = null;
    overlays.remove('confirm_dialog');
    request?.onCancel?.call();
  }

  void promptQuitToMenu() {
    showConfirmationDialog(
      title: 'Quit to Main Menu?',
      message: 'Any unsaved progress will be lost.',
      confirmLabel: 'Quit',
      cancelLabel: 'Stay',
      onConfirm: () {
        overlays.remove('pause_menu');
        resumeEngine();
        // No-op: Navigation handled by Flutter GameScreen
      },
    );
  }

  ConfirmationRequest? _pendingConfirmation;
}

class _NotificationBanner extends PositionComponent {
  _NotificationBanner(this.message) : super(anchor: Anchor.center);

  final String message;
  double _elapsed = 0;

  @override
  Future<void> onLoad() async {
    size = Vector2(500, 40);
  }

  @override
  void update(double dt) {
    super.update(dt);
    _elapsed += dt;
    if (_elapsed >= 2.4) {
      removeFromParent();
    }
  }

  @override
  void render(Canvas canvas) {
    final rect = Rect.fromLTWH(-size.x / 2, -size.y / 2, size.x, size.y);
    canvas.drawRRect(
      RRect.fromRectAndRadius(rect, const Radius.circular(6)),
      Paint()..color = const Color(0xCC000000),
    );
    final textPainter = TextPainter(
      text: TextSpan(
        text: message,
        style: const TextStyle(
          color: Color(0xFFFFFFFF),
          fontSize: 16,
          fontWeight: FontWeight.w600,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: size.x - 16);
    textPainter.paint(
      canvas,
      Offset(-textPainter.width / 2, -textPainter.height / 2),
    );
  }
}

class ConfirmationRequest {
  ConfirmationRequest({
    required this.title,
    required this.message,
    required this.confirmLabel,
    required this.cancelLabel,
    required this.onConfirm,
    this.onCancel,
  });

  final String title;
  final String message;
  final String confirmLabel;
  final String cancelLabel;
  final VoidCallback onConfirm;
  final VoidCallback? onCancel;
}
