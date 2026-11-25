import 'package:flutter/material.dart';

import '../../app/app_state.dart';
import '../../bridge/link/model.dart' as dto;
import '../../design/components/game_card.dart';
import '../../design/icons.dart';
import '../../design/status.dart';
import '../../design/tokens/colors.g.dart';
import '../../design/tokens/size.g.dart';
import '../../design/tokens/space.g.dart';
import '../../design/tokens/type.g.dart';
import '../../l10n/generated/app_localizations.dart';
import '../home/status_mapper.dart';

/// The library as a lazy grid of read-only cards, in the order given. Card
/// width is capped by `LsSize.cardMaxWidth`; the column count follows from
/// the available width ([gameColumns]). Rows are built lazily and each row is
/// as tall as its tallest cell, so cards of one row line up and rows never
/// leave a fixed-height gap or clip at large text sizes. A sliver, so the
/// screen can put things above it.
class GameGridSliver extends StatelessWidget {
  const GameGridSliver({super.key, required this.state, required this.now});

  final AppState state;
  final DateTime now;

  @override
  Widget build(BuildContext context) {
    final games = state.games;
    return SliverLayoutBuilder(
      builder: (context, constraints) {
        final cols = gameColumns(constraints.crossAxisExtent);
        final rows = (games.length + cols - 1) ~/ cols;
        return SliverList.separated(
          itemCount: rows,
          separatorBuilder: (_, _) => const SizedBox(height: LsSpace.md),
          itemBuilder: (context, r) => IntrinsicHeight(
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              spacing: LsSpace.md,
              children: [
                for (var i = r * cols; i < (r + 1) * cols; i++)
                  Expanded(
                    child: i < games.length
                        ? _Cell(
                            game: games[i],
                            status: state.cardStatus(games[i]),
                            now: now,
                          )
                        : const SizedBox.shrink(),
                  ),
              ],
            ),
          ),
        );
      },
    );
  }
}

class _Cell extends StatelessWidget {
  const _Cell({required this.game, required this.status, required this.now});

  final dto.GameEntry game;
  final LsStatus status;
  final DateTime now;

  /// The newest `last_version_ms` of any stream; null when none has one.
  int? get _newest {
    int? best;
    for (final s in game.streams) {
      final ms = s.lastVersionMs?.toInt();
      if (ms != null && (best == null || ms > best)) best = ms;
    }
    return best;
  }

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final c = Theme.of(context).extension<LsColors>()!;
    final newest = _newest;
    final players = game.streams.length;
    final emulator = players == 0
        ? game.emulator
        : '${game.emulator} · ${l.gamesPlayers(players)}';
    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      spacing: LsSpace.xs,
      children: [
        GameCard(
          title: gameTitle(game.name, game.key),
          emulator: emulator,
          status: status,
          lastSynced: newest == null ? null : relative(l, now, newest),
        ),
        if (!game.launchable)
          Row(
            spacing: LsSpace.xs,
            children: [
              Icon(
                LsIcons.folder,
                size: LsSize.iconSm,
                color: c.onSurfaceMuted,
              ),
              Expanded(
                child: Text(
                  l.gamesNoFile,
                  style: LsType.caption.copyWith(color: c.onSurfaceMuted),
                ),
              ),
            ],
          ),
      ],
    );
  }
}

/// Columns for [width] under the max-extent rule: every card at most
/// `LsSize.cardMaxWidth` wide, `LsSpace.md` apart.
int gameColumns(double width) =>
    (width / (LsSize.cardMaxWidth + LsSpace.md)).ceil().clamp(1, 1 << 20);
