import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/app/app_controller.dart';
import 'package:lemusync/bridge/link/calls.dart';
import 'package:lemusync/bridge/link/model.dart' as dto;
import 'package:lemusync/design/components/progress_ring.dart';
import 'package:lemusync/design/icons.dart';
import 'package:lemusync/screens/home/home_screen.dart';

import '../../support/fake_link.dart';
import '../../support/fonts.dart';
import '../harness.dart';

class _SlowLink extends FakeDaemonLink {
  Completer<SyncNow> gate = Completer<SyncNow>();

  @override
  Future<SyncNow> syncNow() {
    syncNowCalls++;
    return gate.future;
  }
}

void main() {
  setUpAll(loadBrandFonts);

  late FakeDaemonLink link;
  late AppController c;

  void make([FakeDaemonLink? l]) {
    link = l ?? FakeDaemonLink();
    c = AppController(link)..start();
  }

  tearDown(() => c.dispose());

  Future<void> pumpHome(
    WidgetTester t, {
    String locale = 'en',
    double scale = 1,
    double width = 390,
    Color? unused,
  }) => t.pumpWidget(
    harness(
      c,
      HomeScreen(clock: () => now),
      locale: locale,
      scale: scale,
      width: width,
    ),
  );

  testWidgets('online synced: hero, last-sync caption, recent', (t) async {
    make();
    await pumpHome(t);
    await feed(
      t,
      link,
      st: status(
        dto.StateDto.synced,
        lastSyncMs: ago(const Duration(minutes: 5)),
      ),
      recent: [
        recentGame('zelda', name: 'Zelda', ms: ago(const Duration(hours: 2))),
      ],
    );
    expect(find.text('All saves synced'), findsOneWidget);
    expect(find.text('Last sync 5 min ago'), findsOneWidget);
    expect(find.text('Sync now'), findsOneWidget);
    expect(find.text('Recent games'), findsOneWidget);
    expect(find.text('Zelda'), findsOneWidget);
  });

  testWidgets('last-sync caption in Spanish', (t) async {
    make();
    await pumpHome(t, locale: 'es');
    await feed(
      t,
      link,
      st: status(
        dto.StateDto.synced,
        lastSyncMs: ago(const Duration(hours: 2)),
      ),
    );
    expect(find.text('Última sincronización hace 2 horas'), findsOneWidget);
  });

  testWidgets('syncing: the hero has a progress ring, no caption', (t) async {
    make();
    await pumpHome(t);
    await feed(
      t,
      link,
      st: status(
        dto.StateDto.syncing,
        uploads: 2,
        lastSyncMs: ago(const Duration(minutes: 5)),
      ),
    );
    expect(find.byType(ProgressRing), findsOneWidget);
    expect(find.text('Uploading 2 changes'), findsOneWidget);
    expect(find.textContaining('Last sync'), findsNothing);
  });

  testWidgets('playing: the detail appears exactly once', (t) async {
    make();
    await pumpHome(t);
    await feed(t, link, st: status(dto.StateDto.synced), sess: session());
    expect(find.text('Playing BotW · Daniel'), findsOneWidget);
  });

  testWidgets('playing in Spanish', (t) async {
    make();
    await pumpHome(t, locale: 'es');
    await feed(t, link, st: status(dto.StateDto.synced), sess: session());
    expect(find.text('Jugando BotW · Daniel'), findsOneWidget);
  });

  testWidgets('syncing with a session shows the separate playing line', (
    t,
  ) async {
    make();
    await pumpHome(t);
    await feed(
      t,
      link,
      st: status(dto.StateDto.syncing, uploads: 1),
      sess: session(),
    );
    expect(find.text('Playing BotW · Daniel'), findsOneWidget);
    expect(find.text('Uploading 1 change'), findsOneWidget);
  });

  testWidgets('attention: a banner and a command per fact', (t) async {
    make();
    await pumpHome(t);
    await feed(
      t,
      link,
      st: status(dto.StateDto.attention, conflicts: 2, leases: 1),
    );
    expect(find.text('2 conflicts need you'), findsOneWidget);
    expect(find.text('1 game is in use on another device'), findsOneWidget);
    expect(find.text('lemusync conflicts'), findsOneWidget);
    expect(find.text('lemusync lease status'), findsOneWidget);
    expect(find.text('You can review them with:'), findsOneWidget);
    expect(find.text('You can see who has them with:'), findsOneWidget);
  });

  testWidgets('error: text by kind and the doctor command', (t) async {
    make();
    await pumpHome(t);
    await feed(
      t,
      link,
      st: status(dto.StateDto.error, error: dto.ErrorKindDto.quota),
    );
    expect(find.text('Your storage is full'), findsOneWidget);
    expect(find.text('See what is wrong with:'), findsOneWidget);
    expect(find.text('lemusync doctor'), findsOneWidget);
    expect(find.text('Sync now'), findsNothing);
  });

  testWidgets('offline daemon without cache: start command', (t) async {
    make();
    await pumpHome(t);
    link.emit(const dto.LinkEvent.connection(dto.ConnectionState.notRunning()));
    await t.pump();
    expect(find.text('lemusync service start'), findsOneWidget);
  });

  testWidgets('offline with cached status: hero stays, stale banner', (
    t,
  ) async {
    make();
    await pumpHome(t);
    await feed(t, link, st: status(dto.StateDto.synced));
    link.emit(const dto.LinkEvent.connection(dto.ConnectionState.notRunning()));
    await t.pump();
    expect(find.text('All saves synced'), findsOneWidget);
    expect(find.text('Showing the last known state'), findsOneWidget);
    expect(find.text('Sync now'), findsNothing);
  });

  testWidgets('not set up: init and join', (t) async {
    make();
    await pumpHome(t);
    await feed(
      t,
      link,
      st: status(dto.StateDto.error, error: dto.ErrorKindDto.notSetUp),
    );
    expect(find.text('Set up LemuSync'), findsOneWidget);
    expect(find.text('lemusync init'), findsOneWidget);
    expect(find.text('lemusync join'), findsOneWidget);
    expect(find.text('Sync now'), findsNothing);
  });

  group('Sync now', () {
    testWidgets('one tap, pending state, enabled again after', (t) async {
      final slow = _SlowLink();
      make(slow);
      await pumpHome(t);
      await feed(t, slow, st: status(dto.StateDto.synced));
      expect(find.text('Sync now'), findsOneWidget);

      await t.tap(find.text('Sync now'));
      await t.pump();
      expect(slow.syncNowCalls, 1);
      expect(find.text('Starting sync…'), findsOneWidget);
      expect(find.text('Sync now'), findsNothing);
      expect(find.bySemanticsLabel('Starting sync…'), findsOneWidget);
      final button = t.widget<FilledButton>(find.byType(FilledButton));
      expect(button.onPressed, isNull);

      await t.tap(find.byType(FilledButton), warnIfMissed: false);
      await t.pump();
      expect(slow.syncNowCalls, 1);

      slow.gate.complete(const SyncNow.done());
      await t.pump();
      await t.pump();
      expect(find.text('Sync now'), findsOneWidget);
      expect(
        t.widget<FilledButton>(find.byType(FilledButton)).onPressed,
        isNotNull,
      );
    });
  });

  group('recent games', () {
    testWidgets('order follows the list, no Play control', (t) async {
      make();
      await pumpHome(t);
      await feed(
        t,
        link,
        st: status(dto.StateDto.synced),
        recent: [
          recentGame('b', name: 'Bravo', ms: ago(const Duration(minutes: 3))),
          recentGame('a', name: 'Alpha', ms: ago(const Duration(days: 1))),
          recentGame('c', ms: null, launchable: false),
        ],
      );
      final bravo = t.getTopLeft(find.text('Bravo')).dy;
      final alpha = t.getTopLeft(find.text('Alpha')).dy;
      final key = t.getTopLeft(find.text('c')).dy;
      expect(bravo, lessThan(alpha));
      expect(alpha, lessThan(key));
      expect(find.text('Play'), findsNothing);
      expect(find.byIcon(LsIcons.play), findsNothing);
      expect(find.text('No game file on this device'), findsOneWidget);
      expect(find.textContaining('3 min ago'), findsOneWidget);
      expect(find.textContaining('1 day ago'), findsOneWidget);
    });

    testWidgets('shows at most 8', (t) async {
      make();
      await pumpHome(t);
      await feed(
        t,
        link,
        st: status(dto.StateDto.synced),
        recent: [for (var i = 0; i < 12; i++) recentGame('g$i', name: 'G$i')],
      );
      await t.scrollUntilVisible(find.text('G7'), 300);
      expect(find.text('G7'), findsOneWidget);
      expect(find.text('G8', skipOffstage: false), findsNothing);
    });

    testWidgets('empty list', (t) async {
      make();
      await pumpHome(t);
      await feed(t, link, st: status(dto.StateDto.synced));
      expect(find.text('Games you play will show here.'), findsOneWidget);
    });
  });

  testWidgets('semantics: hero, banners and rows have labels', (t) async {
    final h = t.ensureSemantics();
    make();
    await pumpHome(t);
    await feed(
      t,
      link,
      st: status(dto.StateDto.attention, conflicts: 1),
      recent: [
        recentGame('zelda', name: 'Zelda', ms: ago(const Duration(hours: 2))),
      ],
    );
    expect(find.bySemanticsLabel('Needs your attention'), findsWidgets);
    expect(find.bySemanticsLabel(RegExp('1 conflict needs you')), findsWidgets);
    expect(
      find.bySemanticsLabel(RegExp('Zelda.*ryujinx', dotAll: true)),
      findsOneWidget,
    );
    h.dispose();
  });

  for (final MapEntry(key: name, value: colors) in themes.entries) {
    for (final width in [390.0, 320.0]) {
      testWidgets('200% text at ${width.toInt()} in $name: no overflow', (
        t,
      ) async {
        final h = t.ensureSemantics();
        make();
        await t.pumpWidget(
          harness(
            c,
            HomeScreen(clock: () => now),
            colors: colors,
            scale: 2,
            width: width,
            locale: 'es',
          ),
        );
        await feed(
          t,
          link,
          st: status(
            dto.StateDto.attention,
            conflicts: 2,
            leases: 3,
            lastSyncMs: ago(const Duration(minutes: 5)),
          ),
          sess: session(),
          recent: [
            recentGame(
              'zelda',
              name: 'The Legend of Zelda: Breath of the Wild',
              ms: ago(const Duration(hours: 2)),
              launchable: false,
            ),
          ],
        );
        expect(t.takeException(), isNull);
        await expectLater(t, meetsGuideline(textContrastGuideline));
        await expectLater(t, meetsGuideline(androidTapTargetGuideline));
        h.dispose();
      });
    }
  }
}
