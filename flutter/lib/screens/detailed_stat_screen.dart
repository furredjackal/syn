import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../models/game_state.dart';
import '../theme/theme.dart';
import '../widgets/stat_bar.dart';

class DetailedStatScreen extends StatefulWidget {
  const DetailedStatScreen({Key? key}) : super(key: key);

  @override
  State<DetailedStatScreen> createState() => _DetailedStatScreenState();
}

class _DetailedStatScreenState extends State<DetailedStatScreen> {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF0A0E27),
      appBar: AppBar(
        backgroundColor: Colors.black.withOpacity(0.5),
        title: Text('DETAILED STATS',
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
                _buildCategory(context, 'PHYSICAL STATS', [
                  ('Health', gameState.health),
                  ('Strength', gameState.strength),
                ]),
                const SizedBox(height: 24),
                _buildCategory(context, 'MENTAL STATS', [
                  ('Intelligence', gameState.intelligence),
                  ('Wisdom', gameState.wisdom),
                  ('Stability', gameState.stability),
                ]),
                const SizedBox(height: 24),
                _buildCategory(context, 'SOCIAL STATS', [
                  ('Charisma', gameState.charisma),
                ]),
                const SizedBox(height: 24),
                _buildCategory(context, 'ECONOMIC STATS', [
                  ('Wealth', gameState.wealth),
                ]),
                const SizedBox(height: 24),
                _buildTracking(context, gameState),
              ],
            ),
          );
        },
      ),
    );
  }

  Widget _buildCategory(
      BuildContext context, String title, List<(String, int)> stats) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(title,
            style: Theme.of(context)
                .textTheme
                .titleSmall
                ?.copyWith(color: const Color(0xFF00D9FF))),
        const SizedBox(height: 12),
        ...stats
            .map((stat) => Padding(
                  padding: const EdgeInsets.only(bottom: 16),
                  child: StatBar(label: stat.$1, value: stat.$2),
                ))
            .toList(),
      ],
    );
  }

  Widget _buildTracking(BuildContext context, GameState gameState) {
    return Container(
      padding: const EdgeInsets.all(16),
      decoration: BoxDecoration(
        border: Border.all(color: const Color(0xFF9D4EDD), width: 1),
        color: const Color(0xFF9D4EDD).withOpacity(0.1),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('NARRATIVE TRACKING',
              style: Theme.of(context)
                  .textTheme
                  .titleSmall
                  ?.copyWith(color: const Color(0xFF9D4EDD))),
          const SizedBox(height: 12),
          Row(
            children: [
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text('Mood', style: Theme.of(context).textTheme.bodySmall),
                    Text(gameState.mood.toString(),
                        style: Theme.of(context).textTheme.titleSmall?.copyWith(
                            color:
                                MoodColors.forMood(gameState.mood.toDouble()))),
                  ],
                ),
              ),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text('Karma', style: Theme.of(context).textTheme.bodySmall),
                    Text(gameState.karma.toString(),
                        style: Theme.of(context).textTheme.titleSmall?.copyWith(
                            color: KarmaColors.forKarma(gameState.karma))),
                  ],
                ),
              ),
            ],
          ),
        ],
      ),
    );
  }
}
