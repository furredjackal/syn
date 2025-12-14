import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_animate/flutter_animate.dart';
import '../theme/syn_theme.dart';
import 'syn_container.dart';

/// Keyboard shortcuts overlay.
///
/// Shows all available keybinds in a styled modal.
/// Triggered by '?' key during gameplay.
class KeyboardShortcutsOverlay extends StatelessWidget {
  final VoidCallback onClose;

  const KeyboardShortcutsOverlay({
    super.key,
    required this.onClose,
  });

  @override
  Widget build(BuildContext context) {
    return Material(
      color: Colors.black.withOpacity(0.85),
      child: KeyboardListener(
        focusNode: FocusNode()..requestFocus(),
        onKeyEvent: (event) {
          if (event is KeyDownEvent &&
              (event.logicalKey == LogicalKeyboardKey.escape ||
               event.logicalKey == LogicalKeyboardKey.question ||
               event.logicalKey == LogicalKeyboardKey.slash)) {
            onClose();
          }
        },
        child: GestureDetector(
          onTap: onClose,
          behavior: HitTestBehavior.opaque,
          child: Center(
            child: GestureDetector(
              onTap: () {}, // Prevent closing when tapping inside
              child: SizedBox(
                width: 600,
                child: SynContainer(
                  enableHover: false,
                  child: Padding(
                    padding: const EdgeInsets.all(32),
                    child: Column(
                      mainAxisSize: MainAxisSize.min,
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        // Header
                        Row(
                          mainAxisAlignment: MainAxisAlignment.spaceBetween,
                          children: [
                            Row(
                              children: [
                                Icon(
                                  Icons.keyboard,
                                  color: SynTheme.accent,
                                  size: 24,
                                ),
                                const SizedBox(width: 12),
                                Text(
                                  'KEYBOARD SHORTCUTS',
                                  style: SynTheme.headline(color: SynTheme.accent),
                                ),
                              ],
                            ),
                            IconButton(
                              icon: Icon(
                                Icons.close,
                                color: SynTheme.textMuted,
                              ),
                              onPressed: onClose,
                            ),
                          ],
                        ),
                        const SizedBox(height: 24),

                        // Divider
                        Container(
                          height: 1,
                          decoration: BoxDecoration(
                            gradient: LinearGradient(
                              colors: [
                                SynTheme.accent.withOpacity(0),
                                SynTheme.accent.withOpacity(0.5),
                                SynTheme.accent.withOpacity(0),
                              ],
                            ),
                          ),
                        ),
                        const SizedBox(height: 24),

                        // Shortcut sections
                        _buildSection('SIMULATION', [
                          _Shortcut('Space', 'Pause / Resume'),
                          _Shortcut('→', 'Advance 1 Hour'),
                          _Shortcut('Shift + →', 'Advance 1 Day'),
                          _Shortcut('1 / 2 / 3', 'Set Speed (1x / 2x / 4x)'),
                        ]),
                        const SizedBox(height: 20),

                        _buildSection('PANELS', [
                          _Shortcut('S', 'Stats Panel'),
                          _Shortcut('I', 'Inventory'),
                          _Shortcut('J', 'Journal'),
                          _Shortcut('M', 'World Map'),
                          _Shortcut('R', 'Relationships'),
                          _Shortcut('C', 'Calendar'),
                        ]),
                        const SizedBox(height: 20),

                        _buildSection('GENERAL', [
                          _Shortcut('Esc', 'Close Panel / Pause Menu'),
                          _Shortcut('?', 'Show Shortcuts'),
                          _Shortcut('`', 'Debug Console'),
                          _Shortcut('F11', 'Toggle Inspector'),
                          _Shortcut('F5', 'Quick Save'),
                          _Shortcut('F9', 'Quick Load'),
                        ]),
                        const SizedBox(height: 24),

                        // Footer hint
                        Center(
                          child: Text(
                            'Press ESC or ? to close',
                            style: SynTheme.caption(color: SynTheme.textMuted),
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              )
                  .animate()
                  .scale(
                    begin: const Offset(0.9, 0.9),
                    end: const Offset(1.0, 1.0),
                    duration: SynTheme.normal,
                    curve: SynTheme.snapIn,
                  )
                  .fadeIn(duration: SynTheme.fast),
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildSection(String title, List<_Shortcut> shortcuts) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          title,
          style: SynTheme.label(color: SynTheme.textMuted),
        ),
        const SizedBox(height: 12),
        ...shortcuts.map((s) => Padding(
          padding: const EdgeInsets.only(bottom: 8),
          child: _buildShortcutRow(s.key, s.action),
        )),
      ],
    );
  }

  Widget _buildShortcutRow(String key, String action) {
    return Row(
      children: [
        Container(
          constraints: const BoxConstraints(minWidth: 80),
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
          decoration: BoxDecoration(
            border: Border.all(color: SynTheme.accent.withOpacity(0.5), width: 1),
            color: SynTheme.bgCard,
          ),
          child: Text(
            key,
            style: SynTheme.label(color: SynTheme.accent),
            textAlign: TextAlign.center,
          ),
        ),
        const SizedBox(width: 16),
        Expanded(
          child: Text(
            action,
            style: SynTheme.body(color: SynTheme.textSecondary),
          ),
        ),
      ],
    );
  }
}

class _Shortcut {
  final String key;
  final String action;

  const _Shortcut(this.key, this.action);
}
