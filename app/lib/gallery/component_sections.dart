import 'package:flutter/material.dart';

import '../design/design.dart';
import '../design/tokens/colors.g.dart';
import '../design/tokens/space.g.dart';
import '../design/tokens/type.g.dart';
import '../l10n/generated/app_localizations.dart';

// Layout-only width of a gallery tile; not a design value.
const double _card = 300;

class SurfacesSection extends StatelessWidget {
  const SurfacesSection({super.key});

  @override
  Widget build(BuildContext context) {
    final c = Theme.of(context).extension<LsColors>()!;
    return Wrap(
      spacing: LsSpace.md,
      runSpacing: LsSpace.md,
      children: [
        for (final level in LsSurfaceLevel.values)
          SizedBox(
            width: _card,
            child: LsSurface(
              level: level,
              child: Text(
                level.name,
                style: LsType.title.copyWith(color: c.onSurface),
              ),
            ),
          ),
      ],
    );
  }
}

class BadgesSection extends StatelessWidget {
  const BadgesSection({super.key});

  @override
  Widget build(BuildContext context) => Wrap(
    spacing: LsSpace.lg,
    runSpacing: LsSpace.md,
    children: [
      for (final s in LsStatus.values) ...[
        StatusBadge(status: s),
        StatusBadge(status: s, compact: true),
      ],
    ],
  );
}

class RingsSection extends StatelessWidget {
  const RingsSection({super.key});

  @override
  Widget build(BuildContext context) => Wrap(
    spacing: LsSpace.lg,
    runSpacing: LsSpace.md,
    children: [
      const ProgressRing(value: 0.5, semanticsLabel: 'Half done'),
      const ProgressRing(),
      for (final s in LsStatus.values)
        ProgressRing(value: 0.75, status: s, semanticsLabel: s.name),
    ],
  );
}

class HeroesSection extends StatelessWidget {
  const HeroesSection({super.key});

  @override
  Widget build(BuildContext context) => Column(
    spacing: LsSpace.md,
    children: [
      for (final s in LsStatus.values)
        StatusHero(
          status: s,
          uploads: s == LsStatus.syncing ? 3 : 0,
          downloads: s == LsStatus.syncing ? 2 : 0,
          detail: s.name,
          onSync: () {},
        ),
    ],
  );
}

class BannersSection extends StatelessWidget {
  const BannersSection({super.key});

  @override
  Widget build(BuildContext context) => Column(
    spacing: LsSpace.md,
    children: [
      for (final s in LsStatus.values)
        LsBanner(
          tone: s,
          message: s.label(AppLocalizations.of(context)),
          actionLabel: AppLocalizations.of(context).actionRetry,
          onAction: () {},
        ),
    ],
  );
}

class CardsSection extends StatelessWidget {
  const CardsSection({super.key});

  GameCard _game(LsStatus s) => GameCard(
    title: 'Mario Kart 8 Deluxe',
    emulator: 'Ryujinx',
    status: s,
    lastSynced: '2 min ago',
    lease: s == LsStatus.attention ? 'In use on Desktop' : null,
    onPlay: () {},
    onTap: () {},
  );

  @override
  Widget build(BuildContext context) => Wrap(
    spacing: LsSpace.md,
    runSpacing: LsSpace.md,
    children: [
      for (final s in LsStatus.values) SizedBox(width: _card, child: _game(s)),
    ],
  );
}

class StatesSection extends StatelessWidget {
  const StatesSection({super.key});

  @override
  Widget build(BuildContext context) => Column(
    spacing: LsSpace.lg,
    children: [
      const EmptyState(title: 'No games yet', body: 'Add an emulator folder'),
      ErrorState(
        title: 'Could not reach your storage',
        body: 'Check your connection',
        onRetry: () {},
      ),
    ],
  );
}
