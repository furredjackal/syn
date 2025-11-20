import 'package:flutter/material.dart';

import '../syn_game.dart';
import '../components/ui/syn_theme.dart';

Widget buildTextInputOverlay(BuildContext context, SynGame game) {
  return _TextInputOverlay(game: game);
}

class _TextInputOverlay extends StatefulWidget {
  const _TextInputOverlay({required this.game});

  final SynGame game;

  @override
  State<_TextInputOverlay> createState() => _TextInputOverlayState();
}

class _TextInputOverlayState extends State<_TextInputOverlay> {
  late final TextEditingController _controller;

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
    widget.game.handleTextInput(_controller.text);
    widget.game.overlays.remove('text_input');
  }

  void _cancel() {
    widget.game.overlays.remove('text_input');
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: SynColors.bgOverlay,
      body: Center(
        child: Container(
          width: 420,
          padding: const EdgeInsets.all(SynLayout.paddingLarge),
          decoration: BoxDecoration(
            color: SynColors.bgPanel,
            border: Border.all(color: SynColors.primaryCyan, width: 2),
          ),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const Text(
                'INPUT',
                style: SynTextStyles.h1Event,
              ),
              const SizedBox(height: SynLayout.paddingMedium),
              TextField(
                controller: _controller,
                autofocus: true,
                style: const TextStyle(color: SynColors.textPrimary),
                decoration: const InputDecoration(
                  hintText: 'Type here...',
                  hintStyle: TextStyle(color: SynColors.textMuted),
                  filled: true,
                  fillColor: Color(0xFF0B0F18),
                  border: OutlineInputBorder(),
                ),
                onSubmitted: (_) => _submit(),
              ),
              const SizedBox(height: SynLayout.paddingMedium),
              Row(
                mainAxisAlignment: MainAxisAlignment.end,
                children: [
                  TextButton(
                    onPressed: _cancel,
                    child: const Text('Cancel'),
                  ),
                  const SizedBox(width: SynLayout.paddingSmall),
                  ElevatedButton(
                    onPressed: _submit,
                    child: const Text('Submit'),
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
