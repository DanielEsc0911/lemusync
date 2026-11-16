import 'dart:math' as math;

import 'package:flutter/material.dart';

import '../status.dart';
import '../tokens/colors.g.dart';
import '../tokens/motion.g.dart';
import '../tokens/size.g.dart';
import 'ring_painter.dart';

/// Lemur-tail progress. Determinate when [value] (0–1) is set, otherwise
/// a spinning ring. With reduced motion the indeterminate ring stands still.
class ProgressRing extends StatefulWidget {
  const ProgressRing({
    super.key,
    this.value,
    this.size = LsSize.iconLg,
    this.status = LsStatus.syncing,
    this.semanticsLabel,
  });

  final double? value;
  final double size;
  final LsStatus status;
  final String? semanticsLabel;

  @override
  State<ProgressRing> createState() => _ProgressRingState();
}

class _ProgressRingState extends State<ProgressRing>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller = AnimationController(
    vsync: this,
    duration: LsMotion.slow * 4,
  );

  void _update() {
    final animate =
        widget.value == null && !MediaQuery.disableAnimationsOf(context);
    _sync(animate);
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    _update();
  }

  @override
  void didUpdateWidget(ProgressRing old) {
    super.didUpdateWidget(old);
    _update();
  }

  void _sync(bool animate) {
    if (animate && !_controller.isAnimating) {
      _controller.repeat();
    } else if (!animate && _controller.isAnimating) {
      _controller.stop();
    }
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final c = Theme.of(context).extension<LsColors>()!;
    final value = widget.value?.clamp(0.0, 1.0);
    final color = widget.status.color(c);

    // Few segments lit as soon as there is any progress; full only at 1.
    final filled = value == null
        ? 4 // A comet of four lit segments that turns around the ring.
        : value > 0
        ? math.max(1, (value * RingPainter.segments).floor())
        : 0;
    final ring = CustomPaint(
      size: Size.square(widget.size),
      painter: RingPainter(
        filled: filled,
        active: color,
        idle: c.outline,
        stroke: LsSize.ringStroke,
        turns: value == null ? _controller : null,
      ),
    );
    return Semantics(
      label: widget.semanticsLabel,
      value: value == null ? null : '${(value * 100).round()}%',
      container: true,
      excludeSemantics: true,
      child: RepaintBoundary(child: ring),
    );
  }
}
