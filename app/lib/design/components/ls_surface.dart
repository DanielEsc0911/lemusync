import 'package:flutter/material.dart';

import '../tokens/colors.g.dart';
import '../tokens/shape.g.dart';
import '../tokens/size.g.dart';
import '../tokens/space.g.dart';

enum LsSurfaceLevel { sunken, base, raised }

/// A plastic panel: flat fill inside a one-pixel two-tone bevel.
/// The bevel is a gradient behind the fill, so rounded corners are safe.
/// `sunken` inverts the bevel so it reads pressed in.
class LsSurface extends StatelessWidget {
  const LsSurface({
    super.key,
    required this.child,
    this.level = LsSurfaceLevel.base,
    this.padding,
    this.radius,
  });

  final Widget child;
  final LsSurfaceLevel level;
  final EdgeInsetsGeometry? padding;
  final double? radius;

  @override
  Widget build(BuildContext context) {
    final c = Theme.of(context).extension<LsColors>()!;
    final outerRadius = radius ?? LsShape.radiusLg;
    final innerRadius = (outerRadius - LsSize.bevel).clamp(0.0, outerRadius);
    final fill = switch (level) {
      LsSurfaceLevel.sunken => c.surfaceSunken,
      LsSurfaceLevel.base => c.surface,
      LsSurfaceLevel.raised => c.surfaceRaised,
    };
    // Sunken reads pressed in: the light and shade swap.
    final bevel = level == LsSurfaceLevel.sunken
        ? [c.bevelDark, c.bevelLight]
        : [c.bevelLight, c.bevelDark];
    return DecoratedBox(
      decoration: BoxDecoration(
        gradient: LinearGradient(
          begin: Alignment.topLeft,
          end: Alignment.bottomRight,
          colors: bevel,
        ),
        borderRadius: BorderRadius.circular(outerRadius),
      ),
      child: Padding(
        padding: const EdgeInsets.all(LsSize.bevel),
        child: DecoratedBox(
          decoration: BoxDecoration(
            color: fill,
            borderRadius: BorderRadius.circular(innerRadius),
          ),
          child: Padding(
            padding: padding ?? const EdgeInsets.all(LsSpace.lg),
            child: child,
          ),
        ),
      ),
    );
  }
}
