import 'dart:async';

import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/app/daemon_link.dart';
import 'package:lemusync/bridge/api/daemon.dart';
import 'package:lemusync/bridge/link/calls.dart';
import 'package:lemusync/bridge/link/model.dart' as dto;

/// The contract Task 5's fake must satisfy: it compiles only while the
/// interface keeps these four members.
class _FakeLink implements DaemonLink {
  @override
  Stream<dto.LinkEvent> events() => const Stream.empty();

  @override
  Future<dto.Snapshot> snapshot() async => const dto.Snapshot(
    connection: dto.ConnectionState.notRunning(),
    players: [],
  );

  @override
  Future<dto.GamesDto?> games() async => null;

  @override
  Future<SyncNow> syncNow() async => const SyncNow.notRunning();
}

/// A [WatchHandle] without the native library: records `stop` and serves the
/// stream the test controls.
class _FakeHandle implements WatchHandle {
  _FakeHandle(this.source);

  final StreamController<dto.LinkEvent> source;
  int stops = 0;

  @override
  Stream<dto.LinkEvent> watch() => source.stream;

  @override
  Future<void> stop() async => stops++;

  @override
  dynamic noSuchMethod(Invocation invocation) => null;
}

void main() {
  test('FrbDaemonLink and a hand-written fake are both a DaemonLink', () {
    expect(_FakeLink(), isA<DaemonLink>());
    final DaemonLink real = FrbDaemonLink(
      newHandle: () => _FakeHandle(StreamController<dto.LinkEvent>()),
    );
    expect(real, isA<DaemonLink>());
  });

  test('events() starts the watch on listen and forwards its events', () async {
    final source = StreamController<dto.LinkEvent>();
    final handle = _FakeHandle(source);
    final link = FrbDaemonLink(newHandle: () => handle);
    final got = <dto.LinkEvent>[];
    final sub = link.events().listen(got.add);
    source.add(
      const dto.LinkEvent.connection(dto.ConnectionState.notRunning()),
    );
    await pumpEventQueue();
    expect(got, hasLength(1));
    await sub.cancel();
    await source.close();
  });

  test('cancelling the subscription calls WatchHandle.stop once', () async {
    final source = StreamController<dto.LinkEvent>();
    final handle = _FakeHandle(source);
    final link = FrbDaemonLink(newHandle: () => handle);
    final sub = link.events().listen((_) {});
    await pumpEventQueue();
    expect(handle.stops, 0);
    await sub.cancel();
    expect(handle.stops, 1);
    await source.close();
  });

  test('nothing starts before someone listens', () {
    var made = 0;
    FrbDaemonLink(
      newHandle: () {
        made++;
        return _FakeHandle(StreamController<dto.LinkEvent>());
      },
    ).events();
    expect(made, 0);
  });
}
