import 'package:flutter/material.dart';

class ItemCardWidget extends StatelessWidget {
  final String name;
  final String description;
  final IconData icon;
  final Map<String, int> effects;
  final int rarity;
  final VoidCallback? onTap;

  const ItemCardWidget({
    Key? key,
    required this.name,
    required this.description,
    required this.icon,
    this.effects = const {},
    this.rarity = 1,
    this.onTap,
  }) : super(key: key);

  Color _getRarityColor() {
    switch (rarity) {
      case 5:
        return Colors.amber;
      case 4:
        return Colors.purple;
      case 3:
        return Colors.blue;
      case 2:
        return Colors.green;
      default:
        return Colors.grey;
    }
  }

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: Container(
        decoration: BoxDecoration(
          border: Border.all(color: _getRarityColor().withOpacity(0.6), width: 1.5),
          borderRadius: BorderRadius.circular(4),
          color: Colors.black.withOpacity(0.3),
        ),
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                Icon(icon, color: _getRarityColor(), size: 28),
                const SizedBox(width: 8),
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(name, style: Theme.of(context).textTheme.bodyMedium?.copyWith(fontWeight: FontWeight.bold)),
                      Text('★ ' * rarity, style: Theme.of(context).textTheme.labelSmall?.copyWith(color: _getRarityColor())),
                    ],
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            Text(description, style: Theme.of(context).textTheme.bodySmall, maxLines: 2, overflow: TextOverflow.ellipsis),
            if (effects.isNotEmpty) ...[
              const SizedBox(height: 8),
              Wrap(
                spacing: 6,
                children: effects.entries.map((e) {
                  final color = e.value > 0 ? Colors.green : Colors.red;
                  return Text(
                    '${e.key} ${e.value > 0 ? '+' : ''}${e.value}',
                    style: Theme.of(context).textTheme.labelSmall?.copyWith(color: color, fontSize: 10),
                  );
                }).toList(),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
