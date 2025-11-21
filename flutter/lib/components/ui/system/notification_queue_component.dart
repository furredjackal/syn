import 'package:flame/components.dart';
import 'package:flutter/material.dart';

import '../../../syn_game.dart';
import '../syn_theme.dart';

/// Notification queue that stacks angled toasts near the top-right corner.
class NotificationQueueComponent extends PositionComponent
    with HasGameReference<SynGame> {
  NotificationQueueComponent({List<String>? initialMessages})
      : _history = List.of(initialMessages ?? []);

  final List<_ToastEntry> _activeToasts = [];
  final List<String> _history;
  bool _showHistory = false;

  static const double _cardWidth = 260;
  static const double _cardHeight = 42;
  static const double _cardSpacing = 10;
  static const double _shadowBlur = 8;
  static const int _maxActive = 4;

  static const double _toastLifetime = 4.0;
  static const double _fadeIn = 0.3;
  static const double _fadeOut = 0.5;

  void toggleHistory() => _showHistory = !_showHistory;

  @override
  Future<void> onLoad() async {
    await super.onLoad();
    final seedToasts = _history.reversed.take(_maxActive).toList().reversed;
    for (final msg in seedToasts) {
      _activeToasts.add(
        _ToastEntry(text: msg, age: 0, lifetime: _toastLifetime),
      );
    }
  }

  void addMessage(String text) {
    _history.add(text);
    _activeToasts.insert(
      0,
      _ToastEntry(text: text, age: 0, lifetime: _toastLifetime),
    );
    if (_activeToasts.length > _maxActive) {
      _activeToasts.removeLast();
    }
  }

  @override
  void update(double dt) {
    super.update(dt);
    for (final toast in _activeToasts) {
      toast.age += dt;
    }
    _activeToasts.removeWhere((t) => t.age >= t.lifetime);
  }

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    _renderToasts(canvas);
    if (_showHistory) {
      _renderHistoryOverlay(canvas);
    }
  }

  void _renderToasts(Canvas canvas) {
    for (var i = 0; i < _activeToasts.length; i++) {
      final toast = _activeToasts[i];
      final alpha = _toastAlpha(toast);
      final y = i * (_cardHeight + _cardSpacing);
      final path = _toastPath(y.toDouble());

      final bgPaint = Paint()
        ..color = SynColors.bgPanel.withValues(alpha: alpha)
        ..style = PaintingStyle.fill;
      final accentPaint = Paint()
        ..color = SynColors.primaryCyan.withValues(alpha: alpha * 0.7)
        ..style = PaintingStyle.fill;

      canvas.drawShadow(
        Path()..addPath(path, Offset.zero),
        const Color(0xAA000000),
        _shadowBlur,
        false,
      );
      canvas.drawPath(path, bgPaint);

      final accent = Path()
        ..moveTo(0, y.toDouble())
        ..lineTo(18, y.toDouble())
        ..lineTo(4, y + _cardHeight)
        ..lineTo(0, y + _cardHeight)
        ..close();
      canvas.drawPath(accent, accentPaint);

      final textPainter = TextPaint(
        style: TextStyle(
          color: Colors.white.withValues(alpha: alpha),
          fontSize: 12,
          fontWeight: FontWeight.w700,
        ),
      ).toTextPainter(toast.text);
      textPainter.paint(
        canvas,
        Offset(24, y + (_cardHeight - textPainter.height) / 2),
      );
    }
  }

  Path _toastPath(double y) {
    return Path()
      ..moveTo(0, y)
      ..lineTo(_cardWidth - 12, y)
      ..lineTo(_cardWidth, y + _cardHeight * 0.6)
      ..lineTo(_cardWidth - 14, y + _cardHeight)
      ..lineTo(10, y + _cardHeight)
      ..lineTo(0, y + _cardHeight * 0.35)
      ..close();
  }

  double _toastAlpha(_ToastEntry toast) {
    if (toast.age < _fadeIn) {
      return (toast.age / _fadeIn).clamp(0, 1);
    }
    final visibleMid = toast.lifetime - _fadeOut;
    if (toast.age > visibleMid) {
      return (1 - (toast.age - visibleMid) / _fadeOut).clamp(0, 1);
    }
    return 1;
  }

  void _renderHistoryOverlay(Canvas canvas) {
    final viewport = game.size;
    final backdrop = Rect.fromLTWH(0, 0, viewport.x, viewport.y);
    canvas.drawRect(
      backdrop,
      Paint()..color = const Color(0xCC000000),
    );

    const double panelWidth = 360;
    const double panelHeight = 260;
    const double padding = 18;
    final panelRect = Rect.fromLTWH(
      viewport.x - panelWidth - 24,
      24,
      panelWidth,
      panelHeight,
    );

    final panelPath = Path()
      ..moveTo(panelRect.left + 12, panelRect.top)
      ..lineTo(panelRect.right - 18, panelRect.top)
      ..lineTo(panelRect.right, panelRect.top + 48)
      ..lineTo(panelRect.right - 16, panelRect.bottom)
      ..lineTo(panelRect.left + 10, panelRect.bottom)
      ..lineTo(panelRect.left, panelRect.top + 32)
      ..close();

    canvas.drawShadow(panelPath, const Color(0x99000000), 12, false);
    canvas.drawPath(
      panelPath,
      Paint()..color = SynColors.bgPanel.withValues(alpha: 0.95),
    );

    final title = TextPaint(
      style: const TextStyle(
        color: Colors.white,
        fontSize: 14,
        fontWeight: FontWeight.w800,
        letterSpacing: 1.2,
      ),
    ).toTextPainter('NOTIFICATIONS');
    title.paint(
      canvas,
      Offset(
        panelRect.left + padding,
        panelRect.top + padding,
      ),
    );

    final entries = _history.reversed.take(12).toList();
    double y = panelRect.top + padding + title.height + 10;
    for (final entry in entries) {
      final painter = TextPaint(
        style: const TextStyle(
          color: Colors.white,
          fontSize: 12,
          fontWeight: FontWeight.w600,
        ),
      ).toTextPainter(entry);
      if (y + painter.height > panelRect.bottom - padding) break;
      painter.paint(canvas, Offset(panelRect.left + padding, y));
      y += painter.height + 8;
    }
  }
}

class _ToastEntry {
  _ToastEntry({
    required this.text,
    required this.age,
    required this.lifetime,
  });

  final String text;
  double age;
  final double lifetime;
}
