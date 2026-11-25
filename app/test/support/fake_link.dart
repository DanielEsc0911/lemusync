import 'dart:async';

import 'package:lemusync/app/daemon_link.dart';
import 'package:lemusync/bridge/link/calls.dart';
import 'package:lemusync/bridge/link/model.dart' as dto;

/// A [DaemonLink] for tests: never loads the native library.
class FakeDaemonLink implements DaemonLink {
  final StreamController<dto.LinkEvent> _events =
      StreamController<dto.LinkEvent>.broadcast();

  dto.Snapshot next = const dto.Snapshot(
    connection: dto.ConnectionState.notRunning(),
    players: [],
  );
  dto.GamesDto? nextGames;
  SyncNow nextSyncNow = const SyncNow.done();

  int snapshotCalls = 0;
  int gamesCalls = 0;
  int syncNowCalls = 0;

  bool get hasListener => _events.hasListener;

  void emit(dto.LinkEvent event) => _events.add(event);
  void emitError(Object error) => _events.addError(error);

  @override
  Stream<dto.LinkEvent> events() => _events.stream;

  @override
  Future<dto.Snapshot> snapshot() async {
    snapshotCalls++;
    return next;
  }

  @override
  Future<dto.GamesDto?> games() async {
    gamesCalls++;
    return nextGames;
  }

  @override
  Future<SyncNow> syncNow() async {
    syncNowCalls++;
    return nextSyncNow;
  }
}
