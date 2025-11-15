import 'package:flutter/material.dart';

class DialogWidget extends StatelessWidget {
  final String title;
  final String message;
  final String? confirmText;
  final String? cancelText;
  final VoidCallback? onConfirm;
  final VoidCallback? onCancel;
  final bool isDestructive;

  const DialogWidget({
    Key? key,
    required this.title,
    required this.message,
    this.confirmText = 'OK',
    this.cancelText = 'CANCEL',
    this.onConfirm,
    this.onCancel,
    this.isDestructive = false,
  }) : super(key: key);

  @override
  Widget build(BuildContext context) {
    return Dialog(
      backgroundColor: const Color(0xFF0A0E27),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(4)),
      child: Container(
        decoration: BoxDecoration(
          border: Border.all(color: Colors.cyan.withOpacity(0.3), width: 1),
          borderRadius: BorderRadius.circular(4),
        ),
        padding: const EdgeInsets.all(24),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(title, style: Theme.of(context).textTheme.titleMedium?.copyWith(color: Colors.cyan)),
            const SizedBox(height: 12),
            Text(message, style: Theme.of(context).textTheme.bodySmall),
            const SizedBox(height: 24),
            Row(
              mainAxisAlignment: MainAxisAlignment.end,
              children: [
                if (cancelText != null)
                  Padding(
                    padding: const EdgeInsets.only(right: 8),
                    child: ElevatedButton(
                      onPressed: () {
                        onCancel?.call();
                        Navigator.pop(context);
                      },
                      style: ElevatedButton.styleFrom(backgroundColor: Colors.grey.shade700),
                      child: Text(cancelText!),
                    ),
                  ),
                ElevatedButton(
                  onPressed: () {
                    onConfirm?.call();
                    Navigator.pop(context);
                  },
                  style: ElevatedButton.styleFrom(
                    backgroundColor: isDestructive ? Colors.red.shade700 : Colors.cyan,
                  ),
                  child: Text(confirmText!),
                ),
              ],
            ),
          ],
        ),
      ),
    );
  }
}
