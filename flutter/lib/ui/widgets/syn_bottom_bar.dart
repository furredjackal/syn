import 'package:flutter/material.dart';
import 'package:flutter_animate/flutter_animate.dart';
import '../theme/syn_theme.dart';
import 'syn_container.dart';

/// Bottom action bar for gameplay HUD.
///
/// Features:
/// - Time controls (pause, 1x, 2x, 4x speeds)
/// - Day/time display with advance button
/// - Quick action buttons (rest, work, social)
/// - Mini stat indicators
/// - Activity status display
class SynBottomBar extends StatefulWidget {
  /// Current simulation day
  final int day;

  /// Current year
  final int year;

  /// Current hour (0-23)
  final int hour;

  /// Current activity/action being performed
  final String? currentActivity;

  /// Quick stat values for mini display
  final double health;
  final double energy;
  final double mood;

  /// Whether simulation is paused
  final bool isPaused;

  /// Current speed multiplier (1, 2, 4)
  final int speedMultiplier;

  /// Callbacks
  final VoidCallback? onAdvanceTime;
  final VoidCallback? onTogglePause;
  final ValueChanged<int>? onSpeedChange;
  final VoidCallback? onOpenCalendar;
  final VoidCallback? onRest;
  final VoidCallback? onWork;
  final VoidCallback? onSocialize;

  const SynBottomBar({
    super.key,
    required this.day,
    required this.year,
    required this.hour,
    this.currentActivity,
    this.health = 100,
    this.energy = 100,
    this.mood = 50,
    this.isPaused = false,
    this.speedMultiplier = 1,
    this.onAdvanceTime,
    this.onTogglePause,
    this.onSpeedChange,
    this.onOpenCalendar,
    this.onRest,
    this.onWork,
    this.onSocialize,
  });

  @override
  State<SynBottomBar> createState() => _SynBottomBarState();
}

class _SynBottomBarState extends State<SynBottomBar> {
  String _formatHour(int hour) {
    final period = hour >= 12 ? 'PM' : 'AM';
    final displayHour = hour == 0 ? 12 : (hour > 12 ? hour - 12 : hour);
    return '$displayHour:00 $period';
  }

  String _getTimeOfDayLabel(int hour) {
    if (hour >= 5 && hour < 12) return 'MORNING';
    if (hour >= 12 && hour < 17) return 'AFTERNOON';
    if (hour >= 17 && hour < 21) return 'EVENING';
    return 'NIGHT';
  }

  Color _getTimeOfDayColor(int hour) {
    if (hour >= 5 && hour < 12) return const Color(0xFFFFB347); // Orange
    if (hour >= 12 && hour < 17) return const Color(0xFF87CEEB); // Sky blue
    if (hour >= 17 && hour < 21) return const Color(0xFFDDA0DD); // Purple
    return const Color(0xFF1E3A5F); // Dark blue
  }

  @override
  Widget build(BuildContext context) {
    return SynContainer(
      enableHover: false,
      padding: EdgeInsets.zero,
      child: Container(
        height: 72,
        padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
        child: Row(
          children: [
            // Left section: Time display & controls
            _buildTimeSection(),
            
            const SizedBox(width: 24),
            
            // Divider
            Container(
              width: 2,
              height: 48,
              decoration: BoxDecoration(
                gradient: LinearGradient(
                  begin: Alignment.topCenter,
                  end: Alignment.bottomCenter,
                  colors: [
                    SynTheme.accent.withOpacity(0),
                    SynTheme.accent.withOpacity(0.5),
                    SynTheme.accent.withOpacity(0),
                  ],
                ),
              ),
            ),
            
            const SizedBox(width: 24),
            
            // Center section: Quick actions
            Expanded(child: _buildQuickActions()),
            
            const SizedBox(width: 24),
            
            // Divider
            Container(
              width: 2,
              height: 48,
              decoration: BoxDecoration(
                gradient: LinearGradient(
                  begin: Alignment.topCenter,
                  end: Alignment.bottomCenter,
                  colors: [
                    SynTheme.accent.withOpacity(0),
                    SynTheme.accent.withOpacity(0.5),
                    SynTheme.accent.withOpacity(0),
                  ],
                ),
              ),
            ),
            
            const SizedBox(width: 24),
            
            // Right section: Mini stats
            _buildMiniStats(),
          ],
        ),
      ),
    );
  }

  Widget _buildTimeSection() {
    final timeColor = _getTimeOfDayColor(widget.hour);

    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        // Calendar button
        _ActionButton(
          icon: Icons.calendar_month,
          onTap: widget.onOpenCalendar,
          tooltip: 'Open Calendar',
        ),
        
        const SizedBox(width: 12),
        
        // Time display
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          decoration: BoxDecoration(
            border: Border.all(color: timeColor.withOpacity(0.5), width: 1),
            color: SynTheme.bgCard,
          ),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  Container(
                    width: 8,
                    height: 8,
                    decoration: BoxDecoration(
                      color: timeColor,
                      shape: BoxShape.circle,
                      boxShadow: [
                        BoxShadow(
                          color: timeColor.withOpacity(0.6),
                          blurRadius: 6,
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(width: 8),
                  Text(
                    _getTimeOfDayLabel(widget.hour),
                    style: SynTheme.caption(color: timeColor),
                  ),
                ],
              ),
              const SizedBox(height: 2),
              Text(
                'DAY ${widget.day} • ${_formatHour(widget.hour)}',
                style: SynTheme.label(color: SynTheme.textPrimary),
              ),
            ],
          ),
        ),
        
        const SizedBox(width: 12),
        
        // Time controls
        Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            // Pause/Play
            _ActionButton(
              icon: widget.isPaused ? Icons.play_arrow : Icons.pause,
              isActive: widget.isPaused,
              activeColor: SynTheme.accentWarm,
              onTap: widget.onTogglePause,
              tooltip: widget.isPaused ? 'Resume' : 'Pause',
            ),
            const SizedBox(width: 4),
            // Speed buttons
            ...List.generate(3, (i) {
              final speed = [1, 2, 4][i];
              final isActive = widget.speedMultiplier == speed;
              return Padding(
                padding: const EdgeInsets.only(left: 4),
                child: _SpeedButton(
                  speed: speed,
                  isActive: isActive,
                  onTap: () => widget.onSpeedChange?.call(speed),
                ),
              );
            }),
            const SizedBox(width: 8),
            // Advance button
            _ActionButton(
              icon: Icons.skip_next,
              onTap: widget.onAdvanceTime,
              tooltip: 'Advance 1 Hour',
              size: 32,
              highlight: true,
            ),
          ],
        ),
      ],
    );
  }

  Widget _buildQuickActions() {
    return Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        // Current activity indicator
        if (widget.currentActivity != null) ...[
          Container(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 6),
            decoration: BoxDecoration(
              border: Border.all(color: SynTheme.accent.withOpacity(0.3)),
              color: SynTheme.accent.withOpacity(0.1),
            ),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                SizedBox(
                  width: 12,
                  height: 12,
                  child: CircularProgressIndicator(
                    strokeWidth: 2,
                    valueColor: AlwaysStoppedAnimation(SynTheme.accent),
                  ),
                ),
                const SizedBox(width: 8),
                Text(
                  widget.currentActivity!.toUpperCase(),
                  style: SynTheme.caption(color: SynTheme.accent),
                ),
              ],
            ),
          ).animate(onPlay: (c) => c.repeat()).shimmer(
            duration: 2.seconds,
            color: SynTheme.accent.withOpacity(0.3),
          ),
          const SizedBox(width: 24),
        ],

        // Quick action buttons
        _QuickActionButton(
          icon: Icons.hotel,
          label: 'REST',
          onTap: widget.onRest,
        ),
        const SizedBox(width: 12),
        _QuickActionButton(
          icon: Icons.work,
          label: 'WORK',
          onTap: widget.onWork,
        ),
        const SizedBox(width: 12),
        _QuickActionButton(
          icon: Icons.groups,
          label: 'SOCIAL',
          onTap: widget.onSocialize,
        ),
      ],
    );
  }

  Widget _buildMiniStats() {
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        _MiniStat(
          icon: Icons.favorite,
          value: widget.health,
          color: SynTheme.accentHot,
          label: 'HP',
        ),
        const SizedBox(width: 12),
        _MiniStat(
          icon: Icons.bolt,
          value: widget.energy,
          color: SynTheme.accentWarm,
          label: 'EN',
        ),
        const SizedBox(width: 12),
        _MiniStat(
          icon: Icons.sentiment_satisfied,
          value: widget.mood,
          color: SynTheme.accent,
          label: 'MD',
        ),
      ],
    );
  }
}

class _ActionButton extends StatefulWidget {
  final IconData icon;
  final VoidCallback? onTap;
  final String? tooltip;
  final bool isActive;
  final Color? activeColor;
  final double size;
  final bool highlight;

  const _ActionButton({
    required this.icon,
    this.onTap,
    this.tooltip,
    this.isActive = false,
    this.activeColor,
    this.size = 28,
    this.highlight = false,
  });

  @override
  State<_ActionButton> createState() => _ActionButtonState();
}

class _ActionButtonState extends State<_ActionButton> {
  bool _isHovered = false;

  @override
  Widget build(BuildContext context) {
    final color = widget.isActive
        ? (widget.activeColor ?? SynTheme.accent)
        : (_isHovered ? SynTheme.accent : SynTheme.textMuted);

    return MouseRegion(
      onEnter: (_) => setState(() => _isHovered = true),
      onExit: (_) => setState(() => _isHovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: Tooltip(
          message: widget.tooltip ?? '',
          child: AnimatedContainer(
            duration: SynTheme.fast,
            width: widget.size,
            height: widget.size,
            decoration: BoxDecoration(
              border: Border.all(
                color: widget.highlight
                    ? SynTheme.accent.withOpacity(_isHovered ? 1.0 : 0.5)
                    : color.withOpacity(0.5),
                width: widget.highlight ? 2 : 1,
              ),
              color: widget.isActive || _isHovered
                  ? color.withOpacity(0.15)
                  : Colors.transparent,
            ),
            child: Icon(
              widget.icon,
              size: widget.size * 0.6,
              color: color,
            ),
          ),
        ),
      ),
    );
  }
}

class _SpeedButton extends StatefulWidget {
  final int speed;
  final bool isActive;
  final VoidCallback? onTap;

  const _SpeedButton({
    required this.speed,
    required this.isActive,
    this.onTap,
  });

  @override
  State<_SpeedButton> createState() => _SpeedButtonState();
}

class _SpeedButtonState extends State<_SpeedButton> {
  bool _isHovered = false;

  @override
  Widget build(BuildContext context) {
    final color = widget.isActive
        ? SynTheme.accent
        : (_isHovered ? SynTheme.textPrimary : SynTheme.textMuted);

    return MouseRegion(
      onEnter: (_) => setState(() => _isHovered = true),
      onExit: (_) => setState(() => _isHovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: SynTheme.fast,
          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 4),
          decoration: BoxDecoration(
            border: Border.all(color: color.withOpacity(0.5), width: 1),
            color: widget.isActive ? color.withOpacity(0.2) : Colors.transparent,
          ),
          child: Text(
            '${widget.speed}x',
            style: SynTheme.caption(color: color),
          ),
        ),
      ),
    );
  }
}

class _QuickActionButton extends StatefulWidget {
  final IconData icon;
  final String label;
  final VoidCallback? onTap;

  const _QuickActionButton({
    required this.icon,
    required this.label,
    this.onTap,
  });

  @override
  State<_QuickActionButton> createState() => _QuickActionButtonState();
}

class _QuickActionButtonState extends State<_QuickActionButton> {
  bool _isHovered = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _isHovered = true),
      onExit: (_) => setState(() => _isHovered = false),
      child: GestureDetector(
        onTap: widget.onTap,
        child: AnimatedContainer(
          duration: SynTheme.fast,
          transform: Matrix4.identity()..rotateZ(_isHovered ? -0.02 : 0),
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          decoration: BoxDecoration(
            border: Border.all(
              color: _isHovered
                  ? SynTheme.accent
                  : SynTheme.accent.withOpacity(0.3),
              width: 1,
            ),
            color: _isHovered
                ? SynTheme.accent.withOpacity(0.15)
                : Colors.transparent,
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(
                widget.icon,
                size: 16,
                color: _isHovered ? SynTheme.accent : SynTheme.textMuted,
              ),
              const SizedBox(width: 6),
              Text(
                widget.label,
                style: SynTheme.caption(
                  color: _isHovered ? SynTheme.accent : SynTheme.textMuted,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

class _MiniStat extends StatelessWidget {
  final IconData icon;
  final double value;
  final Color color;
  final String label;

  const _MiniStat({
    required this.icon,
    required this.value,
    required this.color,
    required this.label,
  });

  @override
  Widget build(BuildContext context) {
    final fillPercent = (value / 100).clamp(0.0, 1.0);

    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(icon, size: 12, color: color),
            const SizedBox(width: 4),
            Text(
              label,
              style: SynTheme.caption(color: SynTheme.textMuted),
            ),
          ],
        ),
        const SizedBox(height: 4),
        Container(
          width: 48,
          height: 6,
          decoration: BoxDecoration(
            color: SynTheme.bgSurface,
            border: Border.all(color: color.withOpacity(0.3), width: 1),
          ),
          child: FractionallySizedBox(
            alignment: Alignment.centerLeft,
            widthFactor: fillPercent,
            child: Container(
              decoration: BoxDecoration(
                color: color,
                boxShadow: [
                  BoxShadow(
                    color: color.withOpacity(0.5),
                    blurRadius: 4,
                  ),
                ],
              ),
            ),
          ),
        ),
      ],
    );
  }
}
