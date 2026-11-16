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

  /// Replaces the source's stream when a test needs a custom one.
  Stream<dto.LinkEvent>? stream;

  @override
  Stream<dto.LinkEvent> watch() => stream ?? source.stream;

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

  test('cancel still stops the handle when the inner cancel throws', () async {
    final handle = _FakeHandle(StreamController<dto.LinkEvent>())
      ..stream = _ThrowOnCancel();
    final link = FrbDaemonLink(newHandle: () => handle);
    final sub = link.events().listen((_) {});
    await pumpEventQueue();
    await sub.cancel().then((_) {}, onError: (_) {});
    expect(handle.stops, 1);
  });

  test('a stream error reaches the listener', () async {
    final source = StreamController<dto.LinkEvent>();
    final link = FrbDaemonLink(newHandle: () => _FakeHandle(source));
    final errors = <Object>[];
    final sub = link.events().listen((_) {}, onError: errors.add);
    source.addError('boom');
    await pumpEventQueue();
    expect(errors, ['boom']);
    await sub.cancel();
    await source.close();
  });

  test('the handle stream finishing closes the returned stream', () async {
    final source = StreamController<dto.LinkEvent>();
    final link = FrbDaemonLink(newHandle: () => _FakeHandle(source));
    var done = false;
    link.events().listen((_) {}, onDone: () => done = true);
    await source.close();
    await pumpEventQueue();
    expect(done, isTrue);
  });

  test(
    'cancelling in the turn it was listened stops that handle once',
    () async {
      var made = 0;
      late _FakeHandle handle;
      final link = FrbDaemonLink(
        newHandle: () {
          made++;
          return handle = _FakeHandle(StreamController<dto.LinkEvent>());
        },
      );
      final sub = link.events().listen((_) {});
      await sub.cancel();
      expect(made, 1);
      expect(handle.stops, 1);
    },
  );

  test('two events() calls make two independent handles', () async {
    final handles = <_FakeHandle>[];
    final link = FrbDaemonLink(
      newHandle: () {
        final h = _FakeHandle(StreamController<dto.LinkEvent>());
        handles.add(h);
        return h;
      },
    );
    final a = link.events().listen((_) {});
    final b = link.events().listen((_) {});
    await pumpEventQueue();
    expect(handles, hasLength(2));
    await a.cancel();
    expect(handles[0].stops, 1);
    expect(handles[1].stops, 0);
    await b.cancel();
    expect(handles[1].stops, 1);
  });
}

/// A stream whose subscription throws when cancelled.
class _ThrowOnCancel extends Stream<dto.LinkEvent> {
  @override
  StreamSubscription<dto.LinkEvent> listen(
    void Function(dto.LinkEvent event)? onData, {
    Function? onError,
    void Function()? onDone,
    bool? cancelOnError,
  }) => _ThrowingSub();
}

class _ThrowingSub implements StreamSubscription<dto.LinkEvent> {
  @override
  Future<void> cancel() => Future.error(StateError('cancel failed'));

  @override
  dynamic noSuchMethod(Invocation invocation) => null;
}
