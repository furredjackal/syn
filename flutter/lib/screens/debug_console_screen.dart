import 'package:flutter/material.dart';

class DebugConsoleScreen extends StatefulWidget {
  const DebugConsoleScreen({Key? key}) : super(key: key);

  @override
  State<DebugConsoleScreen> createState() => _DebugConsoleScreenState();
}

class _DebugConsoleScreenState extends State<DebugConsoleScreen> {
  final TextEditingController _commandController = TextEditingController();
  final List<String> _logs = [
    'Debug Console Initialized',
    'Ready for commands...'
  ];

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF0A0E27),
      appBar: AppBar(
        backgroundColor: Colors.black.withOpacity(0.5),
        title: Text(
          'DEBUG CONSOLE',
          style: Theme.of(context)
              .textTheme
              .titleMedium
              ?.copyWith(color: Colors.lime),
        ),
        leading: IconButton(
            icon: const Icon(Icons.arrow_back),
            onPressed: () => Navigator.pop(context)),
      ),
      body: Column(
        children: [
          Expanded(
            child: SingleChildScrollView(
              padding: const EdgeInsets.all(12),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  for (final log in _logs)
                    Text(
                      log,
                      style: Theme.of(context).textTheme.bodySmall?.copyWith(
                          color: Colors.lime, fontFamily: 'monospace'),
                    ),
                ],
              ),
            ),
          ),
          Container(
            color: Colors.black.withOpacity(0.5),
            padding: const EdgeInsets.all(12),
            child: Row(
              children: [
                Expanded(
                  child: TextField(
                    controller: _commandController,
                    style: const TextStyle(
                        color: Colors.lime, fontFamily: 'monospace'),
                    decoration: InputDecoration(
                      hintText: '> ',
                      hintStyle: TextStyle(color: Colors.lime.withOpacity(0.5)),
                      border: InputBorder.none,
                    ),
                  ),
                ),
                IconButton(
                  icon: const Icon(Icons.send, color: Colors.lime),
                  onPressed: _executeCommand,
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  void _executeCommand() {
    final command = _commandController.text.trim();
    if (command.isEmpty) return;

    setState(() {
      _logs.add('> $command');
      _logs.add(_processCommand(command));
    });
    _commandController.clear();
  }

  String _processCommand(String command) {
    final parts = command.split(' ');
    switch (parts[0].toLowerCase()) {
      case 'help':
        return 'Available commands: help, seed, heat <value>, mood <value>, add_memory <tag>, clear';
      case 'seed':
        return 'World seed: 12345 (deterministic)';
      case 'heat':
        return parts.length > 1
            ? 'Narrative heat set to ${parts[1]}'
            : 'Usage: heat <value>';
      case 'mood':
        return parts.length > 1
            ? 'Mood set to ${parts[1]}'
            : 'Usage: mood <value>';
      case 'add_memory':
        return parts.length > 1
            ? 'Memory added with tag: ${parts[1]}'
            : 'Usage: add_memory <tag>';
      case 'clear':
        _logs.clear();
        return '';
      default:
        return 'Unknown command: $command';
    }
  }

  @override
  void dispose() {
    _commandController.dispose();
    super.dispose();
  }
}
