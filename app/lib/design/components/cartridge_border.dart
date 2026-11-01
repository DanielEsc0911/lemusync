import 'package:flutter/widgets.dart';

import '../tokens/shape.g.dart';
import '../tokens/space.g.dart';

/// A rounded rectangle with a shallow grip notch centred on the top edge,
/// like the top of a game cartridge. It paints nothing: the bevel comes from
/// the gradient behind the fill (see `LsSurface`), clipped to this shape.
class CartridgeBorder extends OutlinedBorder {
  const CartridgeBorder({this.radius = LsShape.radiusLg, super.side});

  /// Notch width as a share of the card width (the shape's own geometry).
  static const double _notchShare = 0.4;

  /// Depth of the notch.
  static const double notchDepth = LsSpace.md;

  final double radius;

  @override
  EdgeInsetsGeometry get dimensions =>
      EdgeInsets.only(top: notchDepth + side.width);

  @override
  Path getOuterPath(Rect rect, {TextDirection? textDirection}) {
    final r = radius.clamp(0.0, rect.shortestSide / 2);
    final halfNotch = rect.width * _notchShare / 2;
    final nl = rect.center.dx - halfNotch;
    final nr = rect.center.dx + halfNotch;
    const d = notchDepth;
    return Path()
      ..moveTo(rect.left + r, rect.top)
      ..lineTo(nl, rect.top)
      ..lineTo(nl + d, rect.top + d)
      ..lineTo(nr - d, rect.top + d)
      ..lineTo(nr, rect.top)
      ..lineTo(rect.right - r, rect.top)
      ..arcToPoint(Offset(rect.right, rect.top + r), radius: Radius.circular(r))
      ..lineTo(rect.right, rect.bottom - r)
      ..arcToPoint(
        Offset(rect.right - r, rect.bottom),
        radius: Radius.circular(r),
      )
      ..lineTo(rect.left + r, rect.bottom)
      ..arcToPoint(
        Offset(rect.left, rect.bottom - r),
        radius: Radius.circular(r),
      )
      ..lineTo(rect.left, rect.top + r)
      ..arcToPoint(Offset(rect.left + r, rect.top), radius: Radius.circular(r))
      ..close();
  }

  @override
  Path getInnerPath(Rect rect, {TextDirection? textDirection}) =>
      getOuterPath(rect.deflate(side.width), textDirection: textDirection);

  @override
  void paint(Canvas canvas, Rect rect, {TextDirection? textDirection}) {}

  @override
  CartridgeBorder copyWith({BorderSide? side, double? radius}) =>
      CartridgeBorder(radius: radius ?? this.radius, side: side ?? this.side);

  @override
  ShapeBorder scale(double t) =>
      CartridgeBorder(radius: radius * t, side: side.scale(t));

  @override
  bool operator ==(Object other) =>
      other is CartridgeBorder && other.radius == radius && other.side == side;

  @override
  int get hashCode => Object.hash(radius, side);
}
