import 'package:flutter/material.dart';

import '../status.dart';
import '../tokens/colors.g.dart';
import '../tokens/shape.g.dart';
import '../tokens/size.g.dart';
import '../tokens/space.g.dart';
import '../tokens/type.g.dart';
import 'ls_surface.dart';

/// An inline message with a tone strip, the tone icon and an optional action.
class LsBanner extends StatelessWidget {
  const LsBanner({
    super.key,
    required this.tone,
    required this.message,
    this.actionLabel,
    this.onAction,
  });

  final LsStatus tone;
  final String message;
  final String? actionLabel;
  final VoidCallback? onAction;

  @override
  Widget build(BuildContext context) {
    final c = Theme.of(context).extension<LsColors>()!;
    final color = tone.color(c);
    const inner = LsShape.radiusLg - LsSize.bevel;
    return LsSurface(
      level: LsSurfaceLevel.raised,
      padding: EdgeInsets.zero,
      child: ClipRRect(
        borderRadius: BorderRadius.circular(inner),
        child: Stack(
          children: [
            Positioned(
              left: 0,
              top: 0,
              bottom: 0,
              width: LsSpace.xs,
              child: ColoredBox(color: color),
            ),
            Padding(
              padding: const EdgeInsets.only(left: LsSpace.xs),
              child: Padding(
                padding: const EdgeInsets.all(LsSpace.md),
                child: Wrap(
                  crossAxisAlignment: WrapCrossAlignment.center,
                  spacing: LsSpace.md,
                  runSpacing: LsSpace.xs,
                  children: [
                    Row(
                      mainAxisSize: MainAxisSize.min,
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Icon(tone.icon, size: LsSize.iconMd, color: color),
                        const SizedBox(width: LsSpace.sm),
                        Flexible(
                          child: Text(
                            message,
                            style: LsType.body.copyWith(color: c.onSurface),
                          ),
                        ),
                      ],
                    ),
                    if (actionLabel != null)
                      TextButton(
                        onPressed: onAction,
                        child: Text(actionLabel!),
                      ),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}
