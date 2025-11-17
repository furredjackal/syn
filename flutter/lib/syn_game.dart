import 'dart:ui';

import 'package:flame/camera.dart';
import 'package:flame/components.dart';
import 'package:flame/events.dart';
import 'package:flame/game.dart';
import 'package:flutter/painting.dart'
    show TextAlign, TextDirection, TextPainter, TextSpan, TextStyle;
import 'package:flutter/services.dart';
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
    camera.viewport =
        FixedResolutionViewport(resolution: Vector2(1280, 720));

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
    _setGameSystemsVisible(false);
    _router.pushReplacementNamed('menu');
  }

  void showCharacterCreation() {
    _setGameSystemsVisible(false);
    _router.pushReplacementNamed('character_creation');
  }

  void startGameplay() {
    _router.pushReplacementNamed('gameplay');
    _setGameSystemsVisible(true);
  }

  void returnToTitle() {
    _setGameSystemsVisible(false);
    _router.pushReplacementNamed('menu');
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

  void showPauseOverlay() {
    pauseEngine();
    add(_PauseOverlay(onResume: () {
      resumeEngine();
    }));
  }

  custom.ParticleSystemComponent get particleSystem => _particleSystem;
  UIEffectLayer get uiEffectLayer => _uiEffectLayer;
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

class _PauseOverlay extends PositionComponent
    with TapCallbacks, KeyboardHandler, HasGameReference<SynGame> {
  _PauseOverlay({required this.onResume});

  final VoidCallback onResume;

  @override
  Future<void> onLoad() async {
    size = game.size;
  }

  @override
  void render(Canvas canvas) {
    final rect = Rect.fromLTWH(0, 0, size.x, size.y);
    canvas.drawRect(rect, Paint()..color = const Color(0xAA000000));
    final textPainter = TextPainter(
      textAlign: TextAlign.center,
      textDirection: TextDirection.ltr,
      text: const TextSpan(
        text: 'PAUSED\nTap anywhere or press Enter to resume',
        style: TextStyle(
          color: Color(0xFFFFFFFF),
          fontSize: 28,
          fontWeight: FontWeight.w700,
        ),
      ),
    )..layout(maxWidth: size.x);
    textPainter.paint(
      canvas,
      Offset(
        (size.x - textPainter.width) / 2,
        (size.y - textPainter.height) / 2,
      ),
    );
  }

  void _close() {
    removeFromParent();
    onResume();
  }

  @override
  void onTapUp(TapUpEvent event) {
    _close();
  }

  @override
  bool onKeyEvent(KeyEvent event, Set<LogicalKeyboardKey> keysPressed) {
    if (event is KeyDownEvent &&
        (event.logicalKey == LogicalKeyboardKey.enter ||
            event.logicalKey == LogicalKeyboardKey.space)) {
      _close();
      return true;
    }
    return false;
  }

  @override
  void onGameResize(Vector2 size) {
    super.onGameResize(size);
    this.size = size;
  }
}
