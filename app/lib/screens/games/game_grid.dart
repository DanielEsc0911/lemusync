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
class GameGridSliver extends StatefulWidget {
  const GameGridSliver({super.key, required this.state, required this.now});

  final AppState state;
  final DateTime now;

  @override
  State<GameGridSliver> createState() => _GameGridSliverState();
}

class _GameGridSliverState extends State<GameGridSliver> {
  /// The last cell built for each game, reused while its inputs are equal.
  final Map<String, _Cell> _cells = {};

  _Cell _cell(dto.GameEntry game, LsStatus status, String? lastSynced) {
    final fresh = _Cell(game: game, status: status, lastSynced: lastSynced);
    final id = '${game.emulator}/${game.key}';
    final old = _cells[id];
    if (old != null && old.sameInputs(fresh)) return old;
    return _cells[id] = fresh;
  }

  @override
  Widget build(BuildContext context) {
    final state = widget.state;
    final now = widget.now;
    final games = state.games;
    final l = AppLocalizations.of(context);
    // Forget cells of games that left the library, so the cache stays the
    // size of the library.
    if (_cells.length > games.length) {
      final ids = {for (final g in games) '${g.emulator}/${g.key}'};
      _cells.removeWhere((id, _) => !ids.contains(id));
    }
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
                        ? _cell(
                            games[i],
                            state.cardStatus(games[i]),
                            _lastSyncedText(l, now, games[i]),
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

/// The relative time of the game's newest version, as text; null when no
/// stream has one. Text, not [now], is what a cell compares, so a cell whose
/// text did not change is not rebuilt.
String? _lastSyncedText(AppLocalizations l, DateTime now, dto.GameEntry game) {
  int? best;
  for (final s in game.streams) {
    final ms = s.lastVersionMs?.toInt();
    if (ms != null && (best == null || ms > best)) best = ms;
  }
  return best == null ? null : relative(l, now, best);
}

/// One card. [GameGridSliver] hands out the same instance while the inputs
/// are the same, so Flutter keeps the built cell when something else changed
/// (a live transfer of another game): the rebuild budget of the perf test.
class _Cell extends StatelessWidget {
  const _Cell({
    required this.game,
    required this.status,
    required this.lastSynced,
  });

  final dto.GameEntry game;
  final LsStatus status;
  final String? lastSynced;

  bool sameInputs(_Cell other) =>
      game == other.game &&
      status == other.status &&
      lastSynced == other.lastSynced;

  @override
  Widget build(BuildContext context) {
    final l = AppLocalizations.of(context);
    final c = Theme.of(context).extension<LsColors>()!;
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
          lastSynced: lastSynced,
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
