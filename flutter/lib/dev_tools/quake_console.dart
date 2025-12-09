import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

/// Quake-style drop-down debug console for runtime command execution.
///
/// Inspired by the iconic Quake/Source engine developer console,
/// this widget provides a terminal-like interface for executing commands
/// and viewing debug logs at runtime.
///
/// Features:
/// - Terminal-style green monospace text on black background
/// - Command history with up/down arrow navigation
/// - Scrollable log viewer
/// - Command auto-completion support (future)
class QuakeConsole extends StatefulWidget {
  /// Callback triggered when a command is submitted
  final Function(String) onCommand;

  const QuakeConsole({
    super.key,
    required this.onCommand,
  });

  @override
  State<QuakeConsole> createState() => _QuakeConsoleState();
}

class _QuakeConsoleState extends State<QuakeConsole> {
  final TextEditingController _controller = TextEditingController();
  final FocusNode _focusNode = FocusNode();
  final ScrollController _scrollController = ScrollController();
  
  final List<String> _logs = [
    '> SYN Console v0.1.0 - Type "help" for commands',
    '> Loaded Rust bridge successfully',
    '> Ready for input...',
  ];
  
  final List<String> _commandHistory = [];
  int _historyIndex = -1;

  @override
  void initState() {
    super.initState();
    // Auto-focus the text field when console opens
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _focusNode.requestFocus();
    });
  }

  @override
  void dispose() {
    _controller.dispose();
    _focusNode.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  void _submitCommand() {
    final command = _controller.text.trim();
    if (command.isEmpty) return;

    // Add to logs
    setState(() {
      _logs.add('> $command');
      _commandHistory.insert(0, command);
      _historyIndex = -1;
    });

    // Clear input
    _controller.clear();

    // Execute command callback
    widget.onCommand(command);

    // Scroll to bottom
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (_scrollController.hasClients) {
        _scrollController.animateTo(
          _scrollController.position.maxScrollExtent,
          duration: const Duration(milliseconds: 200),
          curve: Curves.easeOut,
        );
      }
    });
  }

  void _navigateHistory(bool up) {
    if (_commandHistory.isEmpty) return;

    setState(() {
      if (up) {
        // Navigate up (older commands)
        if (_historyIndex < _commandHistory.length - 1) {
          _historyIndex++;
          _controller.text = _commandHistory[_historyIndex];
          _controller.selection = TextSelection.fromPosition(
            TextPosition(offset: _controller.text.length),
          );
        }
      } else {
        // Navigate down (newer commands)
        if (_historyIndex > 0) {
          _historyIndex--;
          _controller.text = _commandHistory[_historyIndex];
          _controller.selection = TextSelection.fromPosition(
            TextPosition(offset: _controller.text.length),
          );
        } else if (_historyIndex == 0) {
          _historyIndex = -1;
          _controller.clear();
        }
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      height: MediaQuery.of(context).size.height * 0.5,
      decoration: BoxDecoration(
        color: Colors.black.withValues(alpha: 0.95),
        border: Border(
          bottom: BorderSide(color: Colors.green, width: 2),
        ),
        boxShadow: [
          BoxShadow(
            color: Colors.green.withValues(alpha: 0.3),
            blurRadius: 20,
            offset: const Offset(0, 10),
          ),
        ],
      ),
      child: Column(
        children: [
          // Header
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
            decoration: BoxDecoration(
              color: Colors.green.withValues(alpha: 0.15),
              border: Border(
                bottom: BorderSide(color: Colors.green, width: 1),
              ),
            ),
            child: Row(
              children: [
                Icon(Icons.terminal, color: Colors.green, size: 20),
                const SizedBox(width: 12),
                Text(
                  'QUAKE CONSOLE',
                  style: TextStyle(
                    color: Colors.green,
                    fontSize: 14,
                    fontWeight: FontWeight.bold,
                    fontFamily: 'monospace',
                    letterSpacing: 2,
                  ),
                ),
                const Spacer(),
                Text(
                  'Press ~ or ESC to close',
                  style: TextStyle(
                    color: Colors.green.withValues(alpha: 0.6),
                    fontSize: 11,
                    fontFamily: 'monospace',
                  ),
                ),
              ],
            ),
          ),

          // Log Viewer
          Expanded(
            child: ListView.builder(
              controller: _scrollController,
              padding: const EdgeInsets.all(16),
              itemCount: _logs.length,
              itemBuilder: (context, index) {
                return Padding(
                  padding: const EdgeInsets.only(bottom: 4),
                  child: Text(
                    _logs[index],
                    style: TextStyle(
                      color: Colors.green,
                      fontSize: 13,
                      fontFamily: 'monospace',
                      height: 1.4,
                    ),
                  ),
                );
              },
            ),
          ),

          // Command Input
          Container(
            padding: const EdgeInsets.all(16),
            decoration: BoxDecoration(
              color: Colors.black,
              border: Border(
                top: BorderSide(color: Colors.green, width: 1),
              ),
            ),
            child: Row(
              children: [
                Text(
                  '> ',
                  style: TextStyle(
                    color: Colors.green,
                    fontSize: 14,
                    fontFamily: 'monospace',
                    fontWeight: FontWeight.bold,
                  ),
                ),
                Expanded(
                  child: KeyboardListener(
                    focusNode: FocusNode(),
                    onKeyEvent: (event) {
                      if (event is KeyDownEvent) {
                        if (event.logicalKey == LogicalKeyboardKey.arrowUp) {
                          _navigateHistory(true);
                        } else if (event.logicalKey == LogicalKeyboardKey.arrowDown) {
                          _navigateHistory(false);
                        }
                      }
                    },
                    child: TextField(
                      controller: _controller,
                      focusNode: _focusNode,
                      style: TextStyle(
                        color: Colors.green,
                        fontSize: 14,
                        fontFamily: 'monospace',
                      ),
                      decoration: InputDecoration(
                        border: InputBorder.none,
                        hintText: 'Enter command...',
                        hintStyle: TextStyle(
                          color: Colors.green.withValues(alpha: 0.4),
                          fontFamily: 'monospace',
                        ),
                      ),
                      cursorColor: Colors.green,
                      onSubmitted: (_) => _submitCommand(),
                    ),
                  ),
                ),
                IconButton(
                  icon: Icon(Icons.send, color: Colors.green, size: 20),
                  onPressed: _submitCommand,
                  tooltip: 'Execute Command',
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }

  /// Public method to add a log entry from outside the widget
  void addLog(String message) {
    setState(() {
      _logs.add(message);
    });
    
    // Auto-scroll to bottom
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (_scrollController.hasClients) {
        _scrollController.animateTo(
          _scrollController.position.maxScrollExtent,
          duration: const Duration(milliseconds: 200),
          curve: Curves.easeOut,
        );
      }
    });
  }
}
