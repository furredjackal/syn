import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

/// Controller for managing QuakeConsole state and logs.
///
/// Holds the log buffer and provides methods to add logs, clear logs,
/// and optionally customize the prompt.
class QuakeConsoleController extends ChangeNotifier {
  final List<String> _logs = [];
  String _prompt = '>';
  static const int _maxLogLines = 1000;

  /// Current list of log entries
  List<String> get logs => List.unmodifiable(_logs);

  /// Current prompt symbol
  String get prompt => _prompt;

  QuakeConsoleController() {
    // Default startup logs
    _logs.addAll([
      '> SYN Console v0.1.0 - Type "help" for commands',
      '> Loaded Rust bridge successfully',
      '> Ready for input...',
    ]);
  }

  /// Add a log entry to the console with optional log level
  void addLog(String message, {String? level}) {
    final prefix = level != null ? '[$level] ' : '';
    _logs.add('$prefix$message');
    
    // Cap at max lines, drop oldest
    while (_logs.length > _maxLogLines) {
      _logs.removeAt(0);
    }
    
    notifyListeners();
  }

  /// Add info log
  void info(String message) => addLog(message, level: 'INFO');

  /// Add warning log
  void warn(String message) => addLog(message, level: 'WARN');

  /// Add error log
  void error(String message) => addLog(message, level: 'ERR ');

  /// Clear all logs
  void clear() {
    _logs.clear();
    _logs.add('> Console cleared');
    notifyListeners();
  }

  /// Set custom prompt symbol (optional)
  void setPrompt(String prompt) {
    _prompt = prompt;
    notifyListeners();
  }
}

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
  /// Controller for managing console state and logs
  final QuakeConsoleController controller;

  /// Callback triggered when a command is submitted
  final Function(String) onCommand;

  /// Callback triggered when console should close (Escape, ~, etc.)
  final VoidCallback onClose;

  const QuakeConsole({
    super.key,
    required this.controller,
    required this.onCommand,
    required this.onClose,
  });

  @override
  State<QuakeConsole> createState() => _QuakeConsoleState();
}

class _QuakeConsoleState extends State<QuakeConsole> {
  final TextEditingController _textController = TextEditingController();
  final FocusNode _focusNode = FocusNode();
  final ScrollController _scrollController = ScrollController();
  
  // History stored oldest->newest
  final List<String> _commandHistory = [];
  int _historyIndex = -1;
  String _draftInput = '';

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
    _textController.dispose();
    _focusNode.dispose();
    _scrollController.dispose();
    super.dispose();
  }

  void _submitCommand() {
    final command = _textController.text.trim();
    if (command.isEmpty) return;

    // Add to logs via controller
    widget.controller.addLog('${widget.controller.prompt} $command');

    // Add to history (oldest->newest)
    setState(() {
      _commandHistory.add(command);
      _historyIndex = _commandHistory.length; // Reset to "after newest"
      _draftInput = ''; // Clear draft
    });

    // Clear input
    _textController.clear();

    // Execute command callback
    widget.onCommand(command);

    // Scroll to bottom
    _scrollToBottom();
  }

  void _scrollToBottom() {
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
        // Save draft on first press from fresh state
        if (_historyIndex == _commandHistory.length) {
          _draftInput = _textController.text;
        }

        // Navigate backward (toward older)
        if (_historyIndex > 0) {
          _historyIndex--;
          _textController.text = _commandHistory[_historyIndex];
          _textController.selection = TextSelection.fromPosition(
            TextPosition(offset: _textController.text.length),
          );
        }
      } else {
        // Navigate forward (toward newer)
        if (_historyIndex < _commandHistory.length - 1) {
          _historyIndex++;
          _textController.text = _commandHistory[_historyIndex];
          _textController.selection = TextSelection.fromPosition(
            TextPosition(offset: _textController.text.length),
          );
        } else if (_historyIndex < _commandHistory.length) {
          // Restore draft when going past newest
          _historyIndex = _commandHistory.length;
          _textController.text = _draftInput;
          _textController.selection = TextSelection.fromPosition(
            TextPosition(offset: _textController.text.length),
          );
        }
      }
    });
  }

  void _clearLogs() {
    widget.controller.clear();
    _scrollToBottom();
  }

  void _scrollPage(bool up) {
    if (!_scrollController.hasClients) return;

    final viewportHeight = _scrollController.position.viewportDimension;
    final targetOffset = up
        ? (_scrollController.offset - viewportHeight).clamp(0.0, _scrollController.position.maxScrollExtent)
        : (_scrollController.offset + viewportHeight).clamp(0.0, _scrollController.position.maxScrollExtent);

    _scrollController.animateTo(
      targetOffset,
      duration: const Duration(milliseconds: 300),
      curve: Curves.easeInOut,
    );
  }

  bool _canNavigateHistory() {
    // Allow history navigation only if cursor is at the end of text
    final cursorAtEnd = _textController.selection.baseOffset == _textController.text.length;
    return cursorAtEnd || _textController.text.isEmpty;
  }

  /// Build a log entry with rich text formatting for log levels and command highlighting
  Widget _buildLogEntry(String log) {
    // Define colors for log levels
    const infoColor = Colors.cyanAccent;
    const warnColor = Colors.amber;
    const errorColor = Colors.redAccent;
    const commandColor = Colors.yellowAccent;

    final baseStyle = TextStyle(
      fontSize: 13,
      fontFamily: 'monospace',
      height: 1.4,
    );

    // Parse log level prefix
    Color textColor = infoColor;
    String displayText = log;

    if (log.startsWith('[INFO]')) {
      textColor = infoColor;
    } else if (log.startsWith('[WARN]')) {
      textColor = warnColor;
    } else if (log.startsWith('[ERR ]')) {
      textColor = errorColor;
    }

    // Check if this is a command line (starts with >)
    if (log.startsWith('>')) {
      final parts = log.substring(1).trim().split(RegExp(r'\s+'));
      if (parts.isNotEmpty) {
        final command = parts[0];
        final rest = parts.skip(1).join(' ');

        return Text.rich(
          TextSpan(
            children: [
              TextSpan(
                text: '> ',
                style: baseStyle.copyWith(color: infoColor, fontWeight: FontWeight.bold),
              ),
              TextSpan(
                text: command,
                style: baseStyle.copyWith(color: commandColor, fontWeight: FontWeight.bold),
              ),
              if (rest.isNotEmpty)
                TextSpan(
                  text: ' $rest',
                  style: baseStyle.copyWith(color: infoColor),
                ),
            ],
          ),
        );
      }
    }

    // Check for log level prefixes and colorize them
    if (log.startsWith('[INFO]') || log.startsWith('[WARN]') || log.startsWith('[ERR ]')) {
      final levelEnd = log.indexOf(']') + 1;
      final level = log.substring(0, levelEnd);
      final message = log.substring(levelEnd);

      return Text.rich(
        TextSpan(
          children: [
            TextSpan(
              text: level,
              style: baseStyle.copyWith(color: textColor, fontWeight: FontWeight.bold),
            ),
            TextSpan(
              text: message,
              style: baseStyle.copyWith(color: infoColor),
            ),
          ],
        ),
      );
    }

    // Default rendering
    return Text(
      displayText,
      style: baseStyle.copyWith(color: textColor),
    );
  }

  @override
  Widget build(BuildContext context) {
    return Focus(
      autofocus: true,
      child: Shortcuts(
        shortcuts: {
          LogicalKeySet(LogicalKeyboardKey.arrowUp): const _HistoryUpIntent(),
          LogicalKeySet(LogicalKeyboardKey.arrowDown): const _HistoryDownIntent(),
          LogicalKeySet(LogicalKeyboardKey.escape): const _CloseIntent(),
          LogicalKeySet(LogicalKeyboardKey.backquote): const _CloseIntent(),
          LogicalKeySet(LogicalKeyboardKey.control, LogicalKeyboardKey.keyL): const _ClearIntent(),
          LogicalKeySet(LogicalKeyboardKey.pageUp): const _ScrollPageUpIntent(),
          LogicalKeySet(LogicalKeyboardKey.pageDown): const _ScrollPageDownIntent(),
        },
        child: Actions(
          actions: {
            _HistoryUpIntent: CallbackAction<_HistoryUpIntent>(
              onInvoke: (_) {
                if (_canNavigateHistory()) {
                  _navigateHistory(true);
                }
                return null;
              },
            ),
            _HistoryDownIntent: CallbackAction<_HistoryDownIntent>(
              onInvoke: (_) {
                if (_canNavigateHistory()) {
                  _navigateHistory(false);
                }
                return null;
              },
            ),
            _CloseIntent: CallbackAction<_CloseIntent>(
              onInvoke: (_) {
                widget.onClose();
                return null;
              },
            ),
            _ClearIntent: CallbackAction<_ClearIntent>(
              onInvoke: (_) {
                _clearLogs();
                return null;
              },
            ),
            _ScrollPageUpIntent: CallbackAction<_ScrollPageUpIntent>(
              onInvoke: (_) {
                _scrollPage(true);
                return null;
              },
            ),
            _ScrollPageDownIntent: CallbackAction<_ScrollPageDownIntent>(
              onInvoke: (_) {
                _scrollPage(false);
                return null;
              },
            ),
          },
          child: AnimatedBuilder(
            animation: widget.controller,
            builder: (context, child) {
              return Container(
                height: MediaQuery.of(context).size.height * 0.5,
                decoration: BoxDecoration(
                  color: Colors.black.withValues(alpha: 0.95),
                  border: Border(
                    bottom: BorderSide(color: Colors.cyanAccent, width: 2),
                  ),
                  boxShadow: [
                    BoxShadow(
                      color: Colors.cyanAccent.withValues(alpha: 0.3),
                      blurRadius: 20,
                      offset: const Offset(0, 10),
                    ),
                  ],
                ),
                child: Column(
                  children: [
                    // Header with diagonal slash edge
                    ClipPath(
                      clipper: _DiagonalSlashClipper(),
                      child: Container(
                        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
                        decoration: BoxDecoration(
                          color: Colors.cyanAccent.withValues(alpha: 0.15),
                          border: Border(
                            bottom: BorderSide(color: Colors.cyanAccent, width: 2),
                          ),
                        ),
                        child: Row(
                          children: [
                            Icon(Icons.terminal, color: Colors.cyanAccent, size: 20),
                            const SizedBox(width: 12),
                            Text(
                              'SYN CONSOLE',
                              style: TextStyle(
                                color: Colors.cyanAccent,
                                fontSize: 14,
                                fontWeight: FontWeight.bold,
                                fontFamily: 'monospace',
                                letterSpacing: 2,
                              ),
                            ),
                            const Spacer(),
                            Text(
                              'ESC/~ close | Ctrl+L clear | PgUp/PgDn scroll',
                              style: TextStyle(
                                color: Colors.cyanAccent.withValues(alpha: 0.6),
                                fontSize: 11,
                                fontFamily: 'monospace',
                              ),
                            ),
                          ],
                        ),
                      ),
                    ),

                    // Log Viewer
                    Expanded(
                      child: ListView.builder(
                        controller: _scrollController,
                        padding: const EdgeInsets.all(16),
                        itemCount: widget.controller.logs.length,
                        itemBuilder: (context, index) {
                          return Padding(
                            padding: const EdgeInsets.only(bottom: 4),
                            child: _buildLogEntry(widget.controller.logs[index]),
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
                          top: BorderSide(color: Colors.cyanAccent, width: 1),
                        ),
                      ),
                      child: Row(
                        children: [
                          Text(
                            '${widget.controller.prompt} ',
                            style: TextStyle(
                              color: Colors.cyanAccent,
                              fontSize: 14,
                              fontFamily: 'monospace',
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                          Expanded(
                            child: TextField(
                              controller: _textController,
                              focusNode: _focusNode,
                              style: TextStyle(
                                color: Colors.cyanAccent,
                                fontSize: 14,
                                fontFamily: 'monospace',
                              ),
                              decoration: InputDecoration(
                                border: InputBorder.none,
                                hintText: 'Enter command...',
                                hintStyle: TextStyle(
                                  color: Colors.cyanAccent.withValues(alpha: 0.4),
                                  fontFamily: 'monospace',
                                ),
                              ),
                              cursorColor: Colors.cyanAccent,
                              onSubmitted: (_) => _submitCommand(),
                            ),
                          ),
                          IconButton(
                            icon: Icon(Icons.send, color: Colors.cyanAccent, size: 20),
                            onPressed: _submitCommand,
                            tooltip: 'Execute Command (Enter)',
                          ),
                        ],
                      ),
                    ),
                  ],
                ),
              );
            },
          ),
        ),
      ),
    );
  }
}

// Custom clipper for diagonal slash header edge
class _DiagonalSlashClipper extends CustomClipper<Path> {
  @override
  Path getClip(Size size) {
    final path = Path();
    const slashHeight = 8.0;

    path.lineTo(0, size.height - slashHeight);
    path.lineTo(size.width, size.height);
    path.lineTo(size.width, 0);
    path.close();

    return path;
  }

  @override
  bool shouldReclip(covariant CustomClipper<Path> oldClipper) => false;
}

// Intent classes for keyboard shortcuts
class _HistoryUpIntent extends Intent {
  const _HistoryUpIntent();
}

class _HistoryDownIntent extends Intent {
  const _HistoryDownIntent();
}

class _CloseIntent extends Intent {
  const _CloseIntent();
}

class _ClearIntent extends Intent {
  const _ClearIntent();
}

class _ScrollPageUpIntent extends Intent {
  const _ScrollPageUpIntent();
}

class _ScrollPageDownIntent extends Intent {
  const _ScrollPageDownIntent();
}
