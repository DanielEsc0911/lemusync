import 'package:flutter/material.dart';

import '../../l10n/generated/app_localizations.dart';
import '../icons.dart';
import '../status.dart';
import '../tokens/colors.g.dart';
import '../tokens/shape.g.dart';
import '../tokens/size.g.dart';
import '../tokens/space.g.dart';
import '../tokens/type.g.dart';
import 'cartridge_border.dart';
import 'ls_surface.dart';
import 'status_badge.dart';

/// A game as a cartridge: grip notch on top, a sunken label with the title
/// and emulator, the status dot, an optional lease line and Play.
class GameCard extends StatelessWidget {
  const GameCard({
    super.key,
    required this.title,
    required this.emulator,
    this.status = LsStatus.synced,
    this.lastSynced,
    this.lease,
    this.onPlay,
    this.onTap,
  });

  final String title;
  final String emulator;
  final LsStatus status;

  /// Preformatted time, for example "2 min ago".
  final String? lastSynced;

  /// Preformatted lease text, for example "In use on Desktop".
  final String? lease;
  final VoidCallback? onPlay;
  final VoidCallback? onTap;

  @override
  Widget build(BuildContext context) {
    final c = Theme.of(context).extension<LsColors>()!;
    final l = AppLocalizations.of(context);
    const outer = CartridgeBorder();
    const inner = CartridgeBorder(radius: LsShape.radiusLg - LsSize.bevel);
    final label = [title, emulator, status.label(l), ?lease].join(', ');

    final info = Semantics(
      label: label,
      container: true,
      excludeSemantics: true,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        spacing: LsSpace.sm,
        children: [
          LsSurface(
            level: LsSurfaceLevel.sunken,
            radius: LsShape.radiusMd,
            padding: const EdgeInsets.all(LsSpace.md),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              spacing: LsSpace.xs,
              children: [
                Text(
                  title,
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                  style: LsType.title.copyWith(color: c.onSurface),
                ),
                Text(
                  emulator,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                  style: LsType.caption.copyWith(color: c.onSurfaceMuted),
                ),
              ],
            ),
          ),
          if (lease != null)
            Row(
              spacing: LsSpace.xs,
              children: [
                Icon(LsIcons.lock, size: LsSize.iconSm, color: c.attention),
                Expanded(
                  child: Text(
                    lease!,
                    style: LsType.caption.copyWith(color: c.onSurfaceMuted),
                  ),
                ),
              ],
            ),
          if (lastSynced != null)
            Text(
              l.gameLastSynced(lastSynced!),
              style: LsType.caption.copyWith(color: c.onSurfaceMuted),
            ),
        ],
      ),
    );

    return ClipPath(
      clipper: const ShapeBorderClipper(shape: outer),
      child: DecoratedBox(
        decoration: BoxDecoration(
          gradient: LinearGradient(
            begin: Alignment.topLeft,
            end: Alignment.bottomRight,
            colors: [c.bevelLight, c.bevelDark],
          ),
        ),
        child: Padding(
          padding: const EdgeInsets.all(LsSize.bevel),
          child: ClipPath(
            clipper: const ShapeBorderClipper(shape: inner),
            child: ColoredBox(
              color: c.surfaceRaised,
              child: Material(
                type: MaterialType.transparency,
                child: InkWell(
                  onTap: onTap,
                  child: Padding(
                    padding: const EdgeInsets.fromLTRB(
                      LsSpace.md,
                      CartridgeBorder.notchDepth + LsSpace.sm,
                      LsSpace.md,
                      LsSpace.md,
                    ),
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.stretch,
                      spacing: LsSpace.sm,
                      children: [
                        info,
                        Row(
                          children: [
                            StatusBadge(status: status, compact: true),
                            const Spacer(),
                            IconButton.filledTonal(
                              onPressed: onPlay,
                              tooltip: l.gamePlay,
                              icon: const Icon(LsIcons.play),
                            ),
                          ],
                        ),
                      ],
                    ),
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
