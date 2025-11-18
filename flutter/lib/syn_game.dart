import 'dart:async';
import 'dart:ui';

import 'package:flame/camera.dart';
import 'package:flame/components.dart';
import 'package:flame/events.dart';
import 'package:flame/game.dart';
import 'package:flutter/painting.dart'
    show TextDirection, TextPainter, TextSpan, TextStyle;
import 'models/game_state.dart';
import 'game_screen_component.dart';
import 'main_menu_component.dart';
import 'splash_screen_component.dart';
import 'character_creation_component.dart';
import 'ui_effect_layer.dart';
import 'widgets/particle_system_component.dart' as custom;
import 'settings_screen_component.dart';

class SynGame extends FlameGame
    with HasKeyboardHandlerComponents, MouseMovementDetector {
  SynGame() {
    gameState = GameState();
  }

  late final RouterComponent _router;
  final UIEffectLayer _uiEffectLayer = UIEffectLayer();
  final custom.ParticleSystemComponent _particleSystem =
      custom.ParticleSystemComponent();
  late final GameState gameState;
  Vector2? _mousePosition;
  bool _resumeGameAfterSettings = false;

  Vector2? get mousePosition => _mousePosition;

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    camera.viewport = FixedResolutionViewport(resolution: Vector2(1280, 720));

    _uiEffectLayer
      ..size = size
      ..setActive(false);
    _particleSystem
      ..size = size
      ..setActive(false);
    add(_uiEffectLayer);
    add(_particleSystem);

    _router = RouterComponent(
      initialRoute: 'splash',
      routes: {
        'splash': Route(() => SplashScreenComponent()),
        'menu': Route(() => MainMenuComponent()),
        'character_creation': Route(() => CharacterCreationComponent()),
        'gameplay': Route(() => GameScreenComponent()),
        'settings': Route(() => SettingsScreenComponent()),
      },
    );
    add(_router);
  }

  @override
  void onGameResize(Vector2 size) {
    super.onGameResize(size);
    _uiEffectLayer.size = size;
    _particleSystem.size = size;
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

  void showSplash() {
    _router.pushReplacementNamed('splash');
  }

  void showMainMenu() {
    unawaited(_navigateToMenu());
  }

  Future<void> _navigateToMenu() async {
    await _performSceneTransition(() async {
      _setGameSystemsVisible(false);
      _router.pushReplacementNamed('menu');
    });
  }

  void showCharacterCreation() {
    unawaited(_navigateToCharacterCreation());
  }

  Future<void> _navigateToCharacterCreation() async {
    await _performSceneTransition(() async {
      _setGameSystemsVisible(false);
      _router.pushReplacementNamed('character_creation');
    });
  }

  Future<void> startGameplay() async {
    await _runWithLoadingOverlay(() async {
      await _performSceneTransition(() async {
        _router.pushReplacementNamed('gameplay');
        _setGameSystemsVisible(true);
      });
    });
  }

  Future<void> startGameplayWithCharacter({
    required String name,
    required String archetype,
    required bool sfwMode,
    required String difficulty,
  }) async {
    await _runWithLoadingOverlay(() async {
      gameState.setPlayerName(name);
      gameState.setArchetype(archetype);
      gameState.sfwMode = sfwMode;
      gameState.setDifficulty(difficulty);
      await _performSceneTransition(() async {
        _router.pushReplacementNamed('gameplay');
        _setGameSystemsVisible(true);
      });
    });
  }

  void returnToTitle() {
    showMainMenu();
  }

  void showSettings() {
    final current = _router.currentRoute.name;
    _resumeGameAfterSettings = current == 'gameplay';
    if (_resumeGameAfterSettings) {
      _setGameSystemsVisible(false);
    }
    _router.pushNamed('settings');
  }

  void closeSettings() {
    if (_router.currentRoute.name == 'settings') {
      if (_router.canPop()) {
        _router.pop();
      }
      if (_resumeGameAfterSettings) {
        _setGameSystemsVisible(true);
      }
      _resumeGameAfterSettings = false;
    }
  }

  void _setGameSystemsVisible(bool visible) {
    _uiEffectLayer.setActive(visible);
    _particleSystem.setActive(visible);
  }

  void showComingSoon(String label) {
    final banner = _NotificationBanner(label)
      ..position = Vector2(size.x / 2, size.y - 40);
    add(banner);
  }

  void togglePauseOverlay() {
    if (overlays.isActive('PauseMenuOverlay')) {
      overlays.remove('PauseMenuOverlay');
      resumeEngine();
    } else {
      pauseEngine();
      overlays.add('PauseMenuOverlay');
    }
  }

  void showPauseOverlay() {
    togglePauseOverlay();
  }

  custom.ParticleSystemComponent get particleSystem => _particleSystem;
  UIEffectLayer get uiEffectLayer => _uiEffectLayer;
  ConfirmationRequest? get pendingConfirmation => _pendingConfirmation;

  Future<void> _runWithLoadingOverlay(Future<void> Function() action) async {
    overlays.add('LoadingScreenOverlay');
    pauseEngine();
    try {
      await action();
    } finally {
      overlays.remove('LoadingScreenOverlay');
      resumeEngine();
    }
  }

  Future<void> _performSceneTransition(Future<void> Function() change) async {
    final completer = Completer<void>();
    _pendingTransitionCallback = () {
      change().whenComplete(() {
        overlays.remove('TransitionOverlay');
        if (!completer.isCompleted) {
          completer.complete();
        }
      });
    };
    overlays.add('TransitionOverlay');
    return completer.future;
  }

  void onTransitionOverlayComplete() {
    final callback = _pendingTransitionCallback;
    _pendingTransitionCallback = null;
    if (callback != null) {
      callback();
    } else {
      overlays.remove('TransitionOverlay');
    }
  }

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
    overlays.add('ConfirmationDialogOverlay');
  }

  void confirmCurrentDialog() {
    final request = _pendingConfirmation;
    if (request == null) {
      overlays.remove('ConfirmationDialogOverlay');
      return;
    }
    _pendingConfirmation = null;
    overlays.remove('ConfirmationDialogOverlay');
    request.onConfirm();
  }

  void cancelCurrentDialog() {
    final request = _pendingConfirmation;
    _pendingConfirmation = null;
    overlays.remove('ConfirmationDialogOverlay');
    request?.onCancel?.call();
  }

  void promptQuitToMenu() {
    showConfirmationDialog(
      title: 'Quit to Main Menu?',
      message: 'Any unsaved progress will be lost.',
      confirmLabel: 'Quit',
      cancelLabel: 'Stay',
      onConfirm: () {
        overlays.remove('PauseMenuOverlay');
        resumeEngine();
        unawaited(_navigateToMenu());
      },
    );
  }

  VoidCallback? _pendingTransitionCallback;
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
