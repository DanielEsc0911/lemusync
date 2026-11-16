@Tags(['golden'])
library;

import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/design/components/game_card.dart';
import 'package:lemusync/design/components/status_hero.dart';
import 'package:lemusync/design/status.dart';
import 'package:lemusync/design/tokens/space.g.dart';

import '../../support/fonts.dart';
import '../../support/golden.dart';

String _t(BuildContext c, String en, String es) =>
    Localizations.localeOf(c).languageCode == 'es' ? es : en;

void main() {
  setUpAll(loadBrandFonts);

  testWidgets('status_hero', (t) async {
    await goldenMatrix(
      t,
      'status_hero',
      () => Builder(
        builder: (c) => Padding(
          padding: const EdgeInsets.all(LsSpace.lg),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            spacing: LsSpace.md,
            children: [
              StatusHero(
                status: LsStatus.synced,
                detail: _t(
                  c,
                  'Last sync 2 min ago',
                  'Última sincronización hace 2 min',
                ),
                onSync: () {},
              ),
              StatusHero(
                status: LsStatus.syncing,
                uploads: 3,
                downloads: 2,
                detail: _t(c, 'Zelda, Mario Kart', 'Zelda, Mario Kart'),
              ),
              StatusHero(
                status: LsStatus.attention,
                detail: _t(
                  c,
                  'Choose which save to keep',
                  'Elige qué partida conservar',
                ),
                onSync: () {},
              ),
              StatusHero(
                status: LsStatus.error,
                detail: _t(
                  c,
                  'Could not reach your storage',
                  'No se pudo conectar con tu almacenamiento',
                ),
                onSync: () {},
              ),
            ],
          ),
        ),
      ),
      sizes: const [Size(390, 844), Size(900, 1000)],
    );
  }, skip: !Platform.isLinux);

  testWidgets('game_card', (t) async {
    await goldenMatrix(
      t,
      'game_card',
      () => Builder(
        builder: (c) => Padding(
          padding: const EdgeInsets.all(LsSpace.lg),
          child: Wrap(
            spacing: LsSpace.lg,
            runSpacing: LsSpace.lg,
            children: [
              SizedBox(
                width: 280,
                child: GameCard(
                  title: 'Mario Kart 8 Deluxe',
                  emulator: 'Ryujinx',
                  lastSynced: _t(c, '2 min ago', 'hace 2 min'),
                  onPlay: () {},
                ),
              ),
              SizedBox(
                width: 280,
                child: GameCard(
                  title: _t(
                    c,
                    'The Legend of Zelda: Breath of the Wild Collector Edition',
                    'La leyenda de Zelda: el aliento de lo salvaje edición coleccionista',
                  ),
                  emulator: 'Cemu',
                  status: LsStatus.attention,
                  lease: _t(c, 'In use on Desktop', 'En uso en Escritorio'),
                  onPlay: () {},
                ),
              ),
            ],
          ),
        ),
      ),
      sizes: const [Size(390, 844), Size(900, 1000), Size(1400, 1000)],
    );
  }, skip: !Platform.isLinux);
}
