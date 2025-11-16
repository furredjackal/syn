import 'package:flutter/material.dart';

class TextInputWidget extends StatefulWidget {
  final String label;
  final String initialValue;
  final ValueChanged<String> onChanged;
  final int maxLines;
  final String? hint;
  final TextInputType inputType;

  const TextInputWidget({
    Key? key,
    required this.label,
    this.initialValue = '',
    required this.onChanged,
    this.maxLines = 1,
    this.hint,
    this.inputType = TextInputType.text,
  }) : super(key: key);

  @override
  State<TextInputWidget> createState() => _TextInputWidgetState();
}

class _TextInputWidgetState extends State<TextInputWidget> {
  late TextEditingController _controller;
  @override
  void initState() {
    super.initState();
    _controller = TextEditingController(text: widget.initialValue);
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(widget.label, style: Theme.of(context).textTheme.bodyMedium),
        const SizedBox(height: 8),
        TextField(
          controller: _controller,
          keyboardType: widget.inputType,
          maxLines: widget.maxLines,
          onChanged: widget.onChanged,
          style: Theme.of(context).textTheme.bodySmall?.copyWith(color: Colors.white),
          decoration: InputDecoration(
            hintText: widget.hint,
            hintStyle: Theme.of(context).textTheme.bodySmall?.copyWith(color: Colors.grey),
            contentPadding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
            border: OutlineInputBorder(
              borderRadius: BorderRadius.circular(4),
              borderSide: BorderSide(color: Colors.cyan.withOpacity(0.5), width: 1),
            ),
            focusedBorder: OutlineInputBorder(
              borderRadius: BorderRadius.circular(4),
              borderSide: const BorderSide(color: Colors.cyan, width: 1.5),
            ),
            filled: true,
            fillColor: Colors.black.withOpacity(0.3),
          ),
        ),
      ],
    );
  }
}
