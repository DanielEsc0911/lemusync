import 'package:flutter/material.dart';

import '../../app/app_scope.dart';
import '../../design/components/empty_state.dart';
import '../../design/tokens/space.g.dart';
import '../../l10n/generated/app_localizations.dart';
import '../command_line.dart';
import '../connection_gate.dart';
import '../home/home_screen.dart' show Clock;
import 'game_grid.dart';

/// Games: the library as cards, read-only (Play and details are 3c).
class GamesScreen extends StatelessWidget {
  const GamesScreen({super.key, this.clock = DateTime.now});

  final Clock clock;

  @override
  Widget build(BuildContext context) {
    final state = AppScope.of(context).state;
    final l = AppLocalizations.of(context);
    return ConnectionGate(
      child: state.games.isEmpty
          ? Center(
              child: SingleChildScrollView(
                padding: const EdgeInsets.all(LsSpace.lg),
                child: Column(
                  mainAxisSize: MainAxisSize.min,
                  crossAxisAlignment: CrossAxisAlignment.stretch,
                  children: [
                    EmptyState(
                      title: l.gamesEmptyTitle,
                      body: l.gamesEmptyBody,
                    ),
                    const CommandLine(command: 'lemusync games'),
                  ],
                ),
              ),
            )
          : CustomScrollView(
              slivers: [
                SliverPadding(
                  padding: const EdgeInsets.all(LsSpace.lg),
                  sliver: GameGridSliver(state: state, now: clock()),
                ),
              ],
            ),
    );
  }
}
