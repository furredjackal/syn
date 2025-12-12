import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_animate/flutter_animate.dart';
import '../ui/widgets/persona_container.dart';

/// Settings Overlay - modal dialog for game settings
/// 
/// Props:
/// - onClose: Callback to close the overlay
/// - audioEnabled: Current audio state
/// - sfxEnabled: Current SFX state
/// - musicVolume: Current music volume (0.0-1.0)
/// - sfxVolume: Current SFX volume (0.0-1.0)
/// - onAudioToggle: Callback when audio is toggled
/// - onSfxToggle: Callback when SFX is toggled
/// - onMusicVolumeChange: Callback when music volume changes
/// - onSfxVolumeChange: Callback when SFX volume changes
class SettingsOverlay extends StatefulWidget {
  final VoidCallback onClose;
  final bool audioEnabled;
  final bool sfxEnabled;
  final double musicVolume;
  final double sfxVolume;
  final Function(bool) onAudioToggle;
  final Function(bool) onSfxToggle;
  final Function(double) onMusicVolumeChange;
  final Function(double) onSfxVolumeChange;

  const SettingsOverlay({
    super.key,
    required this.onClose,
    this.audioEnabled = true,
    this.sfxEnabled = true,
    this.musicVolume = 0.7,
    this.sfxVolume = 0.8,
    required this.onAudioToggle,
    required this.onSfxToggle,
    required this.onMusicVolumeChange,
    required this.onSfxVolumeChange,
  });

  @override
  State<SettingsOverlay> createState() => _SettingsOverlayState();
}

class _SettingsOverlayState extends State<SettingsOverlay> {
  late bool _audioEnabled;
  late bool _sfxEnabled;
  late double _musicVolume;
  late double _sfxVolume;

  @override
  void initState() {
    super.initState();
    _audioEnabled = widget.audioEnabled;
    _sfxEnabled = widget.sfxEnabled;
    _musicVolume = widget.musicVolume;
    _sfxVolume = widget.sfxVolume;
  }

  @override
  Widget build(BuildContext context) {
    return KeyboardListener(
      focusNode: FocusNode()..requestFocus(),
      onKeyEvent: _handleKeyEvent,
      child: Container(
        color: Colors.black.withOpacity(0.85),
        child: Center(
          child: ConstrainedBox(
            constraints: const BoxConstraints(maxWidth: 600, maxHeight: 700),
            child: PersonaContainer(
              skew: -0.15,
              color: Colors.black.withOpacity(0.95),
              child: Padding(
                padding: const EdgeInsets.all(40.0),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    _buildHeader(),
                    const SizedBox(height: 40),
                    _buildAudioSettings(),
                    const SizedBox(height: 40),
                    _buildCloseButton(),
                  ],
                ),
              ),
            ),
          )
              .animate()
              .scale(
                begin: const Offset(0.8, 0.8),
                duration: 300.ms,
                curve: Curves.easeOutBack,
              )
              .fadeIn(duration: 200.ms),
        ),
      ),
    );
  }

  Widget _buildHeader() {
    return Column(
      children: [
        Text(
          'SETTINGS',
          style: TextStyle(
            fontSize: 48,
            fontWeight: FontWeight.w900,
            color: const Color(0xFF00E6FF),
            letterSpacing: 6,
            shadows: [
              Shadow(
                color: const Color(0xFF00E6FF).withOpacity(0.5),
                blurRadius: 15,
              ),
            ],
          ),
          textAlign: TextAlign.center,
        ),
        const SizedBox(height: 15),
        Container(
          height: 2,
          width: 200,
          decoration: BoxDecoration(
            gradient: LinearGradient(
              colors: [
                Colors.transparent,
                const Color(0xFF00E6FF),
                Colors.transparent,
              ],
            ),
            boxShadow: [
              BoxShadow(
                color: const Color(0xFF00E6FF).withOpacity(0.5),
                blurRadius: 8,
              ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _buildAudioSettings() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          'AUDIO',
          style: TextStyle(
            fontSize: 28,
            fontWeight: FontWeight.w900,
            color: const Color(0xFF00E6FF),
            letterSpacing: 3,
          ),
        ),
        const SizedBox(height: 25),
        _buildToggleRow(
          label: 'Master Audio',
          value: _audioEnabled,
          onChanged: (value) {
            setState(() => _audioEnabled = value);
            widget.onAudioToggle(value);
          },
        ),
        const SizedBox(height: 20),
        _buildSliderRow(
          label: 'Music Volume',
          value: _musicVolume,
          enabled: _audioEnabled,
          onChanged: (value) {
            setState(() => _musicVolume = value);
            widget.onMusicVolumeChange(value);
          },
        ),
        const SizedBox(height: 20),
        _buildToggleRow(
          label: 'Sound Effects',
          value: _sfxEnabled,
          onChanged: (value) {
            setState(() => _sfxEnabled = value);
            widget.onSfxToggle(value);
          },
        ),
        const SizedBox(height: 20),
        _buildSliderRow(
          label: 'SFX Volume',
          value: _sfxVolume,
          enabled: _audioEnabled && _sfxEnabled,
          onChanged: (value) {
            setState(() => _sfxVolume = value);
            widget.onSfxVolumeChange(value);
          },
        ),
      ],
    );
  }

  Widget _buildToggleRow({
    required String label,
    required bool value,
    required Function(bool) onChanged,
  }) {
    return Row(
      mainAxisAlignment: MainAxisAlignment.spaceBetween,
      children: [
        Text(
          label,
          style: const TextStyle(
            fontSize: 20,
            color: Colors.white,
            fontWeight: FontWeight.w600,
          ),
        ),
        Switch(
          value: value,
          onChanged: onChanged,
          activeThumbColor: const Color(0xFF00E6FF),
          activeTrackColor: const Color(0xFF00E6FF).withOpacity(0.3),
        ),
      ],
    );
  }

  Widget _buildSliderRow({
    required String label,
    required double value,
    required bool enabled,
    required Function(double) onChanged,
  }) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.spaceBetween,
          children: [
            Text(
              label,
              style: TextStyle(
                fontSize: 20,
                color: enabled ? Colors.white : Colors.white30,
                fontWeight: FontWeight.w600,
              ),
            ),
            Text(
              '${(value * 100).toInt()}%',
              style: TextStyle(
                fontSize: 18,
                color: enabled ? const Color(0xFF00E6FF) : Colors.white30,
                fontWeight: FontWeight.w700,
              ),
            ),
          ],
        ),
        const SizedBox(height: 8),
        SliderTheme(
          data: SliderThemeData(
            activeTrackColor: const Color(0xFF00E6FF),
            inactiveTrackColor: Colors.white30,
            thumbColor: const Color(0xFF00E6FF),
            overlayColor: const Color(0xFF00E6FF).withOpacity(0.2),
            trackHeight: 4,
          ),
          child: Slider(
            value: value,
            onChanged: enabled ? onChanged : null,
            min: 0.0,
            max: 1.0,
          ),
        ),
      ],
    );
  }

  Widget _buildCloseButton() {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        onTap: widget.onClose,
        child: PersonaContainer(
          skew: -0.18,
          color: const Color(0xFF00E6FF).withOpacity(0.2),
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: 18, horizontal: 30),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                const Icon(
                  Icons.close,
                  color: Color(0xFF00E6FF),
                  size: 24,
                ),
                const SizedBox(width: 12),
                Text(
                  'CLOSE',
                  style: TextStyle(
                    fontSize: 22,
                    fontWeight: FontWeight.w900,
                    color: const Color(0xFF00E6FF),
                    letterSpacing: 2,
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  void _handleKeyEvent(KeyEvent event) {
    if (event is KeyDownEvent) {
      if (event.logicalKey == LogicalKeyboardKey.escape) {
        widget.onClose();
      }
    }
  }
}
