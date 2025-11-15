import 'package:flutter/material.dart';

class TabBarWidget extends StatefulWidget {
  final List<String> tabs;
  final int initialIndex;
  final ValueChanged<int> onTabChanged;
  final Color activeColor;
  final Color inactiveColor;

  const TabBarWidget({
    Key? key,
    required this.tabs,
    this.initialIndex = 0,
    required this.onTabChanged,
    this.activeColor = Colors.cyan,
    this.inactiveColor = Colors.grey,
  }) : super(key: key);

  @override
  State<TabBarWidget> createState() => _TabBarWidgetState();
}

class _TabBarWidgetState extends State<TabBarWidget> {
  late int _selectedIndex;

  @override
  void initState() {
    super.initState();
    _selectedIndex = widget.initialIndex;
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
        border: Border(
          bottom: BorderSide(color: Colors.cyan.withOpacity(0.3), width: 1),
        ),
      ),
      child: Row(
        children: List.generate(
          widget.tabs.length,
          (index) => TabButtonWidget(
            label: widget.tabs[index],
            isActive: _selectedIndex == index,
            onPressed: () {
              setState(() {
                _selectedIndex = index;
              });
              widget.onTabChanged(index);
            },
            activeColor: widget.activeColor,
            inactiveColor: widget.inactiveColor,
          ),
        ),
      ),
    );
  }
}

class TabButtonWidget extends StatefulWidget {
  final String label;
  final bool isActive;
  final VoidCallback onPressed;
  final Color activeColor;
  final Color inactiveColor;

  const TabButtonWidget({
    Key? key,
    required this.label,
    required this.isActive,
    required this.onPressed,
    required this.activeColor,
    required this.inactiveColor,
  }) : super(key: key);

  @override
  State<TabButtonWidget> createState() => _TabButtonWidgetState();
}

class _TabButtonWidgetState extends State<TabButtonWidget> {
  bool _isHovering = false;

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _isHovering = true),
      onExit: (_) => setState(() => _isHovering = false),
      child: GestureDetector(
        onTap: widget.onPressed,
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 12),
          decoration: BoxDecoration(
            border: Border(
              bottom: BorderSide(
                color: widget.isActive ? widget.activeColor : Colors.transparent,
                width: 2,
              ),
            ),
            color: _isHovering && !widget.isActive
                ? Colors.grey.withOpacity(0.1)
                : Colors.transparent,
          ),
          child: Text(
            widget.label,
            style: Theme.of(context).textTheme.bodyMedium?.copyWith(
                  color: widget.isActive ? widget.activeColor : widget.inactiveColor,
                  fontWeight: widget.isActive ? FontWeight.bold : FontWeight.normal,
                ),
          ),
        ),
      ),
    );
  }
}
