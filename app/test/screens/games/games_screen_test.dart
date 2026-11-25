import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/app/app_controller.dart';
import 'package:lemusync/app/app_state.dart';
import 'package:lemusync/bridge/link/model.dart' as dto;
import 'package:lemusync/design/components/game_card.dart';
import 'package:lemusync/design/status.dart';
import 'package:lemusync/design/tokens/size.g.dart';
import 'package:lemusync/design/tokens/space.g.dart';
import 'package:lemusync/screens/command_line.dart';
import 'package:lemusync/screens/games/games_screen.dart';

import '../../support/fake_link.dart';
import '../../support/fonts.dart';
import '../harness.dart';

dto.GameEntry entry(
  String key, {
  String? name,
  String emulator = 'ryujinx',
  bool launchable = true,
  List<int?> streams = const [],
}) => dto.GameEntry(
  emulator: emulator,
  key: key,
  name: name,
  launchable: launchable,
  streams: [
    for (var i = 0; i < streams.length; i++)
      dto.PlayerStreamDto(
        player: 'p$i',
        lastVersionMs: streams[i] == null ? null : BigInt.from(streams[i]!),
      ),
  ],
);

AppState st(dto.StateDto s, {Set<String> live = const {}}) =>
    AppState(connection: online, status: status(s), liveTransfers: live);

void main() {
  setUpAll(loadBrandFonts);

  group('cardStatus', () {
    final g = entry('BotW');
    test('live transfer wins, key ignores ASCII case', () {
      final s = st(dto.StateDto.paused, live: {'ryujinx/botw'});
      expect(s.cardStatus(g), LsStatus.syncing);
    });
    test('offline, paused, error are global', () {
      expect(st(dto.StateDto.offline).cardStatus(g), LsStatus.offline);
      expect(st(dto.StateDto.paused).cardStatus(g), LsStatus.paused);
      expect(st(dto.StateDto.error).cardStatus(g), LsStatus.error);
    });
    test('attention, scanning, synced, syncing show synced', () {
      for (final s in [
        dto.StateDto.attention,
        dto.StateDto.scanning,
        dto.StateDto.synced,
        dto.StateDto.syncing,
      ]) {
        expect(st(s).cardStatus(g), LsStatus.synced);
      }
    });
  });

  Future<void> pump(
    WidgetTester t,
    List<dto.GameEntry> games, {
    double width = 390,
    double scale = 1,
    String locale = 'en',
    dto.ConnectionState connection = online,
  }) async {
    t.view.physicalSize = Size(width, 800);
    t.view.devicePixelRatio = 1;
    addTearDown(t.view.reset);
    final link = FakeDaemonLink();
    final c = AppController(link)..start();
    addTearDown(c.dispose);
    await feed(
      t,
      link,
      connection: connection,
      st: connection == online ? status(dto.StateDto.synced) : null,
    );
    link.emit(
      dto.LinkEvent.games(dto.GamesDto(games: games, recent: const [])),
    );
    await t.pump();
    await t.pumpWidget(
      harness(
        c,
        GamesScreen(clock: () => now),
        width: width,
        scale: scale,
        locale: locale,
      ),
    );
    await t.pump();
  }

  testWidgets('one card per entry in order, no Play', (t) async {
    await pump(t, [
      entry('a', name: 'Alpha'),
      entry('b', name: 'Beta'),
      entry('c'),
    ]);
    final cards = t.widgetList<GameCard>(find.byType(GameCard)).toList();
    expect(cards.map((c) => c.title), ['Alpha', 'Beta', 'c']);
    expect(find.byType(IconButton), findsNothing);
    expect(cards.every((c) => c.onPlay == null && c.onTap == null), isTrue);
  });

  testWidgets('not launchable shows the no-file marker', (t) async {
    await pump(t, [entry('a', launchable: false)]);
    expect(find.text('No game file on this device'), findsOneWidget);
  });

  testWidgets('players and last synced from streams', (t) async {
    await pump(t, [
      entry('a', streams: [ago(const Duration(hours: 3)), null]),
    ]);
    expect(find.textContaining('2 players'), findsOneWidget);
    expect(find.textContaining('3 hours ago'), findsOneWidget);
  });

  testWidgets('empty library shows the CLI line', (t) async {
    await pump(t, const []);
    expect(find.text('No games found yet'), findsOneWidget);
    expect(find.byType(CommandLine), findsOneWidget);
    expect(find.text('lemusync games'), findsOneWidget);
  });

  testWidgets('offline daemon reuses the gate', (t) async {
    await pump(t, const [], connection: const dto.ConnectionState.notRunning());
    expect(find.text('lemusync service start'), findsOneWidget);
  });

  testWidgets('1000 entries build lazily', (t) async {
    await pump(t, [for (var i = 0; i < 1000; i++) entry('g$i')]);
    expect(find.byType(GameCard), findsWidgets);
    expect(find.byType(GameCard).evaluate().length, lessThan(40));
  });

  for (final w in [390.0, 900.0, 1400.0]) {
    testWidgets('columns at $w come from the card token', (t) async {
      await pump(t, [for (var i = 0; i < 8; i++) entry('g$i')], width: w);
      final xs = {
        for (final e in find.byType(GameCard).evaluate())
          t.getTopLeft(find.byWidget(e.widget)).dx,
      };
      final grid = t.widget<SliverGrid>(find.byType(SliverGrid));
      final d = grid.gridDelegate as SliverGridDelegateWithMaxCrossAxisExtent;
      expect(d.maxCrossAxisExtent, LsSize.cardMaxWidth);
      // Max-extent grid: columns = ceil(available / (card max + spacing)).
      final first = t.getTopLeft(find.byType(GameCard).first).dx;
      expect(first, LsSpace.lg);
      final avail = w - 2 * LsSpace.lg;
      final cols = (avail / (LsSize.cardMaxWidth + LsSpace.md)).ceil();
      expect(xs.length, cols);
    });
  }

  testWidgets('200% text at 390 does not overflow', (t) async {
    await pump(t, [entry('a', name: 'Mario Kart 8 Deluxe')], scale: 2);
    expect(t.takeException(), isNull);
    await pump(
      t,
      [entry('a', name: 'Mario Kart 8 Deluxe', launchable: false)],
      scale: 2,
      locale: 'es',
    );
    expect(t.takeException(), isNull);
  });

  testWidgets('semantics label has title, emulator and status', (t) async {
    final h = t.ensureSemantics();
    await pump(t, [entry('a', name: 'Alpha')]);
    expect(
      find.bySemanticsLabel(RegExp('^Alpha, ryujinx, All saves synced')),
      findsOneWidget,
    );
    h.dispose();
  });

  testWidgets('cards are not focusable (3c changes this)', (t) async {
    await pump(t, [entry('a'), entry('b')]);
    await t.sendKeyEvent(LogicalKeyboardKey.tab);
    await t.pump();
    final focus = FocusManager.instance.primaryFocus;
    expect(focus?.context?.findAncestorWidgetOfExactType<GameCard>(), isNull);
  });
}
