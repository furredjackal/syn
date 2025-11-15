import 'package:flutter/material.dart';

class DropdownWidget extends StatefulWidget {
  final String label;
  final String value;
  final List<String> options;
  final ValueChanged<String> onChanged;

  const DropdownWidget({
    Key? key,
    required this.label,
    required this.value,
    required this.options,
    required this.onChanged,
  }) : super(key: key);

  @override
  State<DropdownWidget> createState() => _DropdownWidgetState();
}

class _DropdownWidgetState extends State<DropdownWidget> {
  late String _selectedValue;
  bool _isOpen = false;

  @override
  void initState() {
    super.initState();
    _selectedValue = widget.value;
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(widget.label, style: Theme.of(context).textTheme.bodyMedium),
        const SizedBox(height: 8),
        GestureDetector(
          onTap: () => setState(() => _isOpen = !_isOpen),
          child: Container(
            decoration: BoxDecoration(
              border: Border.all(color: Colors.cyan.withOpacity(0.5), width: 1),
              borderRadius: BorderRadius.circular(4),
              color: Colors.black.withOpacity(0.3),
            ),
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
            child: Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                Text(_selectedValue, style: Theme.of(context).textTheme.bodySmall?.copyWith(color: Colors.cyan)),
                Icon(_isOpen ? Icons.expand_less : Icons.expand_more, color: Colors.cyan),
              ],
            ),
          ),
        ),
        if (_isOpen)
          Container(
            decoration: BoxDecoration(
              border: Border.all(color: Colors.cyan.withOpacity(0.5), width: 1),
              borderRadius: BorderRadius.circular(4),
              color: Colors.black.withOpacity(0.5),
            ),
            margin: const EdgeInsets.only(top: 4),
            child: Column(
              children: widget.options.map((option) {
                return GestureDetector(
                  onTap: () {
                    setState(() {
                      _selectedValue = option;
                      _isOpen = false;
                    });
                    widget.onChanged(option);
                  },
                  child: Container(
                    padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
                    decoration: BoxDecoration(
                      color: _selectedValue == option ? Colors.cyan.withOpacity(0.1) : Colors.transparent,
                    ),
                    child: Text(
                      option,
                      style: Theme.of(context).textTheme.bodySmall?.copyWith(
                        color: _selectedValue == option ? Colors.cyan : Colors.grey,
                      ),
                    ),
                  ),
                );
              }).toList(),
            ),
          ),
      ],
    );
  }
}
