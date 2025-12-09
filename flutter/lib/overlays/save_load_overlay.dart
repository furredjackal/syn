import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_animate/flutter_animate.dart';
import '../ui/widgets/persona_container.dart';

/// Save/Load Overlay - modal dialog for managing save files
/// 
/// Props:
/// - onClose: Callback to close the overlay
/// - onSave: Callback when saving to a slot
/// - onLoad: Callback when loading from a slot
/// - saveSlots: List of save slot data
/// - mode: 'save' or 'load' mode
class SaveLoadOverlay extends StatefulWidget {
  final VoidCallback onClose;
  final Function(int slotIndex, String slotName) onSave;
  final Function(int slotIndex) onLoad;
  final List<SaveSlotData> saveSlots;
  final String mode; // 'save' or 'load'

  const SaveLoadOverlay({
    super.key,
    required this.onClose,
    required this.onSave,
    required this.onLoad,
    required this.saveSlots,
    this.mode = 'load',
  });

  @override
  State<SaveLoadOverlay> createState() => _SaveLoadOverlayState();
}

class _SaveLoadOverlayState extends State<SaveLoadOverlay> {
  int _selectedIndex = 0;
  final TextEditingController _slotNameController = TextEditingController();

  @override
  void dispose() {
    _slotNameController.dispose();
    super.dispose();
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
            constraints: const BoxConstraints(maxWidth: 800, maxHeight: 700),
            child: PersonaContainer(
              skew: -0.15,
              color: Colors.black.withOpacity(0.95),
              child: Padding(
                padding: const EdgeInsets.all(40.0),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    _buildHeader(),
                    const SizedBox(height: 30),
                    Expanded(
                      child: _buildSlotList(),
                    ),
                    const SizedBox(height: 20),
                    _buildActions(),
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
          widget.mode.toUpperCase(),
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

  Widget _buildSlotList() {
    return ListView.builder(
      itemCount: widget.saveSlots.length,
      itemBuilder: (context, index) {
        return Padding(
          padding: const EdgeInsets.only(bottom: 15.0),
          child: _buildSlotItem(index),
        );
      },
    );
  }

  Widget _buildSlotItem(int index) {
    final slot = widget.saveSlots[index];
    final isSelected = _selectedIndex == index;

    return MouseRegion(
      cursor: SystemMouseCursors.click,
      onEnter: (_) => setState(() => _selectedIndex = index),
      child: GestureDetector(
        onTap: () => _handleSlotAction(index),
        child: PersonaContainer(
          skew: -0.12,
          color: isSelected
              ? const Color(0xFF00E6FF).withOpacity(0.2)
              : Colors.black.withOpacity(0.6),
          child: Padding(
            padding: const EdgeInsets.all(20.0),
            child: Row(
              children: [
                // Slot Number
                Container(
                  width: 50,
                  height: 50,
                  decoration: BoxDecoration(
                    color: isSelected
                        ? const Color(0xFF00E6FF)
                        : Colors.white30,
                    border: Border.all(
                      color: isSelected
                          ? const Color(0xFF00E6FF)
                          : Colors.white30,
                      width: 2,
                    ),
                  ),
                  child: Center(
                    child: Text(
                      '${index + 1}',
                      style: TextStyle(
                        fontSize: 24,
                        fontWeight: FontWeight.w900,
                        color: isSelected ? Colors.black : Colors.white,
                      ),
                    ),
                  ),
                ),
                const SizedBox(width: 20),
                // Slot Info
                Expanded(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        slot.isEmpty ? '[ EMPTY SLOT ]' : slot.name,
                        style: TextStyle(
                          fontSize: 20,
                          fontWeight: FontWeight.w700,
                          color: slot.isEmpty ? Colors.white30 : Colors.white,
                        ),
                      ),
                      if (!slot.isEmpty) ...[
                        const SizedBox(height: 5),
                        Text(
                          'Age: ${slot.age} • Day: ${slot.day}',
                          style: const TextStyle(
                            fontSize: 14,
                            color: Colors.white60,
                          ),
                        ),
                        Text(
                          slot.timestamp,
                          style: const TextStyle(
                            fontSize: 12,
                            color: Colors.white38,
                          ),
                        ),
                      ],
                    ],
                  ),
                ),
                // Action Icon
                Icon(
                  widget.mode == 'save' ? Icons.save : Icons.upload,
                  color: isSelected ? const Color(0xFF00E6FF) : Colors.white30,
                  size: 28,
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildActions() {
    return Row(
      children: [
        Expanded(
          child: _buildActionButton(
            label: 'CLOSE',
            icon: Icons.close,
            onTap: widget.onClose,
          ),
        ),
      ],
    );
  }

  Widget _buildActionButton({
    required String label,
    required IconData icon,
    required VoidCallback onTap,
  }) {
    return MouseRegion(
      cursor: SystemMouseCursors.click,
      child: GestureDetector(
        onTap: onTap,
        child: PersonaContainer(
          skew: -0.18,
          color: Colors.black.withOpacity(0.6),
          child: Padding(
            padding: const EdgeInsets.symmetric(vertical: 18, horizontal: 30),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Icon(icon, color: Colors.white70, size: 24),
                const SizedBox(width: 12),
                Text(
                  label,
                  style: const TextStyle(
                    fontSize: 22,
                    fontWeight: FontWeight.w900,
                    color: Colors.white,
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
      if (event.logicalKey == LogicalKeyboardKey.arrowUp ||
          event.logicalKey == LogicalKeyboardKey.keyW) {
        setState(() {
          _selectedIndex = (_selectedIndex - 1) % widget.saveSlots.length;
          if (_selectedIndex < 0) {
            _selectedIndex = widget.saveSlots.length - 1;
          }
        });
      } else if (event.logicalKey == LogicalKeyboardKey.arrowDown ||
          event.logicalKey == LogicalKeyboardKey.keyS) {
        setState(() {
          _selectedIndex = (_selectedIndex + 1) % widget.saveSlots.length;
        });
      } else if (event.logicalKey == LogicalKeyboardKey.enter ||
          event.logicalKey == LogicalKeyboardKey.space) {
        _handleSlotAction(_selectedIndex);
      } else if (event.logicalKey == LogicalKeyboardKey.escape) {
        widget.onClose();
      }
    }
  }

  void _handleSlotAction(int index) {
    final slot = widget.saveSlots[index];
    
    if (widget.mode == 'save') {
      // Save mode: prompt for name or use existing
      if (slot.isEmpty) {
        _showSaveDialog(index);
      } else {
        // Overwrite existing save
        widget.onSave(index, slot.name);
      }
    } else {
      // Load mode: can only load non-empty slots
      if (!slot.isEmpty) {
        widget.onLoad(index);
      }
    }
  }

  void _showSaveDialog(int index) {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        backgroundColor: Colors.black,
        shape: Border.all(color: const Color(0xFF00E6FF), width: 2),
        title: const Text(
          'NAME YOUR SAVE',
          style: TextStyle(
            color: Color(0xFF00E6FF),
            fontWeight: FontWeight.w900,
          ),
        ),
        content: TextField(
          controller: _slotNameController,
          style: const TextStyle(color: Colors.white),
          decoration: const InputDecoration(
            hintText: 'Enter save name...',
            hintStyle: TextStyle(color: Colors.white30),
            enabledBorder: UnderlineInputBorder(
              borderSide: BorderSide(color: Colors.white30),
            ),
            focusedBorder: UnderlineInputBorder(
              borderSide: BorderSide(color: Color(0xFF00E6FF)),
            ),
          ),
          autofocus: true,
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: const Text(
              'CANCEL',
              style: TextStyle(color: Colors.white70),
            ),
          ),
          TextButton(
            onPressed: () {
              final name = _slotNameController.text.trim();
              if (name.isNotEmpty) {
                widget.onSave(index, name);
                Navigator.of(context).pop();
              }
            },
            child: const Text(
              'SAVE',
              style: TextStyle(
                color: Color(0xFF00E6FF),
                fontWeight: FontWeight.w900,
              ),
            ),
          ),
        ],
      ),
    );
  }
}

/// Data model for save slots
class SaveSlotData {
  final String name;
  final int age;
  final int day;
  final String timestamp;
  final bool isEmpty;

  const SaveSlotData({
    required this.name,
    required this.age,
    required this.day,
    required this.timestamp,
    this.isEmpty = false,
  });

  factory SaveSlotData.empty() {
    return const SaveSlotData(
      name: '',
      age: 0,
      day: 0,
      timestamp: '',
      isEmpty: true,
    );
  }
}
