import 'package:flutter/material.dart';

import '../components/ui/syn_theme.dart';
import '../syn_game.dart';

Widget buildDebugConsoleOverlay(BuildContext context, SynGame game) {
  return _DebugConsoleOverlay(game: game);
}

class _DebugConsoleOverlay extends StatefulWidget {
  const _DebugConsoleOverlay({required this.game});

  final SynGame game;

  @override
  State<_DebugConsoleOverlay> createState() => _DebugConsoleOverlayState();
}

class _DebugConsoleOverlayState extends State<_DebugConsoleOverlay> {
  late final TextEditingController _controller;
  final List<String> _log = [];

  @override
  void initState() {
    super.initState();
    _controller = TextEditingController();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  void _submit() {
    final command = _controller.text.trim();
    if (command.isEmpty) return;
    setState(() {
      _log.insert(0, command);
      if (_log.length > 20) {
        _log.removeLast();
      }
    });
    widget.game.executeDebugCommand(command);
    _controller.clear();
  }

  void _close() {
    widget.game.overlays.remove('debug_console');
    widget.game.resumeEngine();
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: SynColors.bgOverlay,
      body: Center(
        child: Container(
          width: 640,
          height: 420,
          padding: const EdgeInsets.all(SynLayout.paddingLarge),
          decoration: BoxDecoration(
            color: SynColors.bgPanel,
            border: Border.all(color: SynColors.primaryCyan, width: 2),
          ),
          child: Column(
            children: [
              Row(
                children: [
                  const Expanded(
                    child: Text('DEBUG CONSOLE', style: SynTextStyles.h1Event),
                  ),
                  IconButton(
                    onPressed: _close,
                    icon: const Icon(Icons.close, color: SynColors.textPrimary),
                  ),
                ],
              ),
              const SizedBox(height: SynLayout.paddingSmall),
              Expanded(
                child: Container(
                  decoration: BoxDecoration(
                    color: const Color(0xFF0B0F18),
                    border: Border.all(
                      color: SynColors.primaryCyan.withValues(alpha: 0.4),
                    ),
                  ),
                  child: ListView.builder(
                    reverse: true,
                    itemCount: _log.length,
                    itemBuilder: (context, index) {
                      return Padding(
                        padding: const EdgeInsets.symmetric(
                          horizontal: SynLayout.paddingSmall,
                          vertical: 4,
                        ),
                        child: Text(
                          _log[index],
                          style: const TextStyle(color: SynColors.textPrimary),
                        ),
                      );
                    },
                  ),
                ),
              ),
              const SizedBox(height: SynLayout.paddingMedium),
              Row(
                children: [
                  Expanded(
                    child: TextField(
                      controller: _controller,
                      style: const TextStyle(color: SynColors.textPrimary),
                      decoration: const InputDecoration(
                        hintText: 'Enter a command...',
                        hintStyle: TextStyle(color: SynColors.textMuted),
                        filled: true,
                        fillColor: Color(0xFF0B0F18),
                        border: OutlineInputBorder(),
                      ),
                      onSubmitted: (_) => _submit(),
                    ),
                  ),
                  const SizedBox(width: SynLayout.paddingSmall),
                  ElevatedButton(
                    onPressed: _submit,
                    child: const Text('Run'),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}
