import 'dart:math' show cos, sin;
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_animate/flutter_animate.dart';
import '../widgets/persona_container.dart';

/// Relationship Network Overlay - visualizes character connections
///
/// Props:
/// - onClose: Callback to close the overlay
/// - relationships: List of character relationships to display
class RelationshipNetworkOverlay extends StatefulWidget {
  final VoidCallback onClose;
  final List<CharacterRelationship> relationships;

  const RelationshipNetworkOverlay({
    super.key,
    required this.onClose,
    this.relationships = const [],
  });

  @override
  State<RelationshipNetworkOverlay> createState() =>
      _RelationshipNetworkOverlayState();
}

class _RelationshipNetworkOverlayState
    extends State<RelationshipNetworkOverlay> {
  int? _hoveredNodeIndex;
  final List<Offset> _nodePositions = [];

  @override
  void initState() {
    super.initState();
    _generateNodePositions();
  }

  void _generateNodePositions() {
    // Generate positions in a circular layout
    final count = widget.relationships.isEmpty ? 5 : widget.relationships.length;
    final radius = 200.0;
    final centerX = 0.5;
    final centerY = 0.5;

    for (var i = 0; i < count; i++) {
      final angle = (i / count) * 2 * 3.14159;
      final x = centerX + (radius / 600) * cos(angle);
      final y = centerY + (radius / 600) * sin(angle);
      _nodePositions.add(Offset(x, y));
    }
  }

  @override
  Widget build(BuildContext context) {
    final screenWidth = MediaQuery.of(context).size.width;
    final screenHeight = MediaQuery.of(context).size.height;

    return KeyboardListener(
      focusNode: FocusNode()..requestFocus(),
      onKeyEvent: _handleKeyEvent,
      child: Container(
        color: Colors.black.withValues(alpha: 0.92),
        child: Stack(
          children: [
            // Title
            Positioned(
              left: 60,
              top: 40,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    'RELATIONSHIP NETWORK',
                    style: TextStyle(
                      color: Colors.white,
                      fontSize: 48,
                      fontWeight: FontWeight.w900,
                      letterSpacing: 4,
                      shadows: [
                        Shadow(
                          color: Colors.purpleAccent.withValues(alpha: 0.5),
                          blurRadius: 15,
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(height: 8),
                  Text(
                    'Your social connections visualized',
                    style: TextStyle(
                      color: Colors.white.withValues(alpha: 0.7),
                      fontSize: 16,
                      letterSpacing: 1.2,
                    ),
                  ),
                ],
              )
                  .animate()
                  .fadeIn(duration: 400.ms)
                  .slideY(begin: -0.2, duration: 400.ms, curve: Curves.easeOut),
            ),

            // Network Graph Canvas
            Center(
              child: Container(
                width: screenWidth * 0.7,
                height: screenHeight * 0.7,
                child: CustomPaint(
                  painter: _NetworkPainter(
                    relationships: widget.relationships,
                    nodePositions: _nodePositions,
                    hoveredIndex: _hoveredNodeIndex,
                  ),
                  child: Stack(
                    children: _buildNodes(screenWidth * 0.7, screenHeight * 0.7),
                  ),
                ),
              )
                  .animate()
                  .fadeIn(delay: 200.ms, duration: 600.ms)
                  .scale(
                      begin: const Offset(0.95, 0.95),
                      duration: 600.ms,
                      curve: Curves.easeOutExpo),
            ),

            // Legend
            Positioned(
              left: 60,
              bottom: 60,
              child: _buildLegend()
                  .animate()
                  .fadeIn(delay: 400.ms, duration: 400.ms),
            ),

            // Close Button
            Positioned(
              right: 60,
              top: 40,
              child: _buildCloseButton()
                  .animate()
                  .fadeIn(delay: 300.ms, duration: 400.ms),
            ),
          ],
        ),
      ),
    );
  }

  List<Widget> _buildNodes(double width, double height) {
    final relationships = widget.relationships.isEmpty
        ? _getPlaceholderRelationships()
        : widget.relationships;

    return List.generate(relationships.length, (index) {
      final rel = relationships[index];
      final pos = _nodePositions[index];

      return Positioned(
        left: pos.dx * width - 50,
        top: pos.dy * height - 50,
        child: MouseRegion(
          onEnter: (_) => setState(() => _hoveredNodeIndex = index),
          onExit: (_) => setState(() => _hoveredNodeIndex = null),
          child: _buildNode(rel, index == _hoveredNodeIndex),
        ),
      );
    });
  }

  Widget _buildNode(CharacterRelationship rel, bool isHovered) {
    final color = _getRelationshipColor(rel.strength);

    return PersonaContainer(
      color: isHovered ? color : Colors.black,
      borderColor: color,
      borderWidth: 3,
      skew: -0.15,
      child: Container(
        width: 100,
        height: 100,
        alignment: Alignment.center,
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            Text(
              rel.name,
              style: TextStyle(
                color: isHovered ? Colors.black : Colors.white,
                fontSize: 14,
                fontWeight: FontWeight.w700,
                letterSpacing: 0.5,
              ),
              textAlign: TextAlign.center,
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
            ),
            const SizedBox(height: 4),
            Text(
              rel.role,
              style: TextStyle(
                color: isHovered
                    ? Colors.black.withValues(alpha: 0.7)
                    : Colors.white.withValues(alpha: 0.6),
                fontSize: 11,
                letterSpacing: 0.3,
              ),
              textAlign: TextAlign.center,
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildLegend() {
    return PersonaContainer(
      color: Colors.black.withValues(alpha: 0.8),
      borderColor: Colors.white.withValues(alpha: 0.3),
      skew: -0.15,
      child: Container(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          mainAxisSize: MainAxisSize.min,
          children: [
            Text(
              'RELATIONSHIP STRENGTH',
              style: TextStyle(
                color: Colors.white,
                fontSize: 12,
                fontWeight: FontWeight.w700,
                letterSpacing: 1.2,
              ),
            ),
            const SizedBox(height: 12),
            _buildLegendItem('High (8-10)', Colors.greenAccent),
            _buildLegendItem('Medium (5-7)', Colors.cyanAccent),
            _buildLegendItem('Low (0-4)', Colors.redAccent),
          ],
        ),
      ),
    );
  }

  Widget _buildLegendItem(String label, Color color) {
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Container(
            width: 20,
            height: 20,
            decoration: BoxDecoration(
              color: color,
              border: Border.all(color: Colors.white, width: 1),
            ),
          ),
          const SizedBox(width: 12),
          Text(
            label,
            style: TextStyle(
              color: Colors.white.withValues(alpha: 0.8),
              fontSize: 14,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildCloseButton() {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        onTap: widget.onClose,
        child: PersonaContainer(
          color: Colors.black,
          borderColor: Colors.redAccent.withValues(alpha: 0.5),
          skew: -0.15,
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 12),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(
                  'ESC',
                  style: TextStyle(
                    color: Colors.white.withValues(alpha: 0.7),
                    fontSize: 14,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const SizedBox(width: 8),
                const Text(
                  'CLOSE',
                  style: TextStyle(
                    color: Colors.white,
                    fontSize: 16,
                    fontWeight: FontWeight.w700,
                    letterSpacing: 1.2,
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  void _handleKeyEvent(KeyEvent event) {
    if (event is! KeyDownEvent) return;
    if (event.logicalKey == LogicalKeyboardKey.escape) {
      widget.onClose();
    }
  }

  Color _getRelationshipColor(double strength) {
    if (strength >= 8.0) return Colors.greenAccent;
    if (strength >= 5.0) return Colors.cyanAccent;
    return Colors.redAccent;
  }

  List<CharacterRelationship> _getPlaceholderRelationships() {
    return [
      CharacterRelationship(name: 'Alex', role: 'Friend', strength: 8.5),
      CharacterRelationship(name: 'Morgan', role: 'Rival', strength: 3.2),
      CharacterRelationship(name: 'Sam', role: 'Mentor', strength: 9.1),
      CharacterRelationship(name: 'Taylor', role: 'Coworker', strength: 6.0),
      CharacterRelationship(name: 'Jordan', role: 'Enemy', strength: 1.5),
    ];
  }
}

/// Custom painter for drawing connection lines
class _NetworkPainter extends CustomPainter {
  final List<CharacterRelationship> relationships;
  final List<Offset> nodePositions;
  final int? hoveredIndex;

  _NetworkPainter({
    required this.relationships,
    required this.nodePositions,
    this.hoveredIndex,
  });

  @override
  void paint(Canvas canvas, Size size) {
    if (nodePositions.isEmpty) return;

    final paint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = 2.0;

    // Draw connections from player (center) to all others
    final centerX = size.width / 2;
    final centerY = size.height / 2;

    for (var i = 0; i < nodePositions.length; i++) {
      final pos = nodePositions[i];
      final targetX = pos.dx * size.width;
      final targetY = pos.dy * size.height;

      final strength = relationships.isEmpty ? 5.0 : relationships[i].strength;
      paint.color = _getLineColor(strength, i == hoveredIndex);

      canvas.drawLine(
        Offset(centerX, centerY),
        Offset(targetX, targetY),
        paint,
      );
    }
  }

  Color _getLineColor(double strength, bool isHovered) {
    final baseColor = strength >= 8.0
        ? Colors.greenAccent
        : strength >= 5.0
            ? Colors.cyanAccent
            : Colors.redAccent;

    return isHovered
        ? baseColor
        : baseColor.withValues(alpha: 0.4);
  }

  @override
  bool shouldRepaint(covariant _NetworkPainter oldDelegate) {
    return hoveredIndex != oldDelegate.hoveredIndex;
  }
}

/// Data model for character relationships
class CharacterRelationship {
  final String name;
  final String role;
  final double strength; // 0-10

  const CharacterRelationship({
    required this.name,
    required this.role,
    required this.strength,
  });
}
