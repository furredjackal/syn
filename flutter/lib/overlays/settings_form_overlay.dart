import 'package:flutter/material.dart';

import '../components/ui/syn_theme.dart';
import '../syn_game.dart';

Widget buildSettingsFormOverlay(BuildContext context, SynGame game) {
  return _SettingsFormOverlay(game: game);
}

class _SettingsFormOverlay extends StatefulWidget {
  const _SettingsFormOverlay({required this.game});

  final SynGame game;

  @override
  State<_SettingsFormOverlay> createState() => _SettingsFormOverlayState();
}

class _SettingsFormOverlayState extends State<_SettingsFormOverlay> {
  bool _sfwMode = false;
  double _volume = 0.8;

  void _close() {
    widget.game.overlays.remove('settings_form');
    widget.game.resumeEngine();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: SynColors.bgOverlay,
      body: Center(
        child: Container(
          width: 520,
          padding: const EdgeInsets.all(SynLayout.paddingLarge),
          decoration: BoxDecoration(
            color: SynColors.bgPanel,
            border: Border.all(
              color: SynColors.primaryCyan,
              width: SynLayout.borderWidthNormal,
            ),
          ),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text('SETTINGS', style: SynTextStyles.h1Event),
              const SizedBox(height: SynLayout.paddingMedium),
              SwitchListTile(
                value: _sfwMode,
                onChanged: (value) => setState(() => _sfwMode = value),
                title: const Text('SFW Mode'),
                subtitle: const Text('Hide explicit content while playing'),
                activeColor: SynColors.primaryCyan,
                contentPadding: EdgeInsets.zero,
              ),
              ListTile(
                contentPadding: EdgeInsets.zero,
                title: const Text('Master Volume'),
                subtitle: Slider(
                  value: _volume,
                  onChanged: (value) => setState(() => _volume = value),
                ),
              ),
              const SizedBox(height: SynLayout.paddingMedium),
              Align(
                alignment: Alignment.centerRight,
                child: ElevatedButton(
                  onPressed: _close,
                  child: const Text('Close'),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
