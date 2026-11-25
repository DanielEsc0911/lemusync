import 'package:flutter/rendering.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/app/app_controller.dart';
import 'package:lemusync/bridge/link/model.dart' as dto;
import 'package:lemusync/screens/games/games_screen.dart';
import 'package:lemusync/screens/home/home_screen.dart';

import '../support/fake_link.dart';
import '../support/fonts.dart';
import 'games/games_screen_test.dart' show entry;
import 'harness.dart';

/// Every screen state of Tasks 8 and 9 at 200 % text and 320 px, in all four
/// themes, plus semantic order and reduce motion on Home.
void main() {
  setUpAll(loadBrandFonts);

  late FakeDaemonLink link;
  late AppController c;

  void make() {
    link = FakeDaemonLink();
    c = AppController(link)..start();
    addTearDown(c.dispose);
  }

  final recent = [
    recentGame(
      'zelda',
      name: 'The Legend of Zelda: Breath of the Wild',
      ms: ago(const Duration(hours: 2)),
    ),
    recentGame('unknown-game', launchable: false),
  ];
  final games = [
    entry('a', name: 'Mario Kart 8 Deluxe', streams: [1700000000000 - 60000]),
    entry('b', name: 'The Legend of Zelda: Breath of the Wild'),
    entry('c', launchable: false),
  ];

  final home = <String, Future<void> Function(WidgetTester)>{
    'synced': (t) => feed(
      t,
      link,
      st: status(
        dto.StateDto.synced,
        lastSyncMs: ago(const Duration(minutes: 5)),
      ),
      recent: recent,
    ),
    'syncing': (t) => feed(
      t,
      link,
      st: status(dto.StateDto.syncing, uploads: 3, downloads: 2),
      recent: recent,
    ),
    'playing': (t) => feed(
      t,
      link,
      st: status(dto.StateDto.synced),
      sess: session(),
      recent: recent,
    ),
    'attention': (t) => feed(
      t,
      link,
      st: status(dto.StateDto.attention, conflicts: 2, leases: 3),
      recent: recent,
    ),
    'offline daemon': (t) =>
        feed(t, link, connection: const dto.ConnectionState.notRunning()),
    'not set up': (t) => feed(
      t,
      link,
      st: status(dto.StateDto.error, error: dto.ErrorKindDto.notSetUp),
    ),
    'failed': (t) => feed(
      t,
      link,
      connection: const dto.ConnectionState.failed(
        code: 'E_IO',
        message: 'connection reset by peer while reading the status',
      ),
    ),
    'unsafe': (t) =>
        feed(t, link, connection: const dto.ConnectionState.unsafe()),
  };

  for (final MapEntry(key: theme, value: colors) in themes.entries) {
    for (final MapEntry(key: name, value: setup) in home.entries) {
      testWidgets('Home $name at 200% and 320 in $theme: no overflow', (
        t,
      ) async {
        make();
        await t.pumpWidget(
          harness(
            c,
            HomeScreen(clock: () => now),
            colors: colors,
            scale: 2,
            width: 320,
            locale: 'es',
          ),
        );
        await setup(t);
        await t.pump();
        expect(t.takeException(), isNull);
      });
    }

    final gamesStates = <String, Future<void> Function(WidgetTester)>{
      'grid': (t) async {
        await feed(t, link, st: status(dto.StateDto.synced));
        link.emit(
          dto.LinkEvent.games(dto.GamesDto(games: games, recent: const [])),
        );
      },
      'empty': (t) => feed(t, link, st: status(dto.StateDto.synced)),
      'offline': (t) =>
          feed(t, link, connection: const dto.ConnectionState.notRunning()),
    };
    for (final MapEntry(key: name, value: setup) in gamesStates.entries) {
      testWidgets('Games $name at 200% and 320 in $theme: no overflow', (
        t,
      ) async {
        make();
        await t.pumpWidget(
          harness(
            c,
            GamesScreen(clock: () => now),
            colors: colors,
            scale: 2,
            width: 320,
            locale: 'es',
          ),
        );
        await setup(t);
        await t.pump();
        expect(t.takeException(), isNull);
      });
    }
  }

  testWidgets('Home semantic order: banner, hero, attention, recent', (
    t,
  ) async {
    final h = t.ensureSemantics();
    make();
    await t.pumpWidget(harness(c, HomeScreen(clock: () => now)));
    // Offline with a cached attention status: the stale banner shows.
    await feed(
      t,
      link,
      st: status(dto.StateDto.attention, conflicts: 1),
      recent: recent,
    );
    link.emit(const dto.LinkEvent.connection(dto.ConnectionState.notRunning()));
    await t.pump();

    final labels = <String>[];
    void walk(SemanticsNode n) {
      if (n.label.isNotEmpty) labels.add(n.label);
      for (final k in n.debugListChildrenInOrder(
        DebugSemanticsDumpOrder.traversalOrder,
      )) {
        walk(k);
      }
    }

    walk(t.binding.renderViews.first.owner!.semanticsOwner!.rootSemanticsNode!);
    int at(Pattern p) => labels.indexWhere((l) => l.contains(p));
    final order = [
      at('Showing the last known state'),
      at('Needs your attention'),
      at('1 conflict needs you'),
      at('Recent games'),
    ];
    expect(order, everyElement(isNonNegative), reason: '$labels');
    expect(order, [...order]..sort(), reason: '$labels');
    h.dispose();
  });

  testWidgets('reduce motion: Home syncing runs no animation', (t) async {
    make();
    await t.pumpWidget(
      harness(c, HomeScreen(clock: () => now), disableAnimations: true),
    );
    await feed(
      t,
      link,
      st: status(dto.StateDto.syncing, uploads: 1, downloads: 1),
      recent: recent,
    );
    await t.pump();
    expect(t.hasRunningAnimations, isFalse);
  });
}
