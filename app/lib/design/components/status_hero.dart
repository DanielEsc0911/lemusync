import 'package:flutter/material.dart';

import '../../l10n/generated/app_localizations.dart';
import '../status.dart';
import '../tokens/colors.g.dart';
import '../tokens/size.g.dart';
import '../tokens/space.g.dart';
import '../tokens/type.g.dart';
import 'ls_surface.dart';
import 'progress_ring.dart';

/// The big sync status at the top of Home: a face-button dot (or the
/// progress ring while busy), the headline, a detail line and Sync now.
class StatusHero extends StatelessWidget {
  const StatusHero({
    super.key,
    required this.status,
    this.detail,
    this.uploads = 0,
    this.downloads = 0,
    this.onSync,
    this.syncPending = false,
  });

  final LsStatus status;
  final String? detail;
  final int uploads;
  final int downloads;
  final VoidCallback? onSync;

  /// A sync was requested and has not answered yet: the button is disabled
  /// and reads "Starting sync…".
  final bool syncPending;

  @override
  Widget build(BuildContext context) {
    final c = Theme.of(context).extension<LsColors>()!;
    final l = AppLocalizations.of(context);
    final syncing = status == LsStatus.syncing;
    final busy = syncing || status == LsStatus.scanning;
    final String headline;
    String? downloadLine;
    if (syncing && uploads > 0) {
      headline = l.statusUploading(uploads);
      if (downloads > 0) downloadLine = l.statusDownloading(downloads);
    } else if (syncing && downloads > 0) {
      headline = l.statusDownloading(downloads);
    } else {
      headline = status.label(l);
    }
    final lines = [?downloadLine, ?detail];

    final Widget lead = busy
        ? ProgressRing(size: LsSize.target, status: status)
        : DecoratedBox(
            decoration: BoxDecoration(
              color: status.color(c),
              shape: BoxShape.circle,
            ),
            child: SizedBox.square(
              dimension: LsSize.target,
              child: Icon(
                status.icon,
                size: LsSize.iconMd,
                color: status.onColor(c),
              ),
            ),
          );

    return LsSurface(
      level: LsSurfaceLevel.raised,
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        spacing: LsSpace.md,
        children: [
          lead,
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              spacing: LsSpace.xs,
              children: [
                Semantics(
                  header: true,
                  child: Text(
                    headline,
                    style: LsType.title.copyWith(color: c.onSurface),
                  ),
                ),
                if (lines.isNotEmpty)
                  Text(
                    lines.join(' · '),
                    style: LsType.body.copyWith(color: c.onSurfaceMuted),
                  ),
                if (onSync != null)
                  Padding(
                    padding: const EdgeInsets.only(top: LsSpace.sm),
                    child: FilledButton(
                      onPressed: syncPending ? null : onSync,
                      child: Text(
                        syncPending ? l.actionSyncStarting : l.actionSyncNow,
                      ),
                    ),
                  ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
