import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';
import '../theme/syn_theme.dart';
import 'syn_container.dart';

/// Toast notification types
enum SynToastType {
  info,
  success,
  warning,
  memory,
  relationship,
  achievement,
}

/// Data for a toast notification
class SynToastData {
  final String title;
  final String? subtitle;
  final IconData? icon;
  final SynToastType type;
  final Duration duration;
  final VoidCallback? onTap;

  const SynToastData({
    required this.title,
    this.subtitle,
    this.icon,
    this.type = SynToastType.info,
    this.duration = const Duration(seconds: 4),
    this.onTap,
  });
}

/// Activity indicator toast system.
///
/// Displays sliding notifications for:
/// - Memory formation
/// - Relationship changes
/// - Events completing
/// - Achievements
class SynToastManager extends StatefulWidget {
  final Widget child;

  /// Global key to access toast manager
  static final GlobalKey<SynToastManagerState> globalKey = GlobalKey();

  const SynToastManager({
    super.key,
    required this.child,
  });

  /// Show a toast from anywhere in the app
  static void show(SynToastData toast) {
    globalKey.currentState?.showToast(toast);
  }

  @override
  State<SynToastManager> createState() => SynToastManagerState();
}

class SynToastManagerState extends State<SynToastManager> {
  final List<SynToastData> _toasts = [];
  static const int _maxToasts = 4;

  void showToast(SynToastData toast) {
    setState(() {
      _toasts.insert(0, toast);
      if (_toasts.length > _maxToasts) {
        _toasts.removeLast();
      }
    });

    // Auto-dismiss after duration
    Future.delayed(toast.duration, () {
      if (mounted && _toasts.contains(toast)) {
        setState(() => _toasts.remove(toast));
      }
    });
  }

  void _dismissToast(SynToastData toast) {
    setState(() => _toasts.remove(toast));
  }

  @override
  Widget build(BuildContext context) {
    return Stack(
      children: [
        widget.child,

        // Toast stack (top-right)
        Positioned(
          top: 80,
          right: 20,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.end,
            children: _toasts.asMap().entries.map((entry) {
              final index = entry.key;
              final toast = entry.value;
              return Padding(
                padding: const EdgeInsets.only(bottom: 8),
                child: _SynToast(
                  data: toast,
                  onDismiss: () => _dismissToast(toast),
                )
                    .animate()
                    .slideX(
                      begin: 1.0,
                      duration: SynTheme.normal,
                      curve: SynTheme.snapIn,
                      delay: Duration(milliseconds: index * 50),
                    )
                    .fadeIn(duration: SynTheme.fast),
              );
            }).toList(),
          ),
        ),
      ],
    );
  }
}

class _SynToast extends StatefulWidget {
  final SynToastData data;
  final VoidCallback onDismiss;

  const _SynToast({
    required this.data,
    required this.onDismiss,
  });

  @override
  State<_SynToast> createState() => _SynToastState();
}

class _SynToastState extends State<_SynToast> {
  bool _isHovered = false;

  Color get _accentColor {
    switch (widget.data.type) {
      case SynToastType.info:
        return SynTheme.accent;
      case SynToastType.success:
        return const Color(0xFF00FF88);
      case SynToastType.warning:
        return SynTheme.accentWarm;
      case SynToastType.memory:
        return const Color(0xFF9D4EDD);
      case SynToastType.relationship:
        return const Color(0xFFFF69B4);
      case SynToastType.achievement:
        return const Color(0xFFFFD700);
    }
  }

  IconData get _defaultIcon {
    switch (widget.data.type) {
      case SynToastType.info:
        return Icons.info_outline;
      case SynToastType.success:
        return Icons.check_circle_outline;
      case SynToastType.warning:
        return Icons.warning_amber_rounded;
      case SynToastType.memory:
        return Icons.psychology;
      case SynToastType.relationship:
        return Icons.favorite;
      case SynToastType.achievement:
        return Icons.emoji_events;
    }
  }

  @override
  Widget build(BuildContext context) {
    final icon = widget.data.icon ?? _defaultIcon;

    return MouseRegion(
      onEnter: (_) => setState(() => _isHovered = true),
      onExit: (_) => setState(() => _isHovered = false),
      child: GestureDetector(
        onTap: () {
          widget.data.onTap?.call();
          widget.onDismiss();
        },
        child: AnimatedContainer(
          duration: SynTheme.fast,
          transform: Matrix4.identity()
            ..rotateZ(_isHovered ? -0.01 : 0),
          child: Container(
            constraints: const BoxConstraints(maxWidth: 320),
            padding: const EdgeInsets.all(12),
            decoration: BoxDecoration(
              color: SynTheme.bgCard,
              border: Border(
                left: BorderSide(color: _accentColor, width: 3),
                top: BorderSide(color: _accentColor.withOpacity(0.3), width: 1),
                right: BorderSide(color: _accentColor.withOpacity(0.3), width: 1),
                bottom: BorderSide(color: _accentColor.withOpacity(0.3), width: 1),
              ),
              boxShadow: [
                BoxShadow(
                  color: _accentColor.withOpacity(0.2),
                  blurRadius: 12,
                  offset: const Offset(-4, 0),
                ),
              ],
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Container(
                  padding: const EdgeInsets.all(8),
                  decoration: BoxDecoration(
                    color: _accentColor.withOpacity(0.15),
                    shape: BoxShape.circle,
                  ),
                  child: Icon(
                    icon,
                    color: _accentColor,
                    size: 18,
                  ),
                ),
                const SizedBox(width: 12),
                Flexible(
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text(
                        widget.data.title,
                        style: SynTheme.label(color: SynTheme.textPrimary),
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                      ),
                      if (widget.data.subtitle != null) ...[
                        const SizedBox(height: 2),
                        Text(
                          widget.data.subtitle!,
                          style: SynTheme.caption(color: SynTheme.textMuted),
                          maxLines: 2,
                          overflow: TextOverflow.ellipsis,
                        ),
                      ],
                    ],
                  ),
                ),
                const SizedBox(width: 8),
                GestureDetector(
                  onTap: widget.onDismiss,
                  child: Icon(
                    Icons.close,
                    color: SynTheme.textMuted,
                    size: 16,
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}

/// Current activity indicator widget.
///
/// Shows what the character is currently doing with progress.
class SynActivityIndicator extends StatelessWidget {
  final String activity;
  final double? progress;
  final IconData? icon;
  final VoidCallback? onCancel;

  const SynActivityIndicator({
    super.key,
    required this.activity,
    this.progress,
    this.icon,
    this.onCancel,
  });

  @override
  Widget build(BuildContext context) {
    return SynContainer(
      enableHover: false,
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            // Activity spinner
            if (progress == null) ...[
              SizedBox(
                width: 16,
                height: 16,
                child: CircularProgressIndicator(
                  strokeWidth: 2,
                  valueColor: AlwaysStoppedAnimation(SynTheme.accent),
                ),
              ),
            ] else ...[
              SizedBox(
                width: 16,
                height: 16,
                child: CircularProgressIndicator(
                  value: progress!,
                  strokeWidth: 2,
                  valueColor: AlwaysStoppedAnimation(SynTheme.accent),
                  backgroundColor: SynTheme.bgSurface,
                ),
              ),
            ],
            const SizedBox(width: 12),
            
            // Activity label
            Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(
                  'CURRENT ACTIVITY',
                  style: SynTheme.caption(color: SynTheme.textMuted),
                ),
                Text(
                  activity.toUpperCase(),
                  style: SynTheme.label(color: SynTheme.accent),
                ),
              ],
            ),

            // Cancel button
            if (onCancel != null) ...[
              const SizedBox(width: 16),
              GestureDetector(
                onTap: onCancel,
                child: Container(
                  padding: const EdgeInsets.all(4),
                  decoration: BoxDecoration(
                    border: Border.all(
                      color: SynTheme.accentHot.withOpacity(0.5),
                      width: 1,
                    ),
                  ),
                  child: Icon(
                    Icons.close,
                    color: SynTheme.accentHot,
                    size: 14,
                  ),
                ),
              ),
            ],
          ],
        ),
      ),
    ).animate(onPlay: (c) => c.repeat()).shimmer(
      duration: 3.seconds,
      color: SynTheme.accent.withOpacity(0.1),
    );
  }
}
