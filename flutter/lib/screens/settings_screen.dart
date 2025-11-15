import 'package:flutter/material.dart';
import 'package:provider/provider.dart';
import '../models/game_state.dart';

class SettingsScreen extends StatefulWidget {
  const SettingsScreen({Key? key}) : super(key: key);

  @override
  State<SettingsScreen> createState() => _SettingsScreenState();
}

class _SettingsScreenState extends State<SettingsScreen> {
  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF0A0E27),
      appBar: AppBar(
        backgroundColor: Colors.black.withOpacity(0.5),
        title: Text(
          'SETTINGS',
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
                // Audio Settings
                _SettingsSection(
                  title: 'AUDIO',
                  children: [
                    _SettingsToggle(
                      label: 'Sound',
                      value: gameState.soundEnabled,
                      onChanged: (value) => gameState.toggleSound(),
                    ),
                    _SettingsToggle(
                      label: 'Music',
                      value: gameState.musicEnabled,
                      onChanged: (value) => gameState.toggleMusic(),
                    ),
                    _SettingsSlider(
                      label: 'Master Volume',
                      value: gameState.masterVolume,
                      onChanged: (value) => gameState.setMasterVolume(value),
                    ),
                  ],
                ),
                const SizedBox(height: 24),

                // Accessibility Settings
                _SettingsSection(
                  title: 'ACCESSIBILITY',
                  children: [
                    _SettingsToggle(
                      label: 'Color Blind Mode',
                      value: gameState.colorBlindMode,
                      onChanged: (value) => gameState.toggleColorBlindMode(),
                    ),
                    _SettingsToggle(
                      label: 'Reduced Motion',
                      value: gameState.reducedMotion,
                      onChanged: (value) => gameState.toggleReducedMotion(),
                    ),
                    _SettingsSlider(
                      label: 'Font Size',
                      value: gameState.fontSize,
                      min: 0.8,
                      max: 1.5,
                      onChanged: (value) => gameState.setFontSize(value),
                    ),
                  ],
                ),
                const SizedBox(height: 24),

                // About
                _SettingsSection(
                  title: 'ABOUT',
                  children: [
                    Padding(
                      padding: const EdgeInsets.symmetric(vertical: 12),
                      child: Row(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          Text(
                            'Version',
                            style: Theme.of(context).textTheme.bodyMedium,
                          ),
                          Text(
                            'v0.1.0',
                            style: Theme.of(context)
                                .textTheme
                                .bodyMedium
                                ?.copyWith(
                                  color: const Color(0xFF00D9FF),
                                ),
                          ),
                        ],
                      ),
                    ),
                  ],
                ),
              ],
            ),
          );
        },
      ),
    );
  }
}

class _SettingsSection extends StatelessWidget {
  final String title;
  final List<Widget> children;

  const _SettingsSection({
    required this.title,
    required this.children,
    Key? key,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          title,
          style: Theme.of(context).textTheme.titleSmall?.copyWith(
                color: const Color(0xFF00D9FF),
              ),
        ),
        const SizedBox(height: 12),
        Container(
          decoration: BoxDecoration(
            border: Border.all(
              color: const Color(0xFF00D9FF).withOpacity(0.3),
              width: 1,
            ),
            color: Colors.black.withOpacity(0.2),
          ),
          child: Column(
            children: List.generate(
              children.length,
              (index) => Column(
                children: [
                  children[index],
                  if (index < children.length - 1)
                    Container(
                      height: 1,
                      color: const Color(0xFF00D9FF).withOpacity(0.1),
                    ),
                ],
              ),
            ),
          ),
        ),
      ],
    );
  }
}

class _SettingsToggle extends StatefulWidget {
  final String label;
  final bool value;
  final Function(bool) onChanged;

  const _SettingsToggle({
    required this.label,
    required this.value,
    required this.onChanged,
    Key? key,
  }) : super(key: key);

  @override
  State<_SettingsToggle> createState() => _SettingsToggleState();
}

class _SettingsToggleState extends State<_SettingsToggle> {
  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 12),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(
            widget.label,
            style: Theme.of(context).textTheme.bodyMedium,
          ),
          GestureDetector(
            onTap: () => widget.onChanged(!widget.value),
            child: Container(
              width: 50,
              height: 28,
              decoration: BoxDecoration(
                border: Border.all(
                  color: const Color(0xFF00D9FF),
                  width: 1,
                ),
                color: widget.value
                    ? const Color(0xFF00D9FF).withOpacity(0.3)
                    : Colors.transparent,
              ),
              child: Align(
                alignment:
                    widget.value ? Alignment.centerRight : Alignment.centerLeft,
                child: Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 2),
                  child: Container(
                    width: 22,
                    height: 22,
                    decoration: BoxDecoration(
                      color: widget.value
                          ? const Color(0xFF00D9FF)
                          : Colors.white.withOpacity(0.3),
                    ),
                  ),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

class _SettingsSlider extends StatefulWidget {
  final String label;
  final double value;
  final double min;
  final double max;
  final Function(double) onChanged;

  const _SettingsSlider({
    required this.label,
    required this.value,
    this.min = 0.0,
    this.max = 1.0,
    required this.onChanged,
    Key? key,
  }) : super(key: key);

  @override
  State<_SettingsSlider> createState() => _SettingsSliderState();
}

class _SettingsSliderState extends State<_SettingsSlider> {
  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Row(
            mainAxisAlignment: MainAxisAlignment.spaceBetween,
            children: [
              Text(
                widget.label,
                style: Theme.of(context).textTheme.bodyMedium,
              ),
              Text(
                widget.value.toStringAsFixed(2),
                style: Theme.of(context).textTheme.bodySmall?.copyWith(
                      color: const Color(0xFF00D9FF),
                    ),
              ),
            ],
          ),
          const SizedBox(height: 8),
          Slider(
            value: widget.value,
            min: widget.min,
            max: widget.max,
            onChanged: widget.onChanged,
            activeColor: const Color(0xFF00D9FF),
            inactiveColor: const Color(0xFF00D9FF).withOpacity(0.2),
          ),
        ],
      ),
    );
  }
}
