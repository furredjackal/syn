import 'package:flutter/material.dart';

class CustomScrollViewWidget extends StatefulWidget {
  final Widget child;
  final ScrollController? scrollController;
  final double scrollbarWidth;
  final Color scrollbarColor;

  const CustomScrollViewWidget({
    Key? key,
    required this.child,
    this.scrollController,
    this.scrollbarWidth = 8,
    this.scrollbarColor = Colors.cyan,
  }) : super(key: key);

  @override
  State<CustomScrollViewWidget> createState() => _CustomScrollViewWidgetState();
}

class _CustomScrollViewWidgetState extends State<CustomScrollViewWidget> {
  late ScrollController _scrollController;
  bool _isScrolling = false;

  @override
  void initState() {
    super.initState();
    _scrollController = widget.scrollController ?? ScrollController();
    _scrollController.addListener(_updateScrolling);
  }

  void _updateScrolling() {
    if (_scrollController.position.isScrollingNotifier.value != _isScrolling) {
      setState(() {
        _isScrolling = _scrollController.position.isScrollingNotifier.value;
      });
    }
  }

  @override
  void dispose() {
    _scrollController.removeListener(_updateScrolling);
    if (widget.scrollController == null) {
      _scrollController.dispose();
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return MouseRegion(
      onEnter: (_) => setState(() => _isScrolling = true),
      onExit: (_) => setState(() => _isScrolling = false),
      child: Scrollbar(
        controller: _scrollController,
        thumbVisibility: true,
        trackVisibility: true,
        thickness: widget.scrollbarWidth,
        radius: Radius.circular(widget.scrollbarWidth / 2),
        child: SingleChildScrollView(
          controller: _scrollController,
          child: widget.child,
        ),
      ),
    );
  }
}

class CustomScrollAreaWidget extends StatefulWidget {
  final List<Widget> children;
  final Axis scrollDirection;
  final bool shrinkWrap;
  final double spacing;

  const CustomScrollAreaWidget({
    Key? key,
    required this.children,
    this.scrollDirection = Axis.vertical,
    this.shrinkWrap = false,
    this.spacing = 8,
  }) : super(key: key);

  @override
  State<CustomScrollAreaWidget> createState() => _CustomScrollAreaWidgetState();
}

class _CustomScrollAreaWidgetState extends State<CustomScrollAreaWidget> {
  final ScrollController _scrollController = ScrollController();

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scrollbar(
      controller: _scrollController,
      thumbVisibility: true,
      trackVisibility: true,
      thickness: 8,
      child: widget.scrollDirection == Axis.vertical
          ? ListView.separated(
              controller: _scrollController,
              shrinkWrap: widget.shrinkWrap,
              itemCount: widget.children.length,
              separatorBuilder: (context, index) => SizedBox(height: widget.spacing),
              itemBuilder: (context, index) => widget.children[index],
            )
          : ListView.separated(
              controller: _scrollController,
              scrollDirection: Axis.horizontal,
              shrinkWrap: widget.shrinkWrap,
              itemCount: widget.children.length,
              separatorBuilder: (context, index) => SizedBox(width: widget.spacing),
              itemBuilder: (context, index) => widget.children[index],
            ),
    );
  }
}
