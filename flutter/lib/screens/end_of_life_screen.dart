import 'package:flutter/material.dart';

class EndOfLifeSummaryScreen extends StatefulWidget {
  const EndOfLifeSummaryScreen({Key? key}) : super(key: key);

  @override
  State<EndOfLifeSummaryScreen> createState() => _EndOfLifeSummaryScreenState();
}

class _EndOfLifeSummaryScreenState extends State<EndOfLifeSummaryScreen> {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF0A0E27),
      appBar: AppBar(
        backgroundColor: Colors.black.withOpacity(0.5),
        automaticallyImplyLeading: false,
        title: Text(
          'LIFE CONCLUDED',
          style: Theme.of(context)
              .textTheme
              .titleMedium
              ?.copyWith(color: const Color(0xFFD9A000)),
        ),
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(24),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.center,
          children: [
            Text('Aria Nightwhisper',
                style: Theme.of(context)
                    .textTheme
                    .displaySmall
                    ?.copyWith(color: const Color(0xFFD9A000))),
            const SizedBox(height: 8),
            Text('1995 - 2068 (73 years)',
                style: Theme.of(context)
                    .textTheme
                    .titleMedium
                    ?.copyWith(color: Colors.grey)),
            const SizedBox(height: 32),
            Container(
              decoration: BoxDecoration(
                border: Border.all(
                    color: const Color(0xFFD9A000).withOpacity(0.5), width: 1),
                borderRadius: BorderRadius.circular(4),
                color: Colors.black.withOpacity(0.3),
              ),
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('LEGACY',
                      style: Theme.of(context)
                          .textTheme
                          .titleSmall
                          ?.copyWith(color: const Color(0xFFD9A000))),
                  const SizedBox(height: 12),
                  _buildLegacyItem(context, 'Relationships Formed',
                      '47 lasting connections'),
                  _buildLegacyItem(
                      context, 'Major Events', '156 significant moments'),
                  _buildLegacyItem(
                      context, 'Achievements Unlocked', '23 milestones'),
                  _buildLegacyItem(
                      context, 'Tragedy Survived', '8 life-altering crises'),
                  _buildLegacyItem(context, 'Final Mood', 'Serene (+6)'),
                ],
              ),
            ),
            const SizedBox(height: 24),
            Container(
              decoration: BoxDecoration(
                border: Border.all(color: Colors.grey.shade700, width: 1),
                borderRadius: BorderRadius.circular(4),
                color: Colors.black.withOpacity(0.3),
              ),
              padding: const EdgeInsets.all(16),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text('FINAL STATS',
                      style: Theme.of(context)
                          .textTheme
                          .titleSmall
                          ?.copyWith(color: const Color(0xFF00D9FF))),
                  const SizedBox(height: 12),
                  _buildStatRow(context, 'Health', '28/100'),
                  _buildStatRow(context, 'Wealth', '12,450'),
                  _buildStatRow(context, 'Reputation', '67/100'),
                  _buildStatRow(context, 'Knowledge', '78/100'),
                  _buildStatRow(context, 'Emotional Balance', '64/100'),
                ],
              ),
            ),
            const SizedBox(height: 32),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceEvenly,
              children: [
                ElevatedButton.icon(
                  onPressed: () => Navigator.pushNamed(context, '/menu'),
                  icon: const Icon(Icons.home),
                  label: const Text('MAIN MENU'),
                ),
                ElevatedButton.icon(
                  onPressed: () =>
                      Navigator.pushNamed(context, '/character_creation'),
                  icon: const Icon(Icons.refresh),
                  label: const Text('NEW LIFE'),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildLegacyItem(BuildContext context, String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: Theme.of(context).textTheme.bodySmall),
          Text(value,
              style: Theme.of(context)
                  .textTheme
                  .bodySmall
                  ?.copyWith(color: const Color(0xFFD9A000))),
        ],
      ),
    );
  }

  Widget _buildStatRow(BuildContext context, String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: Theme.of(context).textTheme.bodySmall),
          Text(value,
              style: Theme.of(context)
                  .textTheme
                  .bodySmall
                  ?.copyWith(color: Colors.cyan)),
        ],
      ),
    );
  }
}
