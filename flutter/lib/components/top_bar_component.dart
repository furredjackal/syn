import 'dart:math' as math;

import 'package:flame/components.dart';
import 'package:flutter/material.dart';
import '../syn_game.dart';
import '../ui/syn_theme.dart';

class TopBarComponent extends PositionComponent with HasGameReference<SynGame> {
  String _lifeStage = '';
  int _age = 0;
  int _mood = 0;
  String? _dateLabel;

  @override
  Future<void> onLoad() async {
    _syncState();
  }

  void _syncState() {
    final state = game.gameState;
    _lifeStage = state.lifeStage.toUpperCase();
    _age = state.age;
    _mood = state.mood;
    _dateLabel = state.year > 0 ? 'YEAR ${state.year}' : null;
  }

  @override
  void update(double dt) {
    super.update(dt);
    _syncState();
  }

  @override
  void render(Canvas canvas) {
    super.render(canvas);
    _drawStrip(canvas);
    _drawStageBlock(canvas);
    _drawDate(canvas);
    _drawMoodChip(canvas);
    _drawAgeBadge(canvas);
  }

  void _drawStrip(Canvas canvas) {
    final path = Path()
      ..moveTo(0, 0)
      ..lineTo(size.x - 40, 0)
      ..lineTo(size.x, size.y)
      ..lineTo(40, size.y)
      ..close();

    final gradient = const LinearGradient(
      colors: [SynColors.bgDark, SynColors.bgPanel],
      begin: Alignment.topLeft,
      end: Alignment.bottomRight,
    );

    canvas.drawPath(
      path,
      Paint()
        ..shader = gradient.createShader(Rect.fromLTWH(0, 0, size.x, size.y)),
    );

    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = SynLayout.borderWidthHeavy
        ..color = SynColors.textPrimary,
    );

    const inset = 4.0;
    final innerPath = Path()
      ..moveTo(inset, inset)
      ..lineTo(size.x - 40 - inset, inset)
      ..lineTo(size.x - inset, size.y - inset)
      ..lineTo(40 + inset, size.y - inset)
      ..close();

    canvas.drawPath(
      innerPath,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = SynLayout.borderWidthLight
        ..color = SynColors.primaryCyan.withOpacity(0.55),
    );
  }

  void _drawStageBlock(Canvas canvas) {
    const padding = 20.0;
    final stagePainter = TextPainter(
      text: TextSpan(
        text: 'STAGE',
        style: SynTextStyles.chip.copyWith(color: SynColors.primaryCyan),
      ),
      textDirection: TextDirection.ltr,
    )..layout();

    final valuePainter = TextPainter(
      text: TextSpan(
        text: _lifeStage,
        style: SynTextStyles.h2Strip.copyWith(fontSize: 24),
      ),
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: size.x * 0.4);

    stagePainter.paint(canvas, const Offset(padding, 8));
    valuePainter.paint(canvas, Offset(padding, 20));
  }

  void _drawDate(Canvas canvas) {
    if (_dateLabel == null) {
      return;
    }
    final painter = TextPainter(
      text: TextSpan(
        text: _dateLabel,
        style: SynTextStyles.body.copyWith(
          fontSize: 14,
          letterSpacing: 1.0,
          color: SynColors.textSubtle,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: size.x * 0.3);

    final x = (size.x - painter.width) / 2;
    painter.paint(canvas, Offset(x, size.y / 2 - painter.height / 2));
  }

  void _drawMoodChip(Canvas canvas) {
    final chipWidth = 120.0;
    final chipHeight = 32.0;
    final right = size.x - 110;
    final top = 12.0;
    final path = Path()
      ..moveTo(right - chipWidth, top + chipHeight)
      ..lineTo(right - chipWidth + 18, top)
      ..lineTo(right, top)
      ..lineTo(right - 18, top + chipHeight)
      ..close();

    final color = moodToColor(_mood);

    canvas.drawPath(
      path,
      Paint()..color = color.withOpacity(0.15),
    );
    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.8
        ..color = color,
    );

    final labelPainter = TextPainter(
      text: TextSpan(
        text: 'MOOD',
        style: SynTextStyles.chip,
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    final valuePainter = TextPainter(
      text: TextSpan(
        text: _mood.toString(),
        style: SynTextStyles.body.copyWith(
          fontSize: 18,
          fontWeight: FontWeight.w700,
          color: color,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();

    final textX = right - chipWidth + 24;
    labelPainter.paint(canvas, Offset(textX, top + 4));
    valuePainter.paint(canvas, Offset(textX, top + 12));
  }

  void _drawAgeBadge(Canvas canvas) {
    final radius = 32.0;
    final center = Offset(size.x - radius - 24, size.y / 2);
    final hexPath = Path();
    for (var i = 0; i < 6; i++) {
      final angle = math.pi / 3 * i - math.pi / 2;
      final x = center.dx + radius * math.cos(angle);
      final y = center.dy + radius * math.sin(angle);
      if (i == 0) {
        hexPath.moveTo(x, y);
      } else {
        hexPath.lineTo(x, y);
      }
    }
    hexPath.close();

    canvas.drawPath(
      hexPath,
      Paint()
        ..shader = const LinearGradient(
          colors: [SynColors.primaryCyan, SynColors.accentViolet],
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
        ).createShader(Rect.fromCircle(center: center, radius: radius)),
    );
    canvas.drawPath(
      hexPath,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = Colors.white,
    );

    final agePainter = TextPainter(
      text: TextSpan(
        text: _age.toString(),
        style: SynTextStyles.body.copyWith(
          fontSize: 20,
          fontWeight: FontWeight.w800,
          color: SynColors.bgDark,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: radius * 1.6);

    agePainter.paint(
      canvas,
      Offset(
        center.dx - agePainter.width / 2,
        center.dy - agePainter.height / 2,
      ),
    );
  }

}
