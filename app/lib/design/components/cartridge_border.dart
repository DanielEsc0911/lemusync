import 'dart:math' as math;
import 'dart:ui' show lerpDouble;

import 'package:flutter/widgets.dart';

import '../tokens/shape.g.dart';
import '../tokens/space.g.dart';

/// A cartridge outline: small top corners, large bottom corners and a shallow
/// grip notch centred on the top edge, its inner joints rounded. `paint`
/// strokes [side] (the edge line) when it has a width; the bevel comes from
/// the gradient behind the fill (see `LsSurface`), clipped to this shape.
class CartridgeBorder extends OutlinedBorder {
  const CartridgeBorder({
    this.radius = LsShape.radiusLg,
    this.topRadius = LsShape.radiusSm,
    super.side,
  });

  /// Notch width as a share of the card width (the shape's own geometry).
  static const double _notchShare = 0.4;

  /// Depth of the notch. `LsSpace.md` (12): at `sm` (8) the notch was too
  /// shallow to read as a grip at card size.
  static const double notchDepth = LsSpace.md;

  /// Radius of the notch's inner joints.
  static const double _joint = LsShape.radiusSm;

  /// Bottom corner radius.
  final double radius;

  /// Top corner radius.
  final double topRadius;

  @override
  EdgeInsetsGeometry get dimensions =>
      EdgeInsets.only(top: notchDepth + side.width);

  @override
  Path getOuterPath(Rect rect, {TextDirection? textDirection}) {
    final rb = radius.clamp(0.0, rect.shortestSide / 2);
    final rt = topRadius.clamp(0.0, rect.shortestSide / 2);
    const d = notchDepth;
    // Never narrower than the depth, so the slopes cannot cross.
    final halfNotch = math.max(rect.width * _notchShare / 2, d);
    final nl = rect.center.dx - halfNotch;
    final nr = rect.center.dx + halfNotch;
    final fits = nl >= rect.left + rt && nr <= rect.right - rt;
    final path = Path()..moveTo(rect.left + rt, rect.top);
    if (fits) {
      final j = math.min(_joint, halfNotch - d);
      final top = rect.top;
      path
        ..lineTo(nl, top)
        ..lineTo(nl + d - j * math.sqrt1_2, top + d - j * math.sqrt1_2)
        ..quadraticBezierTo(nl + d, top + d, nl + d + j, top + d)
        ..lineTo(nr - d - j, top + d)
        ..quadraticBezierTo(
          nr - d,
          top + d,
          nr - d + j * math.sqrt1_2,
          top + d - j * math.sqrt1_2,
        )
        ..lineTo(nr, top);
    }
    return path
      ..lineTo(rect.right - rt, rect.top)
      ..arcToPoint(
        Offset(rect.right, rect.top + rt),
        radius: Radius.circular(rt),
      )
      ..lineTo(rect.right, rect.bottom - rb)
      ..arcToPoint(
        Offset(rect.right - rb, rect.bottom),
        radius: Radius.circular(rb),
      )
      ..lineTo(rect.left + rb, rect.bottom)
      ..arcToPoint(
        Offset(rect.left, rect.bottom - rb),
        radius: Radius.circular(rb),
      )
      ..lineTo(rect.left, rect.top + rt)
      ..arcToPoint(
        Offset(rect.left + rt, rect.top),
        radius: Radius.circular(rt),
      )
      ..close();
  }

  @override
  Path getInnerPath(Rect rect, {TextDirection? textDirection}) =>
      getOuterPath(rect.deflate(side.width), textDirection: textDirection);

  @override
  void paint(Canvas canvas, Rect rect, {TextDirection? textDirection}) {
    if (side.style == BorderStyle.none || side.width == 0) return;
    canvas.drawPath(
      getOuterPath(rect.deflate(side.width / 2), textDirection: textDirection),
      side.toPaint(),
    );
  }

  @override
  CartridgeBorder copyWith({
    BorderSide? side,
    double? radius,
    double? topRadius,
  }) => CartridgeBorder(
    radius: radius ?? this.radius,
    topRadius: topRadius ?? this.topRadius,
    side: side ?? this.side,
  );

  @override
  CartridgeBorder scale(double t) => CartridgeBorder(
    radius: radius * t,
    topRadius: topRadius * t,
    side: side.scale(t),
  );

  @override
  ShapeBorder? lerpFrom(ShapeBorder? a, double t) {
    if (a is CartridgeBorder) return _lerp(a, this, t);
    return super.lerpFrom(a, t);
  }

  @override
  ShapeBorder? lerpTo(ShapeBorder? b, double t) {
    if (b is CartridgeBorder) return _lerp(this, b, t);
    return super.lerpTo(b, t);
  }

  static CartridgeBorder _lerp(
    CartridgeBorder a,
    CartridgeBorder b,
    double t,
  ) => CartridgeBorder(
    radius: lerpDouble(a.radius, b.radius, t)!,
    topRadius: lerpDouble(a.topRadius, b.topRadius, t)!,
    side: BorderSide.lerp(a.side, b.side, t),
  );

  @override
  bool operator ==(Object other) =>
      other is CartridgeBorder &&
      other.radius == radius &&
      other.topRadius == topRadius &&
      other.side == side;

  @override
  int get hashCode => Object.hash(radius, topRadius, side);
}
