import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../models/game_state.dart';

class MemoryJournalScreen extends StatefulWidget {
  const MemoryJournalScreen({Key? key}) : super(key: key);

  @override
  State<MemoryJournalScreen> createState() => _MemoryJournalScreenState();
}

class _MemoryJournalScreenState extends State<MemoryJournalScreen> {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF0A0E27),
      appBar: AppBar(
        backgroundColor: Colors.black.withOpacity(0.5),
        title: Text(
          'MEMORY JOURNAL',
          style: Theme.of(context).textTheme.titleMedium?.copyWith(
                color: const Color(0xFF00D9FF),
              ),
        ),
        leading: IconButton(
          icon: const Icon(Icons.arrow_back),
          onPressed: () => Navigator.pop(context),
        ),
      ),
      body: Consumer<GameState>(
        builder: (context, gameState, _) {
          return SingleChildScrollView(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                if (gameState.memories.isEmpty)
                  Center(
                    child: Padding(
                      padding: const EdgeInsets.symmetric(vertical: 32),
                      child: Text(
                        'No memories recorded yet.',
                        style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                              color: Colors.white.withOpacity(0.5),
                            ),
                      ),
                    ),
                  )
                else
                  ...gameState.memories.asMap().entries.map((entry) {
                    final memory = entry.value;
                    return Padding(
                      padding: const EdgeInsets.only(bottom: 16),
                      child: _MemoryCard(memory: memory),
                    );
                  }).toList(),
              ],
            ),
          );
        },
      ),
    );
  }
}

class _MemoryCard extends StatefulWidget {
  final MemoryEntry memory;

  const _MemoryCard({
    required this.memory,
    Key? key,
  }) : super(key: key);

  @override
  State<_MemoryCard> createState() => _MemoryCardState();
}

class _MemoryCardState extends State<_MemoryCard> {
  bool _isExpanded = false;

  @override
  Widget build(BuildContext context) {
    final emotionalColor = widget.memory.emotionalIntensity > 0
        ? const Color(0xFF00FF00)
        : const Color(0xFFFF4444);

    return Container(
      decoration: BoxDecoration(
        border: Border.all(
          color: emotionalColor.withOpacity(0.5),
          width: 1,
        ),
        color: emotionalColor.withOpacity(0.05),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          GestureDetector(
            onTap: () => setState(() => _isExpanded = !_isExpanded),
            child: Padding(
              padding: const EdgeInsets.all(12),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Expanded(
                        child: Text(
                          widget.memory.eventTitle.toUpperCase(),
                          style:
                              Theme.of(context).textTheme.bodyMedium?.copyWith(
                                    color: emotionalColor,
                                    fontWeight: FontWeight.bold,
                                  ),
                          overflow: TextOverflow.ellipsis,
                        ),
                      ),
                      Icon(
                        _isExpanded ? Icons.expand_less : Icons.expand_more,
                        color: emotionalColor,
                      ),
                    ],
                  ),
                  const SizedBox(height: 8),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceBetween,
                    children: [
                      Text(
                        _formatDate(widget.memory.timestamp),
                        style: Theme.of(context).textTheme.bodySmall?.copyWith(
                              color: Colors.white.withOpacity(0.5),
                              fontSize: 11,
                            ),
                      ),
                      Container(
                        padding: const EdgeInsets.symmetric(
                          horizontal: 8,
                          vertical: 4,
                        ),
                        decoration: BoxDecoration(
                          border: Border.all(
                            color: emotionalColor.withOpacity(0.5),
                          ),
                        ),
                        child: Text(
                          widget.memory.emotionalIntensity > 0
                              ? '+${widget.memory.emotionalIntensity.toStringAsFixed(1)}'
                              : widget.memory.emotionalIntensity
                                  .toStringAsFixed(1),
                          style:
                              Theme.of(context).textTheme.bodySmall?.copyWith(
                                    color: emotionalColor,
                                    fontSize: 11,
                                  ),
                        ),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ),
          if (_isExpanded) ...[
            Container(
              height: 1,
              color: emotionalColor.withOpacity(0.2),
            ),
            Padding(
              padding: const EdgeInsets.all(12),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    widget.memory.description,
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                  if (widget.memory.tags.isNotEmpty) ...[
                    const SizedBox(height: 12),
                    Wrap(
                      spacing: 8,
                      runSpacing: 8,
                      children: widget.memory.tags.map((tag) {
                        return Container(
                          padding: const EdgeInsets.symmetric(
                            horizontal: 8,
                            vertical: 4,
                          ),
                          decoration: BoxDecoration(
                            border: Border.all(
                              color: const Color(0xFF9D4EDD).withOpacity(0.5),
                            ),
                            color: const Color(0xFF9D4EDD).withOpacity(0.1),
                          ),
                          child: Text(
                            tag,
                            style:
                                Theme.of(context).textTheme.bodySmall?.copyWith(
                                      color: const Color(0xFF9D4EDD),
                                      fontSize: 10,
                                    ),
                          ),
                        );
                      }).toList(),
                    ),
                  ],
                ],
              ),
            ),
          ],
        ],
      ),
    );
  }

  String _formatDate(DateTime date) {
    return '${date.year}-${date.month.toString().padLeft(2, '0')}-${date.day.toString().padLeft(2, '0')}';
  }
}
