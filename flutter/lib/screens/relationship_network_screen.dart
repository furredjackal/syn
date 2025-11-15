import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../models/game_state.dart';

class RelationshipNetworkScreen extends StatefulWidget {
  const RelationshipNetworkScreen({Key? key}) : super(key: key);

  @override
  State<RelationshipNetworkScreen> createState() =>
      _RelationshipNetworkScreenState();
}

class _RelationshipNetworkScreenState extends State<RelationshipNetworkScreen> {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF0A0E27),
      appBar: AppBar(
        backgroundColor: Colors.black.withOpacity(0.5),
        title: Text('RELATIONSHIP NETWORK',
            style: Theme.of(context)
                .textTheme
                .titleMedium
                ?.copyWith(color: const Color(0xFF00D9FF))),
        leading: IconButton(
            icon: const Icon(Icons.arrow_back),
            onPressed: () => Navigator.pop(context)),
      ),
      body: Consumer<GameState>(
        builder: (context, gameState, _) {
          return SingleChildScrollView(
            padding: const EdgeInsets.all(16),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text('Total Relationships: ${gameState.relationships.length}',
                    style: Theme.of(context).textTheme.bodyLarge),
                const SizedBox(height: 24),
                if (gameState.relationships.isEmpty)
                  Center(
                    child: Padding(
                      padding: const EdgeInsets.symmetric(vertical: 32),
                      child: Text('No relationships yet.',
                          style: Theme.of(context)
                              .textTheme
                              .bodyMedium
                              ?.copyWith(color: Colors.white.withOpacity(0.5))),
                    ),
                  )
                else
                  ...gameState.relationships
                      .map((rel) => Padding(
                            padding: const EdgeInsets.only(bottom: 16),
                            child: _RelationshipTile(relationship: rel),
                          ))
                      .toList(),
              ],
            ),
          );
        },
      ),
    );
  }
}

class _RelationshipTile extends StatelessWidget {
  final RelationshipData relationship;

  const _RelationshipTile({required this.relationship, Key? key})
      : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        border: Border.all(
            color: const Color(0xFF00D9FF).withOpacity(0.3), width: 1),
        color: Colors.black.withOpacity(0.3),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(relationship.npcName,
              style: Theme.of(context)
                  .textTheme
                  .bodyLarge
                  ?.copyWith(color: const Color(0xFF00D9FF))),
          const SizedBox(height: 8),
          Text('State: ${relationship.state}',
              style: Theme.of(context).textTheme.bodySmall),
          const SizedBox(height: 8),
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              _AxisDisplay('Affection', relationship.affection),
              _AxisDisplay('Trust', relationship.trust),
              _AxisDisplay('Attraction', relationship.attraction),
            ],
          ),
        ],
      ),
    );
  }
}

class _AxisDisplay extends StatelessWidget {
  final String label;
  final double value;

  const _AxisDisplay(this.label, this.value, {Key? key}) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Text(label,
            style:
                Theme.of(context).textTheme.bodySmall?.copyWith(fontSize: 10)),
        Text(value.toStringAsFixed(1),
            style: Theme.of(context)
                .textTheme
                .bodySmall
                ?.copyWith(color: value > 0 ? Colors.green : Colors.red)),
      ],
    );
  }
}
