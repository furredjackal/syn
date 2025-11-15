import 'package:flutter/material.dart';

class SaveSlotWidget extends StatefulWidget {
  final int slotNumber;
  final String? characterName;
  final DateTime? lastSaved;
  final String? location;
  final int? playtimeMinutes;
  final bool isEmpty;
  final VoidCallback onLoad;
  final VoidCallback onDelete;

  const SaveSlotWidget({
    Key? key,
    required this.slotNumber,
    this.characterName,
    this.lastSaved,
    this.location,
    this.playtimeMinutes,
    this.isEmpty = true,
    required this.onLoad,
    required this.onDelete,
  }) : super(key: key);

  @override
  State<SaveSlotWidget> createState() => _SaveSlotWidgetState();
}

class _SaveSlotWidgetState extends State<SaveSlotWidget> {
  bool _isHovering = false;

  String _formatDate(DateTime date) {
    return '${date.month}/${date.day}/${date.year} ${date.hour}:${date.minute.toString().padLeft(2, '0')}';
  }

  String _formatPlaytime(int minutes) {
    final hours = minutes ~/ 60;
    final mins = minutes % 60;
    return '${hours}h ${mins}m';
  }

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _isHovering = true),
      onExit: (_) => setState(() => _isHovering = false),
      child: GestureDetector(
        onTap: widget.isEmpty ? null : widget.onLoad,
        child: Container(
          padding: const EdgeInsets.all(16),
          decoration: BoxDecoration(
            border: Border.all(
              color: widget.isEmpty
                  ? Colors.grey.withOpacity(0.3)
                  : (_isHovering ? Colors.cyan : Colors.cyan.withOpacity(0.5)),
              width: 2,
            ),
            borderRadius: BorderRadius.circular(8),
            color: Colors.black.withOpacity(0.5),
            boxShadow: _isHovering && !widget.isEmpty
                ? [
                    BoxShadow(
                      color: Colors.cyan.withOpacity(0.3),
                      blurRadius: 12,
                      spreadRadius: 2,
                    ),
                  ]
                : [],
          ),
          child: widget.isEmpty
              ? _buildEmptySlot()
              : _buildFilledSlot(),
        ),
      ),
    );
  }

  Widget _buildEmptySlot() {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(
            Icons.add,
            color: Colors.grey,
            size: 32,
          ),
          const SizedBox(height: 8),
          Text(
            'Slot ${widget.slotNumber}',
            style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                  color: Colors.grey,
                ),
          ),
          const SizedBox(height: 4),
          Text(
            'Empty',
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  color: Colors.grey.withOpacity(0.6),
                ),
          ),
        ],
      ),
    );
  }

  Widget _buildFilledSlot() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  widget.characterName ?? 'Unknown',
                  style: Theme.of(context).textTheme.titleMedium?.copyWith(
                        color: Colors.cyan,
                        fontWeight: FontWeight.bold,
                      ),
                ),
                const SizedBox(height: 4),
                Text(
                  'Slot ${widget.slotNumber}',
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                        color: Colors.grey.withOpacity(0.7),
                      ),
                ),
              ],
            ),
            if (_isHovering)
              GestureDetector(
                onTap: widget.onDelete,
                child: Tooltip(
                  message: 'Delete Save',
                  child: Icon(
                    Icons.delete,
                    color: Colors.red,
                    size: 20,
                  ),
                ),
              ),
          ],
        ),
        const SizedBox(height: 12),
        Divider(color: Colors.cyan.withOpacity(0.2)),
        const SizedBox(height: 8),
        if (widget.location != null) ...[
          Row(
            children: [
              Icon(Icons.location_on, color: Colors.purple, size: 14),
              const SizedBox(width: 6),
              Text(
                widget.location!,
                style: Theme.of(context).textTheme.bodySmall?.copyWith(
                      color: Colors.grey[400],
                    ),
              ),
            ],
          ),
          const SizedBox(height: 6),
        ],
        if (widget.playtimeMinutes != null) ...[
          Row(
            children: [
              Icon(Icons.timer, color: Colors.amber, size: 14),
              const SizedBox(width: 6),
              Text(
                _formatPlaytime(widget.playtimeMinutes!),
                style: Theme.of(context).textTheme.bodySmall?.copyWith(
                      color: Colors.grey[400],
                    ),
              ),
            ],
          ),
          const SizedBox(height: 6),
        ],
        if (widget.lastSaved != null) ...[
          Row(
            children: [
              Icon(Icons.access_time, color: Colors.green, size: 14),
              const SizedBox(width: 6),
              Text(
                _formatDate(widget.lastSaved!),
                style: Theme.of(context).textTheme.bodySmall?.copyWith(
                      color: Colors.grey[400],
                    ),
              ),
            ],
          ),
        ],
      ],
    );
  }
}
