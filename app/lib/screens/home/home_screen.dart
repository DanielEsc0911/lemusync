import 'package:flutter/material.dart';

import '../../app/app_scope.dart';
import '../../app/app_state.dart';
import '../../design/components/empty_state.dart';
import '../../design/components/ls_banner.dart';
import '../../design/components/status_hero.dart';
import '../../design/status.dart';
import '../../design/tokens/colors.g.dart';
import '../../design/tokens/size.g.dart';
import '../../design/tokens/space.g.dart';
import '../../design/tokens/type.g.dart';
import '../../l10n/generated/app_localizations.dart';
import '../command_line.dart';
import '../connection_gate.dart';
import 'recent_games.dart';
import 'status_mapper.dart';

/// The current time; a parameter so tests are exact.
typedef Clock = DateTime Function();

/// Home: the live status hero, what needs attention, and recent games.
/// Read-only except Sync now.
class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key, this.clock = DateTime.now});

  final Clock clock;

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  bool _syncPending = false;

  Future<void> _syncNow() async {
    if (_syncPending) return;
    setState(() => _syncPending = true);
    try {
      await AppScope.of(context).syncNow();
    } finally {
      if (mounted) setState(() => _syncPending = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final controller = AppScope.of(context);
    final state = controller.state;
    final l = AppLocalizations.of(context);
    final c = Theme.of(context).extension<LsColors>()!;
    final now = widget.clock();
    final status = state.status;

    return ConnectionGate(
      child: Align(
        alignment: Alignment.topCenter,
        child: ConstrainedBox(
          constraints: const BoxConstraints(maxWidth: LsSize.breakpointMedium),
          child: CustomScrollView(
            slivers: [
              SliverPadding(
                padding: const EdgeInsets.all(LsSpace.lg),
                sliver: SliverMainAxisGroup(
                  slivers: [
                    SliverList.list(
                      children: [
                        if (state.notSetUp)
                          ..._notSetUp(l)
                        else if (status != null)
                          ..._live(state, l, c, now),
                      ],
                    ),
                    if (state.notSetUp)
                      const SliverToBoxAdapter(child: SizedBox.shrink())
                    else if (state.recent.isEmpty)
                      const SliverToBoxAdapter(child: RecentGamesEmpty())
                    else
                      RecentGamesSliver(recent: state.recent, now: now),
                  ],
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  List<Widget> _notSetUp(AppLocalizations l) => [
    EmptyState(title: l.homeNotSetUpTitle, body: l.homeNotSetUpBody),
    const _Commands(['lemusync init', 'lemusync join']),
  ];

  List<Widget> _live(
    AppState state,
    AppLocalizations l,
    LsColors c,
    DateTime now,
  ) {
    final status = state.status!;
    final hero = heroFor(state, l);
    final lsStatus = hero.status;
    final canSync = state.online && lsStatus != LsStatus.error;
    final lastSync = status.lastSyncMs;
    final session = state.session;
    // Attention shows each fact as its own banner below, so the hero does not
    // repeat them.
    final detail = lsStatus == LsStatus.attention ? null : hero.detail;

    return [
      RepaintBoundary(
        child: StatusHero(
          status: lsStatus,
          detail: detail,
          uploads: hero.uploads,
          downloads: hero.downloads,
          onSync: canSync ? _syncNow : null,
          syncPending: _syncPending,
        ),
      ),
      if (lsStatus == LsStatus.synced && lastSync != null)
        _Caption(l.homeLastSync(relative(l, now, lastSync.toInt()))),
      if (session != null && !state.playing)
        _Caption(
          l.homePlaying(gameTitle(session.name, session.game), session.player),
        ),
      if (lsStatus == LsStatus.error) ...[
        const SizedBox(height: LsSpace.md),
        _Lead(l.homeErrorCli),
        const _Commands(['lemusync doctor']),
      ],
      if (status.conflicts > 0) ...[
        const SizedBox(height: LsSpace.md),
        LsBanner(
          tone: LsStatus.attention,
          message: l.homeConflicts(status.conflicts),
        ),
        _Lead(l.homeConflictsCli),
        const _Commands(['lemusync conflicts']),
      ],
      if (status.foreignLeases > 0) ...[
        const SizedBox(height: LsSpace.md),
        LsBanner(
          tone: LsStatus.attention,
          message: l.homeLeases(status.foreignLeases),
        ),
        _Lead(l.homeLeasesCli),
        const _Commands(['lemusync lease status']),
      ],
    ];
  }
}

class _Caption extends StatelessWidget {
  const _Caption(this.text);

  final String text;

  @override
  Widget build(BuildContext context) {
    final c = Theme.of(context).extension<LsColors>()!;
    return Padding(
      padding: const EdgeInsets.only(top: LsSpace.sm),
      child: Text(
        text,
        style: LsType.caption.copyWith(color: c.onSurfaceMuted),
      ),
    );
  }
}

class _Lead extends StatelessWidget {
  const _Lead(this.text);

  final String text;

  @override
  Widget build(BuildContext context) {
    final c = Theme.of(context).extension<LsColors>()!;
    return Padding(
      padding: const EdgeInsets.only(top: LsSpace.sm, bottom: LsSpace.xs),
      child: Text(text, style: LsType.body.copyWith(color: c.onSurfaceMuted)),
    );
  }
}

class _Commands extends StatelessWidget {
  const _Commands(this.commands);

  final List<String> commands;

  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.stretch,
    spacing: LsSpace.xs,
    children: [for (final c in commands) CommandLine(command: c)],
  );
}
