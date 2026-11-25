@Tags(['golden'])
library;

import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/app/app_controller.dart';
import 'package:lemusync/app/app_scope.dart';
import 'package:lemusync/bridge/link/model.dart' as dto;
import 'package:lemusync/design/tokens/colors.g.dart';
import 'package:lemusync/screens/home/home_screen.dart';

import '../support/fake_link.dart';
import '../support/fonts.dart';
import '../support/golden.dart';
import 'harness.dart';

void main() {
  setUpAll(loadBrandFonts);

  testWidgets('home synced high contrast', (t) async {
    final link = FakeDaemonLink();
    final c = AppController(link)..start();
    addTearDown(c.dispose);
    await feed(
      t,
      link,
      st: status(
        dto.StateDto.synced,
        lastSyncMs: ago(const Duration(minutes: 5)),
      ),
      recent: [
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
        recentGame('unknown-game', launchable: false),
      ],
    );
    await goldenMatrix(
      t,
      'home_synced_hc',
      () => Builder(
        builder: (context) => AppScope(
          controller: c,
          child: SizedBox.fromSize(
            size: MediaQuery.sizeOf(context),
            child: HomeScreen(clock: () => now),
          ),
        ),
      ),
      sizes: const [Size(390, 844)],
      locales: const [Locale('en')],
      themes: const {
        'lightHighContrast': LsColors.lightHighContrast,
        'darkHighContrast': LsColors.darkHighContrast,
      },
      precache: const [AssetImage('assets/brand/mark.png')],
    );
  }, skip: !Platform.isLinux);
}
