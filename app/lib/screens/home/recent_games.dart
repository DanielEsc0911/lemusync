import 'package:flutter/material.dart';

import '../../bridge/link/model.dart' as dto;
import '../../design/components/ls_surface.dart';
import '../../design/tokens/colors.g.dart';
import '../../design/tokens/space.g.dart';
import '../../design/tokens/type.g.dart';
import '../../l10n/generated/app_localizations.dart';
import 'status_mapper.dart';

/// Most rows Home shows; the Games screen has the full library.
const int recentLimit = 8;

/// The "Recent games" heading and up to [recentLimit] read-only rows, in the
/// order the engine gives them (the tray's order). Lazy.
class RecentGamesSliver extends StatelessWidget {
  const RecentGamesSliver({super.key, required this.recent, required this.now});

  final List<dto.RecentDto> recent;
  final DateTime now;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final c = Theme.of(context).extension<LsColors>()!;
    final rows = recent.take(recentLimit).toList();
    return SliverList.builder(
      itemCount: rows.length + 1,
      itemBuilder: (context, i) {
        if (i == 0) {
          return Padding(
            padding: const EdgeInsets.only(top: LsSpace.lg, bottom: LsSpace.sm),
            child: Semantics(
              header: true,
              child: Text(
                l.homeRecent,
                style: LsType.title.copyWith(color: c.onSurface),
              ),
            ),
          );
        }
        return Padding(
          padding: const EdgeInsets.only(bottom: LsSpace.sm),
          child: _RecentRow(game: rows[i - 1], now: now),
        );
      },
    );
  }
}

/// Shown instead of the list when nothing was played yet.
class RecentGamesEmpty extends StatelessWidget {
  const RecentGamesEmpty({super.key});

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final c = Theme.of(context).extension<LsColors>()!;
    return Padding(
      padding: const EdgeInsets.only(top: LsSpace.lg),
      child: Text(
        l.homeRecentEmpty,
        style: LsType.body.copyWith(color: c.onSurfaceMuted),
      ),
    );
  }
}

class _RecentRow extends StatelessWidget {
  const _RecentRow({required this.game, required this.now});

  final dto.RecentDto game;
  final DateTime now;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final c = Theme.of(context).extension<LsColors>()!;
    final when = game.lastVersionMs;
    final detail = [
      game.emulator,
      game.player,
      if (when != null) relative(l, now, when.toInt()),
    ].join(' · ');
    return MergeSemantics(
      child: LsSurface(
        padding: const EdgeInsets.symmetric(
          horizontal: LsSpace.md,
          vertical: LsSpace.sm,
        ),
        child: SizedBox(
          width: double.infinity,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            spacing: LsSpace.xs,
            children: [
              Text(
                gameTitle(game.name, game.key),
                style: LsType.label.copyWith(color: c.onSurface),
              ),
              Text(
                detail,
                style: LsType.caption.copyWith(color: c.onSurfaceMuted),
              ),
              if (!game.launchable)
                Text(
                  l.gamesNoFile,
                  style: LsType.caption.copyWith(color: c.onSurfaceMuted),
                ),
            ],
          ),
        ),
      ),
    );
  }
}
