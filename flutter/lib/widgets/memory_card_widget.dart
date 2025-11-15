import 'package:flutter/material.dart';

class MemoryCardWidget extends StatefulWidget {
  final String title;
  final String description;
  final DateTime date;
  final double emotionalIntensity;
  final List<String> tags;
  final VoidCallback? onTap;

  const MemoryCardWidget({
    Key? key,
    required this.title,
    required this.description,
    required this.date,
    required this.emotionalIntensity,
    this.tags = const [],
    this.onTap,
  }) : super(key: key);

  @override
  State<MemoryCardWidget> createState() => _MemoryCardWidgetState();
}

class _MemoryCardWidgetState extends State<MemoryCardWidget> {
  bool _isExpanded = false;

  Color _getIntensityColor() {
    if (widget.emotionalIntensity > 0.5) return Colors.green;
    if (widget.emotionalIntensity > 0) return Colors.yellow;
    if (widget.emotionalIntensity < -0.5) return Colors.red;
    return Colors.grey;
  }

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: () => setState(() => _isExpanded = !_isExpanded),
      child: Container(
        decoration: BoxDecoration(
          border: Border.all(color: _getIntensityColor().withOpacity(0.5), width: 1),
          borderRadius: BorderRadius.circular(4),
          color: Colors.black.withOpacity(0.3),
        ),
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Expanded(
                  child: Text(widget.title, style: Theme.of(context).textTheme.bodyMedium?.copyWith(fontWeight: FontWeight.bold)),
                ),
                Icon(_isExpanded ? Icons.expand_less : Icons.expand_more, color: _getIntensityColor()),
              ],
            ),
            const SizedBox(height: 4),
            Text(
              '${widget.date.month}/${widget.date.day}/${widget.date.year}',
              style: Theme.of(context).textTheme.labelSmall?.copyWith(color: Colors.grey),
            ),
            if (_isExpanded) ...[
              const SizedBox(height: 8),
              Text(widget.description, style: Theme.of(context).textTheme.bodySmall),
              if (widget.tags.isNotEmpty) ...[
                const SizedBox(height: 8),
                Wrap(
                  spacing: 4,
                  children: widget.tags.map((tag) =>
                    Container(
                      padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 2),
                      decoration: BoxDecoration(
                        border: Border.all(color: Colors.cyan.withOpacity(0.3), width: 0.5),
                        borderRadius: BorderRadius.circular(2),
                      ),
                      child: Text(tag, style: Theme.of(context).textTheme.labelSmall?.copyWith(color: Colors.cyan, fontSize: 10)),
                    ),
                  ).toList(),
                ),
              ],
            ],
          ],
        ),
      ),
    );
  }
}
