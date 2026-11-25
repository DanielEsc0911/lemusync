import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/app/app_controller.dart';
import 'package:lemusync/bridge/link/model.dart' as dto;
import 'package:lemusync/design/components/game_card.dart';
import 'package:lemusync/design/components/status_hero.dart';
import 'package:lemusync/screens/games/games_screen.dart';
import 'package:lemusync/screens/home/home_screen.dart';

import '../screens/games/games_screen_test.dart' show entry;
import '../screens/harness.dart';
import '../support/fake_link.dart';
import '../support/fonts.dart';

/// Counts rebuilds by widget type while [body] runs.
Future<Map<Type, int>> _count(Future<void> Function() body) async {
  final counts = <Type, int>{};
  final previous = debugOnRebuildDirtyWidget;
  debugOnRebuildDirtyWidget = (element, builtOnce) {
    counts.update(element.widget.runtimeType, (n) => n + 1, ifAbsent: () => 1);
  };
  try {
    await body();
  } finally {
    debugOnRebuildDirtyWidget = previous;
  }
  return counts;
}

dto.ProgressDto _progress(String game, int done) => dto.ProgressDto(
  player: 'Daniel',
  emulator: 'ryujinx',
  game: game,
  phase: dto.PhaseDto.uploading,
  filesDone: done,
  filesTotal: 500,
  bytesDone: BigInt.from(done),
  bytesTotal: BigInt.from(500),
);

void main() {
  setUpAll(loadBrandFonts);

  late FakeDaemonLink link;
  late AppController c;

  Future<void> start(WidgetTester t, Widget screen) async {
    t.view.physicalSize = const Size(1200, 900);
    t.view.devicePixelRatio = 1;
    addTearDown(t.view.reset);
    link = FakeDaemonLink();
    c = AppController(link)..start();
    addTearDown(c.dispose);
    await feed(t, link, st: status(dto.StateDto.syncing, uploads: 1));
    link.emit(
      dto.LinkEvent.games(
        dto.GamesDto(
          games: [for (var i = 0; i < 40; i++) entry('g$i', name: 'Game $i')],
          recent: [for (var i = 0; i < 8; i++) recentGame('g$i', name: 'G$i')],
        ),
      ),
    );
    await t.pump();
    await t.pumpWidget(harness(c, screen, width: 1200, height: 900));
    await t.pump();
  }

  testWidgets('a repeated identical StatusChanged rebuilds nothing', (t) async {
    await start(t, HomeScreen(clock: () => now));
    final same = status(dto.StateDto.syncing, uploads: 1);
    final counts = await _count(() async {
      for (var i = 0; i < 20; i++) {
        link.emit(dto.LinkEvent.status(same));
        await t.pump();
      }
    });
    expect(counts, isEmpty);
  });

  testWidgets('200 Progress events: Games rebuilds one card at most', (
    t,
  ) async {
    await start(t, GamesScreen(clock: () => now));
    expect(t.widgetList(find.byType(GameCard)).length, greaterThan(4));
    final counts = await _count(() async {
      for (var i = 0; i < 200; i++) {
        link.emit(dto.LinkEvent.progress(_progress('g1', i)));
        await t.pump();
      }
    });
    expect(counts[GameCard] ?? 0, lessThanOrEqualTo(1), reason: '$counts');
  });

  testWidgets('200 Progress events: Home rebuilds the hero at most once', (
    t,
  ) async {
    await start(t, HomeScreen(clock: () => now));
    final counts = await _count(() async {
      for (var i = 0; i < 200; i++) {
        link.emit(dto.LinkEvent.progress(_progress('g1', i)));
        await t.pump();
      }
    });
    expect(counts[StatusHero] ?? 0, lessThanOrEqualTo(1), reason: '$counts');
  });
}
