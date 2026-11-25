@Tags(['golden'])
library;

import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/app/app_controller.dart';
import 'package:lemusync/app/app_scope.dart';
import 'package:lemusync/bridge/link/model.dart' as dto;
import 'package:lemusync/screens/games/games_screen.dart';

import '../../support/fake_link.dart';
import '../../support/fonts.dart';
import '../../support/golden.dart';
import '../harness.dart';
import 'games_screen_test.dart' show entry;

void main() {
  setUpAll(loadBrandFonts);

  Future<void> shoot(
    WidgetTester t,
    List<Locale> locales, {
    required dto.StateDto state,
  }) async {
    final link = FakeDaemonLink();
    final c = AppController(link)..start();
    addTearDown(c.dispose);
    await feed(t, link, st: status(state));
    if (state == dto.StateDto.syncing) {
      link.emit(
        dto.LinkEvent.progress(
          dto.ProgressDto(
            player: 'p',
            emulator: 'ryujinx',
            game: 'mk8',
            phase: dto.PhaseDto.uploading,
            filesDone: 1,
            filesTotal: 2,
            bytesDone: BigInt.one,
            bytesTotal: BigInt.two,
          ),
        ),
      );
    }
    link.emit(
      dto.LinkEvent.games(
        dto.GamesDto(
          games: [
            entry(
              'botw',
              name: 'Zelda: Breath of the Wild',
              streams: [
                ago(const Duration(minutes: 3)),
                ago(const Duration(days: 1)),
              ],
            ),
            entry(
              'mk8',
              name: 'Mario Kart 8 Deluxe',
              streams: [ago(const Duration(hours: 5))],
            ),
            entry('unknown-game', launchable: false),
            entry(
              'pkmn',
              name: 'Pokémon Escarlata',
              emulator: 'yuzu',
              streams: [ago(const Duration(days: 2))],
            ),
            entry('smo', name: 'Super Mario Odyssey', streams: [null]),
            entry(
              'metroid',
              name: 'Metroid Dread',
              streams: [ago(const Duration(minutes: 40))],
            ),
          ],
          recent: const [],
        ),
      ),
    );
    await t.pump();

    await goldenMatrix(
      t,
      'games_grid',
      () => Builder(
        builder: (context) => AppScope(
          controller: c,
          child: SizedBox.fromSize(
            size: MediaQuery.sizeOf(context),
            child: GamesScreen(clock: () => now),
          ),
        ),
      ),
      sizes: const [Size(900, 1000)],
      locales: locales,
      precache: const [AssetImage('assets/brand/mark.png')],
    );
  }

  testWidgets('games grid en', (t) async {
    await shoot(t, const [Locale('en')], state: dto.StateDto.syncing);
  }, skip: !Platform.isLinux);

  testWidgets('games grid es', (t) async {
    await shoot(t, const [Locale('es')], state: dto.StateDto.offline);
  }, skip: !Platform.isLinux);
}
