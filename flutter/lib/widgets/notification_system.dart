import 'package:flutter/material.dart';

enum NotificationType { success, warning, error, info }

class Notification {
  final String message;
  final NotificationType type;
  final Duration duration;

  Notification({
    required this.message,
    this.type = NotificationType.info,
    this.duration = const Duration(seconds: 3),
  });
}

class NotificationOverlay extends StatefulWidget {
  final List<Notification> notifications;

  const NotificationOverlay({
    Key? key,
    required this.notifications,
  }) : super(key: key);

  @override
  State<NotificationOverlay> createState() => _NotificationOverlayState();
}

class _NotificationOverlayState extends State<NotificationOverlay> with SingleTickerProviderStateMixin {
  late AnimationController _animationController;
  late Animation<Offset> _slideAnimation;

  @override
  void initState() {
    super.initState();
    _animationController = AnimationController(
      duration: const Duration(milliseconds: 300),
      vsync: this,
    );
    _slideAnimation = Tween<Offset>(begin: const Offset(1, 0), end: Offset.zero).animate(_animationController);
    _animationController.forward();
  }

  @override
  void dispose() {
    _animationController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return SlideTransition(
      position: _slideAnimation,
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.end,
          children: [
            for (final notification in widget.notifications)
              Padding(
                padding: const EdgeInsets.symmetric(vertical: 8),
                child: _buildNotificationCard(notification),
              ),
          ],
        ),
      ),
    );
  }

  Widget _buildNotificationCard(Notification notif) {
    final color = _getColorForType(notif.type);
    const icon = Icons.info_outline;

    return Container(
      constraints: const BoxConstraints(maxWidth: 300),
      decoration: BoxDecoration(
        border: Border.all(color: color, width: 1),
        borderRadius: BorderRadius.circular(4),
        color: Colors.black.withOpacity(0.8),
      ),
      padding: const EdgeInsets.all(12),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, color: color, size: 20),
          const SizedBox(width: 8),
          Flexible(
            child: Text(
              notif.message,
              style: Theme.of(context).textTheme.bodySmall?.copyWith(color: color),
              maxLines: 2,
              overflow: TextOverflow.ellipsis,
            ),
          ),
        ],
      ),
    );
  }

  Color _getColorForType(NotificationType type) {
    switch (type) {
      case NotificationType.success:
        return Colors.green;
      case NotificationType.warning:
        return Colors.amber;
      case NotificationType.error:
        return Colors.red;
      case NotificationType.info:
        return Colors.cyan;
    }
  }
}
