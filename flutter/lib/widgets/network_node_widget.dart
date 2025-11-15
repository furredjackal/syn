import 'package:flutter/material.dart';

class NetworkNodeWidget extends StatefulWidget {
  final String id;
  final String label;
  final Offset initialPosition;
  final VoidCallback onDragStart;
  final Function(Offset) onDragUpdate;
  final VoidCallback onDragEnd;
  final VoidCallback onTap;
  final Color color;
  final double size;
  final String? avatarUrl;
  final String? stat;

  const NetworkNodeWidget({
    Key? key,
    required this.id,
    required this.label,
    required this.initialPosition,
    required this.onDragStart,
    required this.onDragUpdate,
    required this.onDragEnd,
    required this.onTap,
    this.color = Colors.cyan,
    this.size = 60,
    this.avatarUrl,
    this.stat,
  }) : super(key: key);

  @override
  State<NetworkNodeWidget> createState() => _NetworkNodeWidgetState();
}

class _NetworkNodeWidgetState extends State<NetworkNodeWidget> {
  late Offset _position;
  bool _isHovering = false;
  bool _isDragging = false;

  @override
  void initState() {
    super.initState();
    _position = widget.initialPosition;
  }

  @override
  Widget build(BuildContext context) {
    return Positioned(
      left: _position.dx,
      top: _position.dy,
      child: GestureDetector(
        onPanStart: (details) {
          setState(() => _isDragging = true);
          widget.onDragStart();
        },
        onPanUpdate: (details) {
          setState(() {
            _position = Offset(
              _position.dx + details.delta.dx,
              _position.dy + details.delta.dy,
            );
          });
          widget.onDragUpdate(_position);
        },
        onPanEnd: (details) {
          setState(() => _isDragging = false);
          widget.onDragEnd();
        },
        onTap: widget.onTap,
        child: MouseRegion(
          onEnter: (_) => setState(() => _isHovering = true),
          onExit: (_) => setState(() => _isHovering = false),
          cursor: SystemMouseCursors.move,
          child: Container(
            width: widget.size,
            height: widget.size,
            decoration: BoxDecoration(
              shape: BoxShape.circle,
              color: widget.color.withOpacity(0.2),
              border: Border.all(
                color: _isHovering || _isDragging
                    ? widget.color
                    : widget.color.withOpacity(0.5),
                width: _isHovering || _isDragging ? 3 : 2,
              ),
              boxShadow: _isDragging
                  ? [
                      BoxShadow(
                        color: widget.color.withOpacity(0.5),
                        blurRadius: 16,
                        spreadRadius: 4,
                      ),
                    ]
                  : _isHovering
                      ? [
                          BoxShadow(
                            color: widget.color.withOpacity(0.3),
                            blurRadius: 8,
                            spreadRadius: 2,
                          ),
                        ]
                      : [],
            ),
            child: Stack(
              children: [
                // Center circle
                Center(
                  child: Container(
                    width: widget.size * 0.6,
                    height: widget.size * 0.6,
                    decoration: BoxDecoration(
                      shape: BoxShape.circle,
                      color: Colors.black.withOpacity(0.7),
                      border: Border.all(
                        color: widget.color.withOpacity(0.4),
                        width: 1,
                      ),
                    ),
                    child: widget.avatarUrl != null
                        ? ClipOval(
                            child: Image.network(
                              widget.avatarUrl!,
                              fit: BoxFit.cover,
                              errorBuilder: (context, error, stackTrace) =>
                                  _buildInitialAvatar(),
                            ),
                          )
                        : _buildInitialAvatar(),
                  ),
                ),
                // Label at bottom
                Positioned(
                  bottom: -24,
                  left: 0,
                  right: 0,
                  child: Center(
                    child: Container(
                      padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                      decoration: BoxDecoration(
                        color: Colors.black.withOpacity(0.8),
                        border: Border.all(
                          color: widget.color.withOpacity(0.3),
                          width: 1,
                        ),
                        borderRadius: BorderRadius.circular(4),
                      ),
                      child: Text(
                        widget.label,
                        style: Theme.of(context).textTheme.bodySmall?.copyWith(
                              color: widget.color,
                              fontWeight: FontWeight.w500,
                            ),
                        overflow: TextOverflow.ellipsis,
                        maxLines: 1,
                      ),
                    ),
                  ),
                ),
                // Stat badge at top-right
                if (widget.stat != null)
                  Positioned(
                    top: -8,
                    right: -8,
                    child: Container(
                      padding: const EdgeInsets.all(4),
                      decoration: BoxDecoration(
                        shape: BoxShape.circle,
                        color: widget.color,
                        boxShadow: [
                          BoxShadow(
                            color: widget.color.withOpacity(0.5),
                            blurRadius: 4,
                          ),
                        ],
                      ),
                      child: Text(
                        widget.stat!,
                        style: const TextStyle(
                          color: Colors.black,
                          fontSize: 10,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                    ),
                  ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildInitialAvatar() {
    return Container(
      decoration: BoxDecoration(
        shape: BoxShape.circle,
        gradient: LinearGradient(
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
          colors: [
            widget.color.withOpacity(0.6),
            widget.color.withOpacity(0.2),
          ],
        ),
      ),
      child: Center(
        child: Text(
          widget.label.isNotEmpty ? widget.label[0].toUpperCase() : '?',
          style: TextStyle(
            color: widget.color,
            fontWeight: FontWeight.bold,
            fontSize: 18,
          ),
        ),
      ),
    );
  }
}

class NetworkGraphWidget extends StatefulWidget {
  final List<NetworkNodeData> nodes;
  final List<NetworkConnectionData> connections;
  final double width;
  final double height;

  const NetworkGraphWidget({
    Key? key,
    required this.nodes,
    required this.connections,
    this.width = 800,
    this.height = 600,
  }) : super(key: key);

  @override
  State<NetworkGraphWidget> createState() => _NetworkGraphWidgetState();
}

class _NetworkGraphWidgetState extends State<NetworkGraphWidget> {
  late Map<String, Offset> _nodePositions;

  @override
  void initState() {
    super.initState();
    _initializePositions();
  }

  void _initializePositions() {
    _nodePositions = {};
    for (var node in widget.nodes) {
      _nodePositions[node.id] = node.initialPosition;
    }
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      width: widget.width,
      height: widget.height,
      decoration: BoxDecoration(
        border: Border.all(color: Colors.cyan.withOpacity(0.3), width: 1),
        borderRadius: BorderRadius.circular(8),
        color: Colors.black.withOpacity(0.3),
      ),
      child: Stack(
        children: [
          // Connections canvas
          CustomPaint(
            painter: NetworkConnectionPainter(
              connections: widget.connections,
              nodePositions: _nodePositions,
            ),
            size: Size(widget.width, widget.height),
          ),
          // Nodes
          ..._buildNodes(),
        ],
      ),
    );
  }

  List<Widget> _buildNodes() {
    return widget.nodes.map((node) {
      return NetworkNodeWidget(
        id: node.id,
        label: node.label,
        initialPosition: node.initialPosition,
        color: node.color,
        stat: node.stat,
        onDragStart: () {},
        onDragUpdate: (newPosition) {
          setState(() {
            _nodePositions[node.id] = newPosition;
          });
        },
        onDragEnd: () {},
        onTap: () => node.onTap?.call(),
      );
    }).toList();
  }
}

class NetworkNodeData {
  final String id;
  final String label;
  final Offset initialPosition;
  final Color color;
  final String? avatarUrl;
  final String? stat;
  final VoidCallback? onTap;

  NetworkNodeData({
    required this.id,
    required this.label,
    required this.initialPosition,
    this.color = Colors.cyan,
    this.avatarUrl,
    this.stat,
    this.onTap,
  });
}

class NetworkConnectionData {
  final String fromNodeId;
  final String toNodeId;
  final double strength; // -10 to 10
  final Color? customColor;

  NetworkConnectionData({
    required this.fromNodeId,
    required this.toNodeId,
    this.strength = 0,
    this.customColor,
  });
}

class NetworkConnectionPainter extends CustomPainter {
  final List<NetworkConnectionData> connections;
  final Map<String, Offset> nodePositions;

  NetworkConnectionPainter({
    required this.connections,
    required this.nodePositions,
  });

  @override
  void paint(Canvas canvas, Size size) {
    for (var connection in connections) {
      final fromPos = nodePositions[connection.fromNodeId];
      final toPos = nodePositions[connection.toNodeId];

      if (fromPos == null || toPos == null) continue;

      final normalized = (connection.strength + 10) / 20;
      final color = connection.customColor ?? _getConnectionColor(normalized);
      final strokeWidth = (connection.strength.abs() / 10) * 3 + 1;

      final paint = Paint()
        ..color = color.withOpacity(normalized * 0.8)
        ..strokeWidth = strokeWidth
        ..strokeCap = StrokeCap.round;

      canvas.drawLine(fromPos, toPos, paint);

      // Draw strength indicator at midpoint
      final midpoint = Offset(
        (fromPos.dx + toPos.dx) / 2,
        (fromPos.dy + toPos.dy) / 2,
      );

      if (connection.strength != 0) {
        final label = connection.strength > 0
            ? '+${connection.strength.toStringAsFixed(1)}'
            : connection.strength.toStringAsFixed(1);

        final textPainter = TextPainter(
          text: TextSpan(
            text: label,
            style: TextStyle(
              color: color,
              fontSize: 9,
              fontWeight: FontWeight.bold,
            ),
          ),
          textDirection: TextDirection.ltr,
        );
        textPainter.layout();
        textPainter.paint(
          canvas,
          Offset(
            midpoint.dx - textPainter.width / 2,
            midpoint.dy - textPainter.height / 2,
          ),
        );
      }
    }
  }

  Color _getConnectionColor(double normalized) {
    if (normalized < 0.3) {
      return Colors.red; // Very negative
    } else if (normalized < 0.5) {
      return Colors.orange; // Negative
    } else if (normalized < 0.7) {
      return Colors.yellow; // Neutral
    } else {
      return Colors.green; // Positive
    }
  }

  @override
  bool shouldRepaint(NetworkConnectionPainter oldDelegate) =>
      oldDelegate.connections != connections ||
      oldDelegate.nodePositions != nodePositions;
}
