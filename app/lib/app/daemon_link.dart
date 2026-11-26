import 'dart:async';

import '../bridge/api/daemon.dart' as frb;
import '../bridge/link/calls.dart';
import '../bridge/link/flow_model.dart';
import '../bridge/link/model.dart';
import '../bridge/link/play.dart';
import 'call_result.dart';

/// The app's one door to the daemon (app-shell spec, Data flow). The screens and
/// the controller depend on this interface; tests use a fake of it.
abstract interface class DaemonLink {
  /// Live events. Listening starts the watch; cancelling the subscription
  /// stops the Rust task.
  Stream<LinkEvent> events();

  /// The daemon's state right now. Never throws: "not running" is a value.
  Future<Snapshot> snapshot();

  /// The game library, or null when it could not be read.
  Future<GamesDto?> games();

  /// Asks the daemon to sync now.
  Future<SyncNow> syncNow();

  /// The versions of one player's stream of a game, newest first.
  Future<CallResult<HistoryDto>> history({
    required String player,
    required String emulator,
    required String game,
  });

  /// Every conflicted stream.
  Future<CallResult<List<ConflictDto>>> conflicts();

  /// Every lease in the repo, this device's included.
  Future<CallResult<List<LeaseDto>>> leases();

  /// Restores a version (or a local backup by name); the daemon backs up the
  /// current save first.
  Future<CallResult<MovedDto>> restore({
    required String player,
    required String emulator,
    required String game,
    required String what,
  });

  /// Resolves a conflict by keeping one head; the others stay in history.
  Future<CallResult<MovedDto>> resolve({
    required String player,
    required String emulator,
    required String game,
    required String keep,
  });

  /// Releases this device's leases on a game; how many it released.
  Future<CallResult<int>> leaseRelease(String game);

  /// A play session. It starts when [PlaySession.updates] is listened to.
  PlaySession play({
    required String player,
    required String emulator,
    required String game,
  });
}

/// One play session: steps and questions come in on [updates]; each question
/// is answered with [answer]. Cancelling [updates] ends the app's link to the
/// session (the daemon keeps a running game's session, uploads and releases
/// the lease).
abstract interface class PlaySession {
  Stream<PlayUpdate> get updates;

  /// Answers the waiting question; false when no session runs.
  bool answer(PlayAnswerDto answer);
}

/// [DaemonLink] over the generated bridge. The only file outside `lib/bridge/`
/// that imports it.
final class FrbDaemonLink implements DaemonLink {
  /// [newHandle] and [newPlayHandle] replace the native handles in tests.
  FrbDaemonLink({
    frb.WatchHandle Function()? newHandle,
    frb.PlayHandle Function()? newPlayHandle,
  }) : _newHandle = newHandle ?? frb.WatchHandle.new,
       _newPlayHandle = newPlayHandle ?? frb.PlayHandle.new;

  final frb.WatchHandle Function() _newHandle;
  final frb.PlayHandle Function() _newPlayHandle;

  @override
  Stream<LinkEvent> events() {
    frb.WatchHandle? handle;
    StreamSubscription<LinkEvent>? inner;
    late final StreamController<LinkEvent> controller;
    controller = StreamController<LinkEvent>(
      onListen: () {
        handle = _newHandle();
        inner = handle!.watch().listen(
          controller.add,
          onError: controller.addError,
          onDone: controller.close,
        );
      },
      // Dart stopped listening: end the Rust task now (V-SHELL-11). The Rust
      // side only notices a closed sink on its next emit.
      onCancel: () async {
        try {
          await inner?.cancel();
        } finally {
          await handle?.stop();
        }
      },
    );
    return controller.stream;
  }

  @override
  Future<Snapshot> snapshot() => frb.snapshot();

  @override
  Future<GamesDto?> games() => frb.games();

  @override
  Future<SyncNow> syncNow() => frb.syncNow();

  @override
  Future<CallResult<HistoryDto>> history({
    required String player,
    required String emulator,
    required String game,
  }) =>
      guard(() => frb.history(player: player, emulator: emulator, game: game));

  @override
  Future<CallResult<List<ConflictDto>>> conflicts() => guard(frb.conflicts);

  @override
  Future<CallResult<List<LeaseDto>>> leases() => guard(frb.leases);

  @override
  Future<CallResult<MovedDto>> restore({
    required String player,
    required String emulator,
    required String game,
    required String what,
  }) => guard(
    () =>
        frb.restore(player: player, emulator: emulator, game: game, what: what),
  );

  @override
  Future<CallResult<MovedDto>> resolve({
    required String player,
    required String emulator,
    required String game,
    required String keep,
  }) => guard(
    () =>
        frb.resolve(player: player, emulator: emulator, game: game, keep: keep),
  );

  @override
  Future<CallResult<int>> leaseRelease(String game) =>
      guard(() => frb.leaseRelease(game: game));

  @override
  PlaySession play({
    required String player,
    required String emulator,
    required String game,
  }) => _FrbPlaySession(_newPlayHandle, player, emulator, game);
}

final class _FrbPlaySession implements PlaySession {
  _FrbPlaySession(this._newHandle, this._player, this._emulator, this._game);

  final frb.PlayHandle Function() _newHandle;
  final String _player;
  final String _emulator;
  final String _game;
  frb.PlayHandle? _handle;

  @override
  late final Stream<PlayUpdate> updates = _start();

  Stream<PlayUpdate> _start() {
    StreamSubscription<PlayUpdate>? inner;
    late final StreamController<PlayUpdate> controller;
    controller = StreamController<PlayUpdate>(
      onListen: () {
        final handle = _handle = _newHandle();
        inner = handle
            .play(player: _player, emulator: _emulator, game: _game)
            .listen(
              controller.add,
              onError: controller.addError,
              onDone: controller.close,
            );
      },
      // As events(): stop the Rust side now, not on its next emit.
      onCancel: () async {
        try {
          await inner?.cancel();
        } finally {
          await _handle?.stop();
        }
      },
    );
    return controller.stream;
  }

  @override
  bool answer(PlayAnswerDto answer) => _handle?.answer(answer: answer) ?? false;
}
