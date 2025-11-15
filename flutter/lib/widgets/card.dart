import 'package:flutter/material.dart';

class SynCard extends StatelessWidget {
  final Widget child;
  final Color borderColor;
  final EdgeInsets padding;
  final VoidCallback? onTap;
  final bool selectable;

  const SynCard({
    Key? key,
    required this.child,
    this.borderColor = const Color(0xFF00D9FF),
    this.padding = const EdgeInsets.all(16),
    this.onTap,
    this.selectable = false,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: Container(
        decoration: BoxDecoration(
          border: Border.all(color: borderColor.withOpacity(0.5), width: 1),
          borderRadius: BorderRadius.circular(4),
          color: Colors.black.withOpacity(0.3),
        ),
        padding: padding,
        child: child,
      ),
    );
  }
}

class NPCCard extends StatelessWidget {
  final String name;
  final String relationship;
  final int affection;
  final int trust;
  final VoidCallback onTap;

  const NPCCard({
    Key? key,
    required this.name,
    required this.relationship,
    required this.affection,
    required this.trust,
    required this.onTap,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return GestureDetector(
      onTap: onTap,
      child: Container(
        decoration: BoxDecoration(
          border: Border.all(color: Colors.white10, width: 1),
          borderRadius: BorderRadius.circular(4),
          color: Colors.black.withOpacity(0.3),
        ),
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(name, style: Theme.of(context).textTheme.bodyMedium?.copyWith(fontWeight: FontWeight.bold)),
            const SizedBox(height: 4),
            Text(relationship, style: Theme.of(context).textTheme.labelSmall?.copyWith(color: Colors.cyan)),
            const SizedBox(height: 8),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                _buildAxis(context, 'Affection', affection),
                _buildAxis(context, 'Trust', trust),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildAxis(BuildContext context, String label, int value) {
    final color = value > 5 ? Colors.green : value > 0 ? Colors.yellow : Colors.red;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(label, style: Theme.of(context).textTheme.labelSmall?.copyWith(color: Colors.grey, fontSize: 10)),
        Text('$value', style: Theme.of(context).textTheme.bodySmall?.copyWith(color: color)),
      ],
    );
  }
}
