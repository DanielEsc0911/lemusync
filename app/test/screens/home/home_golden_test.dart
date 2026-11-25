@Tags(['golden'])
library;

import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/app/app_controller.dart';
import 'package:lemusync/app/app_scope.dart';
import 'package:lemusync/bridge/link/model.dart' as dto;
import 'package:lemusync/screens/home/home_screen.dart';

import '../../support/fake_link.dart';
import '../../support/fonts.dart';
import '../../support/golden.dart';
import '../harness.dart';

const _narrow = [Size(390, 844)];
const _all = [Size(390, 844), Size(900, 1000), Size(1400, 1000)];

void main() {
  setUpAll(loadBrandFonts);

  Future<void> shoot(
    WidgetTester t,
    String name, {
    required List<Size> sizes,
    required List<Locale> locales,
    dto.ConnectionState connection = online,
    dto.StatusDto? st,
    dto.SessionDto? sess,
    List<dto.RecentDto> recent = const [],
  }) async {
    final link = FakeDaemonLink();
    final c = AppController(link)..start();
    addTearDown(c.dispose);
    await feed(
      t,
      link,
      connection: connection,
      st: st,
      sess: sess,
      recent: recent,
    );
    await goldenMatrix(
      t,
      name,
      () => Builder(
        builder: (context) => AppScope(
          controller: c,
          child: SizedBox.fromSize(
            size: MediaQuery.sizeOf(context),
            child: HomeScreen(clock: () => now),
          ),
        ),
      ),
      sizes: sizes,
      locales: locales,
      precache: const [AssetImage('assets/brand/mark.png')],
    );
  }

  final recent = [
    recentGame(
      'botw',
      name: 'Zelda: Breath of the Wild',
      ms: ago(const Duration(minutes: 3)),
    ),
    recentGame(
      'mk8',
      name: 'Mario Kart 8 Deluxe',
      ms: ago(const Duration(hours: 5)),
      player: 'Ana',
    ),
    recentGame(
      'pkmn',
      name: 'Pokémon Escarlata',
      emulator: 'yuzu',
      ms: ago(const Duration(days: 2)),
    ),
    recentGame('unknown-game', launchable: false),
  ];
  final synced = status(
    dto.StateDto.synced,
    lastSyncMs: ago(const Duration(minutes: 5)),
  );

  testWidgets('home synced en', (t) async {
    await shoot(
      t,
      'home_synced',
      sizes: _all,
      locales: const [Locale('en')],
      st: synced,
      recent: recent,
    );
  }, skip: !Platform.isLinux);

  testWidgets('home synced es', (t) async {
    await shoot(
      t,
      'home_synced',
      sizes: _narrow,
      locales: const [Locale('es')],
      st: synced,
      recent: recent,
    );
  }, skip: !Platform.isLinux);

  testWidgets('home syncing', (t) async {
    await shoot(
      t,
      'home_syncing',
      sizes: _narrow,
      locales: const [Locale('en')],
      st: status(dto.StateDto.syncing, uploads: 3, downloads: 2),
      recent: recent,
    );
  }, skip: !Platform.isLinux);

  testWidgets('home playing', (t) async {
    await shoot(
      t,
      'home_playing',
      sizes: _narrow,
      locales: const [Locale('en')],
      st: synced,
      sess: session(),
      recent: recent,
    );
  }, skip: !Platform.isLinux);

  testWidgets('home attention', (t) async {
    await shoot(
      t,
      'home_attention',
      sizes: _narrow,
      locales: const [Locale('en')],
      st: status(dto.StateDto.attention, conflicts: 2, leases: 1),
      recent: recent,
    );
  }, skip: !Platform.isLinux);

  testWidgets('home offline daemon', (t) async {
    await shoot(
      t,
      'home_offline_daemon',
      sizes: _narrow,
      locales: const [Locale('en')],
      connection: const dto.ConnectionState.notRunning(),
    );
  }, skip: !Platform.isLinux);

  testWidgets('home not set up', (t) async {
    await shoot(
      t,
      'home_not_set_up',
      sizes: _narrow,
      locales: const [Locale('en')],
      st: status(dto.StateDto.error, error: dto.ErrorKindDto.notSetUp),
    );
  }, skip: !Platform.isLinux);
}
