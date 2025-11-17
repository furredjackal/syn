import 'dart:math' as math;

import 'package:flame/components.dart';
import 'package:flame/events.dart';
import 'package:flutter/material.dart';
import '../models/game_state.dart';
import '../syn_game.dart';
import 'choice_button_component.dart';

/// EventCanvas: Centered focal point with Persona-style slash transitions.
///
/// Floating layout summary:
/// - Angled canvas + cyan border frame
/// - Persona header badge overlapping the top edge
/// - Jagged title banner, description, divider, and staggered choice buttons
/// - Slash accent + slash entrance wipe
class EventCardComponent extends PositionComponent
    with HasGameReference<SynGame> {
  final GameEvent event;
  final Function(int) onChoice;
  late List<ChoiceButtonComponent> choiceButtons;
  double elapsedTime = 0;

  EventCardComponent({
    required this.event,
    required this.onChoice,
    Vector2? position,
    Vector2? size,
  }) : super(position: position, size: size);

  @override
  Future<void> onLoad() async {
    choiceButtons = [];

    // Base layers (bottom → top)
    add(_EventCanvasBackground(size: size));
    add(_PersonaEventBorder(size: size));
    add(_SlashAccent(size: size));

    // Header strip floats above the border slightly for depth
    const headerOverlap = 18.0;
    final headerWidth = math.max(size.x - 32.0, size.x * 0.75);
    final header = _EventHeader(
      lifeStage: event.lifeStage,
      age: event.age,
      size: Vector2(headerWidth, 90),
    )..position = Vector2((size.x - headerWidth) / 2, -headerOverlap);
    add(header);

    add(_SlashTransition(
      size: size,
      duration: 0.4,
      isEntrance: true,
    ));

    // Layout constants
    const double horizontalPadding = 36.0;
    const double spacingAfterHeader = 24.0;
    const double bannerBottomGap = 12.0;
    const double tagRowBottomGap = 16.0;
    const double descriptionBottomGap = 14.0;
    const double dividerBottomGap = 26.0;
    const double impactSummaryBottomGap = 18.0;
    const double choiceGap = 12.0;

    // Track running vertical position for content below the header badge
    double layoutY = header.position.y + header.size.y + spacingAfterHeader;
    layoutY = math.max(layoutY, 72.0);

    final bannerMaxWidth = math.max(size.x - horizontalPadding * 2, 0);
    final titleBanner = _EventTitleBanner(
      title: event.title,
      position: Vector2(horizontalPadding, layoutY),
      maxWidth: bannerMaxWidth,
    );
    await add(titleBanner);

    layoutY += titleBanner.size.y + bannerBottomGap;

    final tags = event.tags;
    if (tags.isNotEmpty) {
      final tagRow = _EventTagChipRow(
        tags: tags,
        maxWidth: bannerMaxWidth,
        position: Vector2(horizontalPadding, layoutY),
      );
      add(tagRow);
      layoutY += tagRow.size.y + tagRowBottomGap;
    }

    // Description text anchors directly under the banner
    final descriptionMaxWidth =
        (size.x * 0.65).clamp(0.0, size.x - horizontalPadding * 2);
    final descriptionPainter = TextPainter(
      text: TextSpan(
        text: event.description,
        style: const TextStyle(
          fontFamily: 'Montserrat',
          fontWeight: FontWeight.w400,
          fontSize: 18,
          height: 1.45,
          color: Color(0xFFEEEEEE),
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: descriptionMaxWidth);

    final description = _TextPainterComponent(painter: descriptionPainter)
      ..position = Vector2(horizontalPadding, layoutY);
    await add(description);

    layoutY += description.size.y + descriptionBottomGap;

    final accentDivider = _AccentDivider(
      width: descriptionMaxWidth * 0.7,
      position: Vector2(horizontalPadding, layoutY),
    );
    add(accentDivider);

    layoutY += accentDivider.size.y + dividerBottomGap;

    final impactStats = event.deltas.keys.toList();
    if (impactStats.isNotEmpty) {
      final impactRow = _ImpactSummaryRow(
        affectedStats: impactStats,
        maxWidth: bannerMaxWidth,
        position: Vector2(horizontalPadding, layoutY),
      );
      add(impactRow);
      layoutY += impactRow.size.y + impactSummaryBottomGap;
    }

    // Choices run full width with even gaps
    double yOffset = layoutY;
    final buttonWidth = math.max(size.x - horizontalPadding * 2, 0);
    for (var i = 0; i < event.choices.length; i++) {
      final choice = event.choices[i];
      final choiceButton = ChoiceButtonComponent(
        choice: choice,
        index: i,
        onPressed: () => onChoice(i),
        position: Vector2(horizontalPadding, yOffset),
        size: Vector2(buttonWidth, 76),
      );

      final buttonWrapper = _TappableButtonWrapper(
        child: choiceButton,
        staggerDelay: 0.25 + (i * 0.12),
        onTap: () => choiceButton.simulateTap(),
      );
      add(buttonWrapper);
      choiceButtons.add(choiceButton);

      yOffset += choiceButton.size.y + choiceGap;
    }
  }

  @override
  void update(double dt) {
    super.update(dt);
    elapsedTime += dt;

    // Scale + fade entrance animation for the card itself
    if (elapsedTime < 0.35) {
      final progress = elapsedTime / 0.35;
      scale.setValues(0.88 + (progress * 0.12), 0.88 + (progress * 0.12));
    } else {
      scale.setValues(1.0, 1.0);
    }
  }
}

/// Wrapper component to handle taps and animate button entrance with stagger
class _TappableButtonWrapper extends PositionComponent
    with HasGameReference<SynGame>, TapCallbacks {
  final ChoiceButtonComponent child;
  final double staggerDelay;
  final VoidCallback onTap;
  double elapsedTime = 0;
  double fadeOpacity = 0;

  _TappableButtonWrapper({
    required this.child,
    required this.staggerDelay,
    required this.onTap,
  }) : super(size: child.size, position: child.position);

  @override
  Future<void> onLoad() async {
    add(child);
  }

  @override
  void update(double dt) {
    super.update(dt);
    elapsedTime += dt;

    if (elapsedTime < staggerDelay) {
      fadeOpacity = 0;
      child.scale.x = 0.8;
      child.scale.y = 0.8;
    } else if (elapsedTime < staggerDelay + 0.2) {
      final fadeProgress = (elapsedTime - staggerDelay) / 0.2;
      fadeOpacity = fadeProgress.clamp(0.0, 1.0);
      child.scale.x = 0.8 + (fadeProgress * 0.2);
      child.scale.y = 0.8 + (fadeProgress * 0.2);
    } else {
      fadeOpacity = 1;
      child.scale.x = 1;
      child.scale.y = 1;
    }

    _updateHoverState();
  }

  @override
  void render(Canvas canvas) {
    canvas.saveLayer(
      Rect.fromLTWH(0, 0, size.x, size.y),
      Paint()..color = Colors.white.withValues(alpha: fadeOpacity),
    );
    super.render(canvas);
    canvas.restore();
  }

  void _updateHoverState() {
    final pointer = game.mousePosition;
    if (pointer == null) {
      child.setHovered(false);
      return;
    }
    final localPointer = absoluteToLocal(pointer);
    child.setHovered(containsLocalPoint(localPointer));
  }

  @override
  void onTapDown(TapDownEvent event) {
    super.onTapDown(event);
    child.setHovered(true);
  }

  @override
  void onTapUp(TapUpEvent event) {
    super.onTapUp(event);
    child.setHovered(false);
    onTap();
  }

  @override
  void onTapCancel(TapCancelEvent event) {
    super.onTapCancel(event);
    child.setHovered(false);
  }
}

class _EventHeader extends PositionComponent {
  final String lifeStage;
  final int age;

  _EventHeader({
    required this.lifeStage,
    required this.age,
    required Vector2 size,
  }) : super(size: size);

  @override
  void render(Canvas canvas) {
    const stageTop = 10.0;
    const stageHeight = 60.0;
    final stageWidth = math.min(size.x * 0.45, 240.0);
    const skew = 24.0;

    final stagePath = Path()
      ..moveTo(skew, stageTop)
      ..lineTo(stageWidth, stageTop)
      ..lineTo(stageWidth - skew, stageTop + stageHeight)
      ..lineTo(0, stageTop + stageHeight)
      ..close();

    final stageRect = stagePath.getBounds();
    final stageFill = Paint()
      ..shader = LinearGradient(
        colors: const [
          Color(0xFF162037),
          Color(0xFF0D1426),
        ],
        begin: Alignment.topLeft,
        end: Alignment.bottomRight,
      ).createShader(stageRect);

    canvas.drawPath(stagePath, stageFill);

    canvas.drawPath(
      stagePath,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2.5
        ..color = const Color(0xFF00D9FF),
    );

    final stageLabelPainter = TextPainter(
      text: const TextSpan(
        text: 'STAGE',
        style: TextStyle(
          fontFamily: 'Montserrat',
          fontWeight: FontWeight.w600,
          fontSize: 13,
          color: Color(0xFF8EF9FF),
          letterSpacing: 2,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: stageWidth - skew);

    final lifeStagePainter = TextPainter(
      text: TextSpan(
        text: lifeStage.toUpperCase(),
        style: const TextStyle(
          fontFamily: 'Montserrat',
          fontWeight: FontWeight.w900,
          fontSize: 26,
          letterSpacing: 1.2,
          color: Colors.white,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: stageWidth - skew - 12);

    stageLabelPainter.paint(canvas, const Offset(18, stageTop + 6));
    lifeStagePainter.paint(canvas, const Offset(18, stageTop + 20));

    // Age hexagon on the right
    const hexRadius = 34.0;
    final hexCenter = Offset(size.x - 70, stageTop + stageHeight / 2 + 4);
    final hexPath = _buildHexagon(hexCenter, hexRadius);
    final hexFill = Paint()
      ..shader = LinearGradient(
        colors: const [Color(0xFF00D9FF), Color(0xFF7B5CFF)],
        begin: Alignment.topCenter,
        end: Alignment.bottomCenter,
      ).createShader(Rect.fromCircle(center: hexCenter, radius: hexRadius));

    canvas.drawPath(hexPath, hexFill);
    canvas.drawPath(
      hexPath,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 2
        ..color = Colors.white,
    );

    final ageLabelPainter = TextPainter(
      text: const TextSpan(
        text: 'AGE',
        style: TextStyle(
          fontFamily: 'Montserrat',
          fontWeight: FontWeight.w600,
          fontSize: 12,
          color: Colors.black87,
          letterSpacing: 1.5,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();

    final ageValuePainter = TextPainter(
      text: TextSpan(
        text: age.toString(),
        style: const TextStyle(
          fontFamily: 'Montserrat',
          fontWeight: FontWeight.w800,
          fontSize: 22,
          color: Colors.black,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();

    ageLabelPainter.paint(
      canvas,
      Offset(hexCenter.dx - ageLabelPainter.width / 2, hexCenter.dy - 22),
    );
    ageValuePainter.paint(
      canvas,
      Offset(hexCenter.dx - ageValuePainter.width / 2, hexCenter.dy - 2),
    );
  }

  Path _buildHexagon(Offset center, double radius) {
    final path = Path();
    for (var i = 0; i < 6; i++) {
      final angle = math.pi / 3 * i - math.pi / 2;
      final x = center.dx + radius * math.cos(angle);
      final y = center.dy + radius * math.sin(angle);
      if (i == 0) {
        path.moveTo(x, y);
      } else {
        path.lineTo(x, y);
      }
    }
    path.close();
    return path;
  }
}

/// Persona-style jagged title banner with gradient fill
class _EventTitleBanner extends PositionComponent {
  final String title;

  _EventTitleBanner({
    required this.title,
    required Vector2 position,
    required double maxWidth,
  }) : super(
          position: position,
          size: Vector2(maxWidth, 82),
        );

  @override
  void render(Canvas canvas) {
    const skew = 24.0;
    const notch = 32.0;
    final w = size.x;
    final h = size.y;

    final path = Path()
      ..moveTo(0, h * 0.35)
      ..lineTo(skew, 0)
      ..lineTo(w - notch, 0)
      ..lineTo(w, h * 0.35)
      ..lineTo(w - skew, h)
      ..lineTo(notch * 0.4, h)
      ..close();

    final rect = Rect.fromLTWH(0, 0, w, h);
    final fill = Paint()
      ..shader = const LinearGradient(
        colors: [Color(0xFF00D9FF), Color(0xFF7B5CFF)],
        begin: Alignment.centerLeft,
        end: Alignment.centerRight,
      ).createShader(rect);

    canvas.drawPath(path, fill);

    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 3
        ..color = Colors.white,
    );

    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.6
        ..color = const Color(0xFF00131C).withOpacity(0.6),
    );

    final titlePainter = TextPainter(
      text: TextSpan(
        text: title.toUpperCase(),
        style: const TextStyle(
          fontFamily: 'Montserrat',
          fontWeight: FontWeight.w900,
          fontSize: 34,
          letterSpacing: 1.8,
          color: Colors.white,
          shadows: [
            Shadow(
              color: Colors.black,
              offset: Offset(3, 3),
              blurRadius: 0,
            ),
          ],
        ),
      ),
      maxLines: 2,
      ellipsis: '…',
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: w - 32);

    final textOffset = Offset(20, h / 2 - titlePainter.height / 2);
    titlePainter.paint(canvas, textOffset);
  }
}

class _EventTagChipRow extends PositionComponent {
  final List<String> tags;
  final double maxWidth;
  static const double _chipHeight = 26.0;
  static const double _horizontalPadding = 12.0;
  static const double _horizontalSpacing = 8.0;
  static const double _verticalSpacing = 6.0;
  static const List<Color> _chipPalette = [
    Color(0xFF145366),
    Color(0xFF2E1A4A),
  ];

  late final List<_TagChipLayout> _chipLayouts;

  _EventTagChipRow({
    required this.tags,
    required this.maxWidth,
    required Vector2 position,
  }) : super(position: position, size: Vector2(maxWidth, 0)) {
    _chipLayouts = _buildLayouts();
    final height =
        _chipLayouts.isEmpty ? 0 : _chipLayouts.map((c) => c.rect.bottom).reduce(math.max);
    size.y = height;
  }

  List<_TagChipLayout> _buildLayouts() {
    final layouts = <_TagChipLayout>[];
    double cursorX = 0;
    double cursorY = 0;
    int paletteIndex = 0;

    for (final rawTag in tags) {
      final label = rawTag.toUpperCase();
      final painter = TextPainter(
        text: TextSpan(
          text: label,
          style: const TextStyle(
            fontFamily: 'Montserrat',
            fontWeight: FontWeight.w700,
            fontSize: 12,
            letterSpacing: 1.4,
            color: Colors.white,
          ),
        ),
        textDirection: TextDirection.ltr,
      )..layout(maxWidth: math.max(maxWidth - _horizontalPadding * 2, 0));

      final chipWidth = math.min(
        painter.width + _horizontalPadding * 2,
        maxWidth,
      );

      if (cursorX + chipWidth > maxWidth && cursorX > 0) {
        cursorX = 0;
        cursorY += _chipHeight + _verticalSpacing;
      }

      final rect = Rect.fromLTWH(cursorX, cursorY, chipWidth, _chipHeight);
      const skew = 10.0;
      final path = Path()
        ..moveTo(rect.left + skew, rect.top)
        ..lineTo(rect.right, rect.top)
        ..lineTo(rect.right - skew, rect.bottom)
        ..lineTo(rect.left, rect.bottom)
        ..close();

      final textOffset = Offset(
        rect.left + _horizontalPadding,
        rect.top + (_chipHeight - painter.height) / 2,
      );

      final fillColor = _chipPalette[paletteIndex % _chipPalette.length];
      paletteIndex++;

      layouts.add(
        _TagChipLayout(
          rect: rect,
          path: path,
          painter: painter,
          textOffset: textOffset,
          fillColor: fillColor,
        ),
      );

      cursorX += chipWidth + _horizontalSpacing;
    }

    return layouts;
  }

  @override
  void render(Canvas canvas) {
    for (final chip in _chipLayouts) {
      canvas.drawPath(
        chip.path,
        Paint()..color = chip.fillColor,
      );
      canvas.drawPath(
        chip.path,
        Paint()
          ..style = PaintingStyle.stroke
          ..strokeWidth = 1
          ..color = Colors.white.withOpacity(0.25),
      );
      chip.painter.paint(canvas, chip.textOffset);
    }
  }
}

class _ImpactSummaryRow extends PositionComponent {
  final List<String> affectedStats;

  _ImpactSummaryRow({
    required this.affectedStats,
    required double maxWidth,
    required Vector2 position,
  }) : super(position: position, size: Vector2(maxWidth, 26));

  @override
  void render(Canvas canvas) {
    if (affectedStats.isEmpty) {
      return;
    }

    final displayStats = affectedStats
        .map((s) => s.isEmpty
            ? s
            : s[0].toUpperCase() +
                (s.length > 1 ? s.substring(1).toLowerCase() : ''))
        .join(', ');
    final summaryPainter = TextPainter(
      text: TextSpan(
        text: 'AFFECTS: $displayStats',
        style: TextStyle(
          fontFamily: 'Montserrat',
          fontWeight: FontWeight.w600,
          fontSize: 14,
          letterSpacing: 0.8,
          color: Colors.white.withOpacity(0.9),
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout(maxWidth: size.x - 24);

    const diamondSize = 4.0;
    final centerY = size.y / 2;
    final diamondCenter = Offset(8, centerY);
    final diamondPath = Path()
      ..moveTo(diamondCenter.dx, diamondCenter.dy - diamondSize)
      ..lineTo(diamondCenter.dx + diamondSize, diamondCenter.dy)
      ..lineTo(diamondCenter.dx, diamondCenter.dy + diamondSize)
      ..lineTo(diamondCenter.dx - diamondSize, diamondCenter.dy)
      ..close();

    canvas.drawPath(
      diamondPath,
      Paint()..color = const Color(0xFF00D9FF),
    );

    summaryPainter.paint(
      canvas,
      Offset(16, centerY - summaryPainter.height / 2),
    );
  }
}

class _TagChipLayout {
  final Rect rect;
  final Path path;
  final TextPainter painter;
  final Offset textOffset;
  final Color fillColor;

  _TagChipLayout({
    required this.rect,
    required this.path,
    required this.painter,
    required this.textOffset,
    required this.fillColor,
  });
}

class _AccentDivider extends PositionComponent {
  _AccentDivider({
    required double width,
    required Vector2 position,
  }) : super(
          size: Vector2(width, 8),
          position: position,
        );

  @override
  void render(Canvas canvas) {
    final path = Path()
      ..moveTo(0, size.y)
      ..lineTo(size.x * 0.7, 0)
      ..lineTo(size.x, 0)
      ..lineTo(size.x * 0.3, size.y)
      ..close();

    canvas.drawPath(
      path,
      Paint()
        ..shader = LinearGradient(
          colors: const [Color(0xFF00D9FF), Color(0xFF7B5CFF)],
          begin: Alignment.centerLeft,
          end: Alignment.centerRight,
        ).createShader(Rect.fromLTWH(0, 0, size.x, size.y)),
    );

    canvas.drawPath(
      path,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.2
        ..color = Colors.white.withOpacity(0.3),
    );
  }
}

class _EventCanvasBackground extends PositionComponent {
  _EventCanvasBackground({required Vector2 size}) : super(size: size);

  @override
  void render(Canvas canvas) {
    final path = _buildCanvasPath();

    canvas.drawPath(
      path,
      Paint()
        ..color = const Color(0xFF000000).withOpacity(0.75)
        ..style = PaintingStyle.fill,
    );

    final gradientPath = Path()..addRect(Rect.fromLTWH(0, 0, size.x, size.y));
    canvas.drawPath(
      gradientPath,
      Paint()
        ..shader = LinearGradient(
          begin: Alignment.topCenter,
          end: Alignment.bottomCenter,
          colors: [
            const Color(0xFF1a1a1a).withOpacity(0.4),
            const Color(0xFF0a0a0a).withOpacity(0.2),
          ],
        ).createShader(Rect.fromLTWH(0, 0, size.x, size.y)),
    );
  }

  Path _buildCanvasPath() {
    const angleOffset = 16.0;
    return Path()
      ..moveTo(angleOffset, 0)
      ..lineTo(size.x, 0)
      ..lineTo(size.x - angleOffset, size.y)
      ..lineTo(0, size.y)
      ..close();
  }
}

class _TextPainterComponent extends PositionComponent {
  _TextPainterComponent({required this.painter})
      : super(size: Vector2(painter.width, painter.height));

  final TextPainter painter;

  @override
  void render(Canvas canvas) {
    painter.paint(canvas, Offset.zero);
  }
}

class _PersonaEventBorder extends PositionComponent {
  _PersonaEventBorder({required Vector2 size}) : super(size: size);

  @override
  void render(Canvas canvas) {
    const angleOffset = 16.0;

    final borderPath = Path()
      ..moveTo(angleOffset, 0)
      ..lineTo(size.x, 0)
      ..lineTo(size.x - angleOffset, size.y)
      ..lineTo(0, size.y)
      ..close();

    canvas.drawPath(
      borderPath,
      Paint()
        ..color = const Color(0xFF00D9FF)
        ..style = PaintingStyle.stroke
        ..strokeWidth = 3,
    );

    const innerOffset = 2.0;
    final innerPath = Path()
      ..moveTo(angleOffset + innerOffset, innerOffset)
      ..lineTo(size.x - innerOffset, innerOffset)
      ..lineTo(size.x - angleOffset - innerOffset, size.y - innerOffset)
      ..lineTo(innerOffset, size.y - innerOffset)
      ..close();

    canvas.drawPath(
      innerPath,
      Paint()
        ..color = const Color(0xFF00D9FF).withValues(alpha: 0.3)
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1,
    );
  }
}

class _SlashAccent extends PositionComponent {
  _SlashAccent({required Vector2 size}) : super(size: size);

  @override
  void render(Canvas canvas) {
    const slashWidth = 80.0;
    const glowRadius = 12.0;

    final slashPath = Path()
      ..moveTo(size.x - slashWidth, -size.y * 0.2)
      ..lineTo(size.x + slashWidth, size.y * 0.8)
      ..lineTo(size.x - slashWidth + 8, size.y * 0.8)
      ..lineTo(size.x + slashWidth - 8, -size.y * 0.2)
      ..close();

    canvas.drawPath(
      slashPath,
      Paint()
        ..color = const Color(0xFF00D9FF).withValues(alpha: 0.15)
        ..style = PaintingStyle.fill
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, glowRadius),
    );

    const coreSlashWidth = 40.0;
    final coreSlashPath = Path()
      ..moveTo(size.x - coreSlashWidth, -size.y * 0.2)
      ..lineTo(size.x + coreSlashWidth, size.y * 0.8)
      ..lineTo(size.x - coreSlashWidth + 4, size.y * 0.8)
      ..lineTo(size.x + coreSlashWidth - 4, -size.y * 0.2)
      ..close();

    canvas.drawPath(
      coreSlashPath,
      Paint()
        ..color = const Color(0xFF00D9FF).withValues(alpha: 0.25)
        ..style = PaintingStyle.fill,
    );
  }
}

class _SlashTransition extends PositionComponent {
  final double duration;
  final bool isEntrance;
  double elapsedTime = 0;

  _SlashTransition({
    required Vector2 size,
    required this.duration,
    required this.isEntrance,
  }) : super(size: size);

  @override
  void update(double dt) {
    super.update(dt);
    elapsedTime += dt;

    if (elapsedTime >= duration) {
      removeFromParent();
    }
  }

  @override
  void render(Canvas canvas) {
    final progress = (elapsedTime / duration).clamp(0.0, 1.0);

    final startX = isEntrance ? size.x : 0;
    const endX = 0;
    final currentX = startX + (endX - startX) * progress;

    const slashWidth = 50.0;
    final path = Path()
      ..moveTo(currentX - slashWidth, -size.y * 0.3)
      ..lineTo(currentX + slashWidth, size.y * 1.3)
      ..lineTo(currentX, size.y * 1.3)
      ..lineTo(currentX - slashWidth * 0.5, -size.y * 0.3)
      ..close();

    canvas.drawPath(
      path,
      Paint()
        ..color = const Color(0xFF00D9FF).withValues(alpha: 0.8)
        ..style = PaintingStyle.fill,
    );

    final edgePath = Path()
      ..moveTo(currentX, -size.y * 0.3)
      ..lineTo(currentX, size.y * 1.3);

    canvas.drawPath(
      edgePath,
      Paint()
        ..color = const Color(0xFFFFFFFF)
        ..strokeWidth = 4
        ..strokeCap = StrokeCap.round
        ..maskFilter = const MaskFilter.blur(BlurStyle.outer, 12),
    );

    canvas.drawPath(
      edgePath,
      Paint()
        ..color = const Color(0xFF00D9FF)
        ..strokeWidth = 2
        ..strokeCap = StrokeCap.round
        ..maskFilter = const MaskFilter.blur(BlurStyle.normal, 6),
    );
  }
}
