import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/app/app_controller.dart';
import 'package:lemusync/app/app_scope.dart';
import 'package:lemusync/bridge/link/calls.dart';
import 'package:lemusync/bridge/link/model.dart' as dto;

import '../support/fake_link.dart';
import '../support/fonts.dart';

dto.StatusDto st(dto.StateDto s) => dto.StatusDto(
  state: s,
  uploads: 0,
  downloads: 0,
  bytes: BigInt.zero,
  queued: 0,
  conflicts: 0,
  foreignLeases: 0,
);

dto.ProgressDto progress(String e, String g) => dto.ProgressDto(
  player: 'p',
  emulator: e,
  game: g,
  phase: dto.PhaseDto.uploading,
  filesDone: 1,
  filesTotal: 2,
  bytesDone: BigInt.one,
  bytesTotal: BigInt.two,
);

const online = dto.ConnectionState.online(daemonVersion: '1');
const entry = dto.GameEntry(
  emulator: 'e',
  key: 'k',
  launchable: true,
  streams: [],
);
const recent = dto.RecentDto(
  emulator: 'e',
  key: 'k',
  player: 'p',
  launchable: true,
);
const games = dto.GamesDto(games: [entry], recent: [recent]);
final session = dto.SessionDto(
  game: 'k',
  emulator: 'e',
  player: 'p',
  device: 'd',
  startedMs: BigInt.one,
);

Future<void> pump() => Future<void>.delayed(Duration.zero);

void main() {
  late FakeDaemonLink link;
  late AppController c;

  setUp(() {
    link = FakeDaemonLink();
    c = AppController(link)..start();
  });
  tearDown(() => c.dispose());

  test('Snapshot populates all fields', () async {
    link.emit(
      dto.LinkEvent.snapshot(
        dto.Snapshot(
          connection: online,
          status: st(dto.StateDto.synced),
          session: session,
          device: 'dev',
          players: const ['p'],
        ),
      ),
    );
    await pump();
    expect(c.state.online, isTrue);
    expect(c.state.status, isNotNull);
    expect(c.state.session, session);
    expect(c.state.device, 'dev');
    expect(c.state.players, ['p']);
    expect(c.state.loaded, isTrue);
  });

  test(
    'Progress adds a live transfer, a non-syncing Status clears it',
    () async {
      link.emit(dto.LinkEvent.status(st(dto.StateDto.syncing)));
      link.emit(dto.LinkEvent.progress(progress('e', 'g')));
      await pump();
      expect(c.state.liveTransfers, {'e/g'});
      link.emit(dto.LinkEvent.status(st(dto.StateDto.syncing)));
      await pump();
      expect(c.state.liveTransfers, {'e/g'});
      link.emit(dto.LinkEvent.status(st(dto.StateDto.synced)));
      await pump();
      expect(c.state.liveTransfers, isEmpty);
    },
  );

  test('a repeated identical Status notifies once', () async {
    var n = 0;
    c.addListener(() => n++);
    link.emit(dto.LinkEvent.status(st(dto.StateDto.synced)));
    link.emit(dto.LinkEvent.status(st(dto.StateDto.synced)));
    await pump();
    expect(n, 1);
  });

  test('Connection(NotRunning) keeps status and games', () async {
    link.emit(dto.LinkEvent.status(st(dto.StateDto.synced)));
    link.emit(const dto.LinkEvent.games(games));
    link.emit(const dto.LinkEvent.connection(dto.ConnectionState.notRunning()));
    await pump();
    expect(c.state.online, isFalse);
    expect(c.state.status, isNotNull);
    expect(c.state.games, [entry]);
  });

  test('stream error becomes NotRunning without throwing', () async {
    link.emit(const dto.LinkEvent.connection(online));
    await pump();
    link.emitError(StateError('boom'));
    await pump();
    expect(c.state.connection, isA<dto.ConnectionState_NotRunning>());
  });

  test('Session(null) clears the overlay', () async {
    link.emit(dto.LinkEvent.session(session));
    await pump();
    expect(c.state.session, session);
    link.emit(const dto.LinkEvent.session(null));
    await pump();
    expect(c.state.session, isNull);
  });

  test('Games replaces games and recent; Snapshot keeps them', () async {
    link.emit(const dto.LinkEvent.games(games));
    await pump();
    expect(c.state.games, [entry]);
    expect(c.state.recent, [recent]);
    link.emit(
      const dto.LinkEvent.snapshot(
        dto.Snapshot(connection: online, players: []),
      ),
    );
    await pump();
    expect(c.state.games, [entry]);
    expect(c.state.recent, [recent]);
  });

  test('sequence: online, offline, new snapshot', () async {
    link.emit(dto.LinkEvent.status(st(dto.StateDto.syncing)));
    link.emit(const dto.LinkEvent.games(games));
    link.emit(dto.LinkEvent.progress(progress('e', 'g')));
    link.emit(const dto.LinkEvent.connection(online));
    await pump();
    expect(c.state.liveTransfers, isNotEmpty);
    link.emit(const dto.LinkEvent.connection(dto.ConnectionState.notRunning()));
    await pump();
    expect(c.state.online, isFalse);
    expect(c.state.status, isNotNull);
    expect(c.state.games, [entry]);
    link.emit(
      dto.LinkEvent.snapshot(
        dto.Snapshot(
          connection: online,
          status: st(dto.StateDto.synced),
          players: const [],
        ),
      ),
    );
    await pump();
    expect(c.state.online, isTrue);
    expect(c.state.lsStatus!.name, 'synced');
    expect(c.state.liveTransfers, isEmpty);
    expect(c.state.games, [entry]);
  });

  test(
    'refresh applies next and nextGames; null games change nothing',
    () async {
      link.next = dto.Snapshot(
        connection: online,
        status: st(dto.StateDto.paused),
        players: const [],
      );
      link.nextGames = games;
      await c.refresh();
      expect(c.state.lsStatus!.name, 'paused');
      expect(c.state.games, [entry]);
      link.nextGames = null;
      await c.refresh();
      expect(c.state.games, [entry]);
      expect(link.snapshotCalls, 2);
      expect(link.gamesCalls, 2);
    },
  );

  test('syncNow forwards the result', () async {
    link.nextSyncNow = const SyncNow.timedOut();
    expect(await c.syncNow(), isA<SyncNow_TimedOut>());
    expect(link.syncNowCalls, 1);
  });

  test('dispose cancels the subscription', () async {
    expect(link.hasListener, isTrue);
    c.dispose();
    await pump();
    expect(link.hasListener, isFalse);
    link.emit(dto.LinkEvent.status(st(dto.StateDto.synced)));
    await pump();
  });

  group('AppScope', () {
    setUpAll(loadBrandFonts);

    testWidgets('refreshes on resume only', (tester) async {
      final l = FakeDaemonLink();
      final ctl = AppController(l)..start();
      addTearDown(ctl.dispose);
      await tester.pumpWidget(
        AppScope(
          controller: ctl,
          child: const Directionality(
            textDirection: TextDirection.ltr,
            child: SizedBox(),
          ),
        ),
      );
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.paused);
      await tester.pump();
      expect(l.snapshotCalls, 0);
      tester.binding.handleAppLifecycleStateChanged(AppLifecycleState.resumed);
      await tester.pump();
      expect(l.snapshotCalls, 1);
      expect(l.gamesCalls, 1);
    });
  });
}
