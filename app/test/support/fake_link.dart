import 'dart:async';

import 'package:lemusync/app/call_result.dart';
import 'package:lemusync/app/daemon_link.dart';
import 'package:lemusync/bridge/link/calls.dart';
import 'package:lemusync/bridge/link/flow_model.dart';
import 'package:lemusync/bridge/link/model.dart' as dto;
import 'package:lemusync/bridge/link/play.dart';

const _notRunning = CallFailed<Never>(dto.ConnectionState.notRunning());

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

  /// When set, `snapshot`, `games` and `syncNow` throw it (a broken bridge).
  Object? throwOnCall;

  bool get hasListener => _events.hasListener;

  void emit(dto.LinkEvent event) => _events.add(event);
  void emitError(Object error) => _events.addError(error);

  @override
  Stream<dto.LinkEvent> events() => _events.stream;

  @override
  Future<dto.Snapshot> snapshot() async {
    snapshotCalls++;
    if (throwOnCall case final e?) throw e;
    return next;
  }

  @override
  Future<dto.GamesDto?> games() async {
    gamesCalls++;
    if (throwOnCall case final e?) throw e;
    return nextGames;
  }

  @override
  Future<SyncNow> syncNow() async {
    syncNowCalls++;
    if (throwOnCall case final e?) throw e;
    return nextSyncNow;
  }

  // Game flows (3c): each call is logged in [calls] and answers the matching
  // `next…` value (not running by default). They never throw: the real link
  // turns every failure into a CallFailed.

  /// One line per flow call, e.g. `restore Ana cemu aa v1`.
  final List<String> calls = [];

  CallResult<HistoryDto> nextHistory = _notRunning;
  CallResult<List<ConflictDto>> nextConflicts = _notRunning;
  CallResult<List<LeaseDto>> nextLeases = _notRunning;
  CallResult<MovedDto> nextRestore = _notRunning;
  CallResult<MovedDto> nextResolve = _notRunning;
  CallResult<int> nextLeaseRelease = _notRunning;

  /// The play sessions started, newest last.
  final List<FakePlaySession> plays = [];

  @override
  Future<CallResult<HistoryDto>> history({
    required String player,
    required String emulator,
    required String game,
  }) async {
    calls.add('history $player $emulator $game');
    return nextHistory;
  }

  @override
  Future<CallResult<List<ConflictDto>>> conflicts() async {
    calls.add('conflicts');
    return nextConflicts;
  }

  @override
  Future<CallResult<List<LeaseDto>>> leases() async {
    calls.add('leases');
    return nextLeases;
  }

  @override
  Future<CallResult<MovedDto>> restore({
    required String player,
    required String emulator,
    required String game,
    required String what,
  }) async {
    calls.add('restore $player $emulator $game $what');
    return nextRestore;
  }

  @override
  Future<CallResult<MovedDto>> resolve({
    required String player,
    required String emulator,
    required String game,
    required String keep,
  }) async {
    calls.add('resolve $player $emulator $game $keep');
    return nextResolve;
  }

  @override
  Future<CallResult<int>> leaseRelease(String game) async {
    calls.add('lease_release $game');
    return nextLeaseRelease;
  }

  @override
  PlaySession play({
    required String player,
    required String emulator,
    required String game,
  }) {
    calls.add('play $player $emulator $game');
    final s = FakePlaySession();
    plays.add(s);
    return s;
  }
}

/// A scripted play session: the test pushes updates with [push] and reads the
/// answers the app gave in [answers].
class FakePlaySession implements PlaySession {
  final StreamController<PlayUpdate> _updates = StreamController<PlayUpdate>(
    sync: true,
  );
  final List<PlayAnswerDto> answers = [];
  bool get listening => _updates.hasListener;
  bool cancelled = false;

  FakePlaySession() {
    _updates.onCancel = () => cancelled = true;
  }

  void push(PlayUpdate u) => _updates.add(u);
  Future<void> end() => _updates.close();

  @override
  Stream<PlayUpdate> get updates => _updates.stream;

  @override
  bool answer(PlayAnswerDto answer) {
    if (cancelled) return false;
    answers.add(answer);
    return true;
  }
}
