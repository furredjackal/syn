import 'package:flutter/material.dart';

/// Shared SynAnimatedBuilder helper widget.
///
/// A simpler interface for AnimatedWidget that uses a builder pattern.
/// Named SynAnimatedBuilder to avoid conflict with Flutter's AnimatedBuilder.
class SynAnimatedBuilder extends AnimatedWidget {
  final Widget Function(BuildContext, Widget?) builder;
  final Widget? child;

  const SynAnimatedBuilder({
    super.key,
    required Animation<double> animation,
    required this.builder,
    this.child,
  }) : super(listenable: animation);

  @override
  Widget build(BuildContext context) => builder(context, child);
}
