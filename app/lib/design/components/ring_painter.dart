import 'dart:math' as math;

import 'package:flutter/widgets.dart';

/// Paints a ring of tail segments with small gaps.
/// Segment `i` is drawn in [active] when `i < filled`, else in [idle].
class RingPainter extends CustomPainter {
  const RingPainter({
    required this.filled,
    required this.active,
    required this.idle,
    required this.stroke,
    this.rotation = 0,
    this.turns,
  }) : super(repaint: turns);

  static const int segments = 7;
  // The ring's own geometry: share of each segment's slot left empty.
  static const double _gapShare = 0.22;

  final int filled;
  final Color active;
  final Color idle;
  final double stroke;

  /// Turns (0–1) the whole pattern is rotated.
  final double rotation;

  /// Drives the rotation when set (turns 0–1); repaints without a rebuild.
  final Animation<double>? turns;

  @override
  void paint(Canvas canvas, Size size) {
    final rect = (Offset.zero & size).deflate(stroke / 2);
    const slot = 2 * math.pi / segments;
    const sweep = slot * (1 - _gapShare);
    final start =
        -math.pi / 2 +
        (turns?.value ?? rotation) * 2 * math.pi +
        slot * _gapShare / 2;
    final paint = Paint()
      ..style = PaintingStyle.stroke
      ..strokeWidth = stroke;
    for (var i = 0; i < segments; i++) {
      paint.color = i < filled ? active : idle;
      canvas.drawArc(rect, start + slot * i, sweep, false, paint);
    }
  }

  @override
  bool shouldRepaint(RingPainter old) =>
      filled != old.filled ||
      active != old.active ||
      idle != old.idle ||
      stroke != old.stroke ||
      (turns == null && rotation != old.rotation) ||
      turns != old.turns;
}
