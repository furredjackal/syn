import 'package:flutter/material.dart';

class SaveSlot {
  final int slot;
  final String characterName;
  final int age;
  final DateTime savedAt;
  final bool isEmpty;

  SaveSlot({
    required this.slot,
    this.characterName = '',
    this.age = 0,
    required this.savedAt,
    this.isEmpty = true,
  });
}

class SaveLoadScreen extends StatefulWidget {
  const SaveLoadScreen({Key? key}) : super(key: key);

  @override
  State<SaveLoadScreen> createState() => _SaveLoadScreenState();
}

class _SaveLoadScreenState extends State<SaveLoadScreen> {
  late List<SaveSlot> saveSlots;
  int? selectedSlot;
  bool isLoading = false;

  @override
  void initState() {
    super.initState();
    saveSlots = List.generate(
      10,
      (index) => SaveSlot(
        slot: index + 1,
        savedAt: DateTime.now(),
      ),
    );
    saveSlots[0] = SaveSlot(
      slot: 1,
      characterName: 'Aria Nightwhisper',
      age: 28,
      savedAt: DateTime.now().subtract(const Duration(hours: 2)),
      isEmpty: false,
    );
    saveSlots[2] = SaveSlot(
      slot: 3,
      characterName: 'Marcus Stone',
      age: 45,
      savedAt: DateTime.now().subtract(const Duration(days: 1)),
      isEmpty: false,
    );
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: const Color(0xFF0A0E27),
      appBar: AppBar(
        backgroundColor: Colors.black.withOpacity(0.5),
        title: Text(
          'SAVE / LOAD',
          style: Theme.of(context)
              .textTheme
              .titleMedium
              ?.copyWith(color: const Color(0xFF00D9FF)),
        ),
        leading: IconButton(
            icon: const Icon(Icons.arrow_back),
            onPressed: () => Navigator.pop(context)),
      ),
      body: SingleChildScrollView(
        padding: const EdgeInsets.all(16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text('MANUAL SAVES',
                style: Theme.of(context)
                    .textTheme
                    .titleMedium
                    ?.copyWith(color: const Color(0xFF00D9FF))),
            const SizedBox(height: 12),
            GridView.builder(
              shrinkWrap: true,
              physics: const NeverScrollableScrollPhysics(),
              gridDelegate: const SliverGridDelegateWithFixedCrossAxisCount(
                crossAxisCount: 2,
                childAspectRatio: 2.5,
                crossAxisSpacing: 8,
                mainAxisSpacing: 8,
              ),
              itemCount: 10,
              itemBuilder: (context, index) =>
                  _buildSaveSlot(context, saveSlots[index]),
            ),
            const SizedBox(height: 24),
            Text('AUTO-SAVE',
                style: Theme.of(context)
                    .textTheme
                    .titleMedium
                    ?.copyWith(color: const Color(0xFF00D9FF))),
            const SizedBox(height: 12),
            _buildAutoSaveSlot(context),
            const SizedBox(height: 24),
            if (selectedSlot != null)
              Row(
                mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                children: [
                  ElevatedButton.icon(
                      onPressed: isLoading ? null : _loadGame,
                      icon: const Icon(Icons.upload),
                      label: const Text('LOAD')),
                  ElevatedButton.icon(
                      onPressed: isLoading ? null : _saveGame,
                      icon: const Icon(Icons.save),
                      label: const Text('SAVE')),
                  ElevatedButton.icon(
                    onPressed: _deleteSlot,
                    icon: const Icon(Icons.delete),
                    label: const Text('DELETE'),
                    style: ElevatedButton.styleFrom(
                        backgroundColor: Colors.red.shade700),
                  ),
                ],
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildSaveSlot(BuildContext context, SaveSlot slot) {
    final isSelected = selectedSlot == slot.slot;
    return GestureDetector(
      onTap: () => setState(() => selectedSlot = slot.slot),
      child: Container(
        decoration: BoxDecoration(
          border: Border.all(
              color: isSelected ? const Color(0xFF00D9FF) : Colors.white24,
              width: isSelected ? 2 : 1),
          borderRadius: BorderRadius.circular(4),
          color: Colors.black.withOpacity(0.3),
        ),
        padding: const EdgeInsets.all(8),
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              slot.isEmpty ? 'EMPTY - Slot ${slot.slot}' : slot.characterName,
              style: Theme.of(context).textTheme.bodySmall?.copyWith(
                  color: slot.isEmpty ? Colors.grey : const Color(0xFF00D9FF)),
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
            ),
            if (!slot.isEmpty) ...[
              Text('Age ${slot.age}',
                  style: Theme.of(context)
                      .textTheme
                      .labelSmall
                      ?.copyWith(color: Colors.grey)),
              Text(
                '${slot.savedAt.month}/${slot.savedAt.day} ${slot.savedAt.hour}:${slot.savedAt.minute.toString().padLeft(2, '0')}',
                style: Theme.of(context)
                    .textTheme
                    .labelSmall
                    ?.copyWith(color: Colors.grey.shade600),
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildAutoSaveSlot(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        border: Border.all(color: Colors.amber.withOpacity(0.5), width: 1),
        borderRadius: BorderRadius.circular(4),
        color: Colors.black.withOpacity(0.3),
      ),
      padding: const EdgeInsets.all(12),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text('Aria Nightwhisper',
              style: Theme.of(context)
                  .textTheme
                  .bodySmall
                  ?.copyWith(color: Colors.amber)),
          const SizedBox(height: 4),
          Text('Age 28 • Updated 5 minutes ago',
              style: Theme.of(context)
                  .textTheme
                  .labelSmall
                  ?.copyWith(color: Colors.grey)),
        ],
      ),
    );
  }

  Future<void> _saveGame() async {
    setState(() => isLoading = true);
    await Future.delayed(const Duration(milliseconds: 500));
    if (mounted) {
      ScaffoldMessenger.of(context)
          .showSnackBar(const SnackBar(content: Text('Game saved')));
      setState(() => isLoading = false);
    }
  }

  Future<void> _loadGame() async {
    setState(() => isLoading = true);
    await Future.delayed(const Duration(milliseconds: 500));
    if (mounted) {
      ScaffoldMessenger.of(context)
          .showSnackBar(const SnackBar(content: Text('Game loaded')));
      setState(() => isLoading = false);
      Navigator.pushNamedAndRemoveUntil(context, '/game', (route) => false);
    }
  }

  void _deleteSlot() {
    showDialog(
      context: context,
      builder: (context) => AlertDialog(
        backgroundColor: const Color(0xFF0A0E27),
        title: const Text('DELETE SAVE?'),
        content: const Text('This action cannot be undone.'),
        actions: [
          TextButton(
              onPressed: () => Navigator.pop(context),
              child: const Text('CANCEL')),
          TextButton(
            onPressed: () {
              setState(() {
                saveSlots[selectedSlot! - 1] =
                    SaveSlot(slot: selectedSlot!, savedAt: DateTime.now());
                selectedSlot = null;
              });
              Navigator.pop(context);
            },
            child: const Text('DELETE', style: TextStyle(color: Colors.red)),
          ),
        ],
      ),
    );
  }
}
