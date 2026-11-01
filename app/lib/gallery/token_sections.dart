import 'package:flutter/material.dart';

import '../design/design.dart';
import '../design/tokens/colors.g.dart';
import '../design/tokens/motion.g.dart';
import '../design/tokens/opacity.g.dart';
import '../design/tokens/shape.g.dart';
import '../design/tokens/size.g.dart';
import '../design/tokens/space.g.dart';
import '../design/tokens/type.g.dart';

// Layout-only width of a gallery tile; not a design value.
const double _cell = 168;

LsColors _colors(BuildContext context) =>
    Theme.of(context).extension<LsColors>()!;

String _hex(Color color) =>
    '#${(color.toARGB32() & 0xFFFFFF).toRadixString(16).padLeft(6, '0').toUpperCase()}';

/// Every color role: swatch, name, hex and an `onX` sample where it exists.
class ColorRolesSection extends StatelessWidget {
  const ColorRolesSection({super.key});

  @override
  Widget build(BuildContext context) {
    final c = _colors(context);
    // (role, color, onRole, onColor)
    final roles = <(String, Color, String?, Color?)>[
      ('surface', c.surface, 'onSurface  onSurfaceMuted', c.onSurface),
      ('surfaceSunken', c.surfaceSunken, 'onSurface', c.onSurface),
      ('surfaceRaised', c.surfaceRaised, 'onSurface', c.onSurface),
      ('outline', c.outline, null, null),
      ('bevelLight', c.bevelLight, null, null),
      ('bevelDark', c.bevelDark, null, null),
      ('accent', c.accent, 'onAccent', c.onAccent),
      ('focus', c.focus, null, null),
      ('synced', c.synced, 'onSynced', c.onSynced),
      ('syncing', c.syncing, 'onSyncing', c.onSyncing),
      ('attention', c.attention, 'onAttention', c.onAttention),
      ('danger', c.danger, 'onDanger', c.onDanger),
      ('offline', c.offline, 'onOffline', c.onOffline),
    ];
    return Wrap(
      spacing: LsSpace.md,
      runSpacing: LsSpace.md,
      children: [
        for (final (name, color, onName, onColor) in roles)
          SizedBox(
            width: _cell,
            child: LsSurface(
              level: LsSurfaceLevel.raised,
              padding: const EdgeInsets.all(LsSpace.sm),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                spacing: LsSpace.xs,
                children: [
                  DecoratedBox(
                    decoration: BoxDecoration(
                      color: color,
                      border: Border.all(color: c.outline),
                      borderRadius: BorderRadius.circular(LsShape.radiusSm),
                    ),
                    child: SizedBox(
                      height: LsSize.target,
                      width: double.infinity,
                      child: onColor == null
                          ? null
                          : Center(
                              child: Text(
                                'Aa',
                                style: LsType.title.copyWith(color: onColor),
                              ),
                            ),
                    ),
                  ),
                  Text(name, style: LsType.label.copyWith(color: c.onSurface)),
                  Text(
                    _hex(color),
                    style: LsType.mono.copyWith(color: c.onSurfaceMuted),
                  ),
                  if (onName != null)
                    Text(
                      'text: $onName ${_hex(onColor!)}',
                      style: LsType.caption.copyWith(color: c.onSurfaceMuted),
                    ),
                ],
              ),
            ),
          ),
      ],
    );
  }
}

class TypeScaleSection extends StatelessWidget {
  const TypeScaleSection({super.key});

  @override
  Widget build(BuildContext context) {
    final c = _colors(context);
    final styles = <(String, TextStyle)>[
      ('display', LsType.display),
      ('headline', LsType.headline),
      ('title', LsType.title),
      ('body', LsType.body),
      ('label', LsType.label),
      ('caption', LsType.caption),
      ('mono', LsType.mono),
    ];
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      spacing: LsSpace.sm,
      children: [
        for (final (name, style) in styles)
          Text(
            '$name  Save games, everywhere',
            style: style.copyWith(color: c.onSurface),
          ),
      ],
    );
  }
}

/// A row of labelled bars for a scale of lengths.
class _Scale extends StatelessWidget {
  const _Scale(this.entries, {this.shape = false});

  final List<(String, double)> entries;
  final bool shape;

  @override
  Widget build(BuildContext context) {
    final c = _colors(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      spacing: LsSpace.sm,
      children: [
        for (final (name, value) in entries)
          Row(
            spacing: LsSpace.md,
            children: [
              SizedBox(
                width: _cell,
                child: Text(
                  '$name  $value',
                  style: LsType.mono.copyWith(color: c.onSurface),
                ),
              ),
              if (shape)
                DecoratedBox(
                  decoration: BoxDecoration(
                    color: c.surfaceRaised,
                    border: Border.all(color: c.outline),
                    borderRadius: BorderRadius.circular(value),
                  ),
                  child: const SizedBox.square(dimension: LsSize.mark),
                )
              else
                ColoredBox(
                  color: c.accent,
                  child: SizedBox(width: value, height: LsSpace.md),
                ),
            ],
          ),
      ],
    );
  }
}

class SpaceSection extends StatelessWidget {
  const SpaceSection({super.key});

  @override
  Widget build(BuildContext context) => const _Scale([
    ('xs', LsSpace.xs),
    ('sm', LsSpace.sm),
    ('md', LsSpace.md),
    ('lg', LsSpace.lg),
    ('xl', LsSpace.xl),
    ('xxl', LsSpace.xxl),
    ('xxxl', LsSpace.xxxl),
  ]);
}

class SizeSection extends StatelessWidget {
  const SizeSection({super.key});

  @override
  Widget build(BuildContext context) => const _Scale([
    ('target', LsSize.target),
    ('iconSm', LsSize.iconSm),
    ('iconMd', LsSize.iconMd),
    ('iconLg', LsSize.iconLg),
    ('bevel', LsSize.bevel),
    ('focusRing', LsSize.focusRing),
    ('ringStroke', LsSize.ringStroke),
    ('mark', LsSize.mark),
  ]);
}

class ShapeSection extends StatelessWidget {
  const ShapeSection({super.key});

  @override
  Widget build(BuildContext context) => const _Scale([
    ('radiusSm', LsShape.radiusSm),
    ('radiusMd', LsShape.radiusMd),
    ('radiusLg', LsShape.radiusLg),
    ('radiusPill', LsShape.radiusPill),
  ], shape: true);
}

class OpacitySection extends StatelessWidget {
  const OpacitySection({super.key});

  @override
  Widget build(BuildContext context) {
    final c = _colors(context);
    const values = <(String, double)>[
      ('pressed', LsOpacity.pressed),
      ('hover', LsOpacity.hover),
      ('disabled', LsOpacity.disabled),
      ('focus', LsOpacity.focus),
    ];
    return Wrap(
      spacing: LsSpace.md,
      runSpacing: LsSpace.md,
      children: [
        for (final (name, v) in values)
          Column(
            spacing: LsSpace.xs,
            children: [
              SizedBox.square(
                dimension: LsSize.target,
                child: ColoredBox(color: c.accent.withValues(alpha: v)),
              ),
              Text(
                '$name  $v',
                style: LsType.mono.copyWith(color: c.onSurface),
              ),
            ],
          ),
      ],
    );
  }
}

/// Tap a box to slide it with each duration and curve. Honours reduce-motion.
class MotionSection extends StatelessWidget {
  const MotionSection({super.key});

  @override
  Widget build(BuildContext context) {
    const entries = <(String, Duration, Curve)>[
      ('fast  standard', LsMotion.fast, LsMotion.standard),
      ('medium  standard', LsMotion.medium, LsMotion.standard),
      ('slow  standard', LsMotion.slow, LsMotion.standard),
      ('medium  emphasized', LsMotion.medium, LsMotion.emphasized),
      ('slow  emphasized', LsMotion.slow, LsMotion.emphasized),
    ];
    return Column(
      spacing: LsSpace.sm,
      children: [
        for (final (name, duration, curve) in entries)
          _MotionRow(name: name, duration: duration, curve: curve),
      ],
    );
  }
}

class _MotionRow extends StatefulWidget {
  const _MotionRow({
    required this.name,
    required this.duration,
    required this.curve,
  });

  final String name;
  final Duration duration;
  final Curve curve;

  @override
  State<_MotionRow> createState() => _MotionRowState();
}

class _MotionRowState extends State<_MotionRow> {
  bool _end = false;

  @override
  Widget build(BuildContext context) {
    final c = _colors(context);
    final reduce = MediaQuery.disableAnimationsOf(context);
    return InkWell(
      onTap: () => setState(() => _end = !_end),
      child: Semantics(
        label: 'Animate ${widget.name}',
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              '${widget.name}  ${widget.duration.inMilliseconds} ms',
              style: LsType.mono.copyWith(color: c.onSurface),
            ),
            SizedBox(
              height: LsSize.target,
              child: AnimatedAlign(
                duration: reduce ? Duration.zero : widget.duration,
                curve: widget.curve,
                alignment: _end ? Alignment.centerRight : Alignment.centerLeft,
                child: DecoratedBox(
                  decoration: BoxDecoration(
                    color: c.accent,
                    borderRadius: BorderRadius.circular(LsShape.radiusSm),
                  ),
                  child: const SizedBox.square(dimension: LsSize.iconLg),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class IconsSection extends StatelessWidget {
  const IconsSection({super.key});

  @override
  Widget build(BuildContext context) {
    final c = _colors(context);
    const icons = <(String, IconData)>[
      ('synced', LsIcons.synced),
      ('syncing', LsIcons.syncing),
      ('paused', LsIcons.paused),
      ('offline', LsIcons.offline),
      ('attention', LsIcons.attention),
      ('error', LsIcons.error),
      ('play', LsIcons.play),
      ('history', LsIcons.history),
      ('restore', LsIcons.restore),
      ('settings', LsIcons.settings),
      ('home', LsIcons.home),
      ('games', LsIcons.games),
      ('activity', LsIcons.activity),
      ('devices', LsIcons.devices),
      ('folder', LsIcons.folder),
      ('cloud', LsIcons.cloud),
      ('lock', LsIcons.lock),
      ('chevronRight', LsIcons.chevronRight),
      ('close', LsIcons.close),
      ('theme', LsIcons.theme),
      ('language', LsIcons.language),
      ('performance', LsIcons.performance),
    ];
    return Wrap(
      spacing: LsSpace.md,
      runSpacing: LsSpace.md,
      children: [
        for (final (name, icon) in icons)
          SizedBox(
            width: LsSize.mark + LsSize.target,
            child: Column(
              spacing: LsSpace.xs,
              children: [
                Icon(icon, size: LsSize.iconLg, color: c.onSurface),
                Text(
                  name,
                  textAlign: TextAlign.center,
                  style: LsType.caption.copyWith(color: c.onSurfaceMuted),
                ),
              ],
            ),
          ),
      ],
    );
  }
}

class MarkSection extends StatelessWidget {
  const MarkSection({super.key});

  @override
  Widget build(BuildContext context) => Image.asset(
    'assets/brand/mark.png',
    width: LsSize.mark,
    height: LsSize.mark,
    excludeFromSemantics: true,
  );
}
