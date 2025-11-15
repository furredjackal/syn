import 'package:flutter/material.dart';
import '../models/game_state.dart';

class CharacterInfo extends StatelessWidget {
  final RelationshipData relationship;

  const CharacterInfo({
    required this.relationship,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: BoxDecoration(
        border: Border.all(
          color: const Color(0xFF00D9FF).withOpacity(0.3),
          width: 1,
        ),
        color: Colors.black.withOpacity(0.3),
      ),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Expanded(
                child: Text(
                  relationship.npcName.toUpperCase(),
                  style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                        color: const Color(0xFF00D9FF),
                        fontWeight: FontWeight.bold,
                      ),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
              Container(
                padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                decoration: BoxDecoration(
                  border: Border.all(
                    color: _stateColor,
                    width: 1,
                  ),
                  color: _stateColor.withOpacity(0.1),
                ),
                child: Text(
                  relationship.state,
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                        color: _stateColor,
                        fontSize: 10,
                      ),
                ),
              ),
            ],
          ),
          const SizedBox(height: 8),
          _RelationshipAxis(
            label: 'Affection',
            value: relationship.affection,
            maxValue: 10,
          ),
          const SizedBox(height: 6),
          _RelationshipAxis(
            label: 'Trust',
            value: relationship.trust,
            maxValue: 10,
          ),
          const SizedBox(height: 6),
          _RelationshipAxis(
            label: 'Attraction',
            value: relationship.attraction,
            maxValue: 10,
          ),
          const SizedBox(height: 6),
          _RelationshipAxis(
            label: 'Familiarity',
            value: relationship.familiarity,
            maxValue: 10,
          ),
          const SizedBox(height: 6),
          _RelationshipAxis(
            label: 'Resentment',
            value: relationship.resentment,
            maxValue: 10,
          ),
        ],
      ),
    );
  }

  Color get _stateColor {
    switch (relationship.state) {
      case 'Friend':
      case 'CloseFriend':
      case 'BestFriend':
        return const Color(0xFF00FF00);
      case 'RomanticInterest':
      case 'Partner':
      case 'Spouse':
        return const Color(0xFFFF1493);
      case 'Rival':
      case 'Estranged':
      case 'BrokenHeart':
        return const Color(0xFFFF4444);
      default:
        return const Color(0xFFFFAA00);
    }
  }
}

class _RelationshipAxis extends StatelessWidget {
  final String label;
  final double value;
  final double maxValue;

  const _RelationshipAxis({
    required this.label,
    required this.value,
    required this.maxValue,
    Key? key,
  }) : super(key: key);

  Color get _axisColor {
    if (value < -5) return const Color(0xFFFF0000);
    if (value < 0) return const Color(0xFFFF8800);
    if (value < 5) return const Color(0xFFFFAA00);
    return const Color(0xFF00FF00);
  }

  @override
  Widget build(BuildContext context) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceBetween,
      children: [
        Text(
          label,
          style: Theme.of(context).textTheme.bodySmall?.copyWith(
                color: Colors.white.withOpacity(0.7),
                fontSize: 11,
              ),
        ),
        Text(
          value.toStringAsFixed(1),
          style: Theme.of(context).textTheme.bodySmall?.copyWith(
                color: _axisColor,
                fontWeight: FontWeight.bold,
                fontSize: 11,
              ),
        ),
      ],
    );
  }
}
