import 'dart:async';

import '../bridge/api/daemon.dart' as frb;
import '../bridge/link/calls.dart';
import '../bridge/link/model.dart';

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
}

/// [DaemonLink] over the generated bridge. The only file outside `lib/bridge/`
/// that imports it.
final class FrbDaemonLink implements DaemonLink {
  /// [newHandle] replaces the native handle in tests.
  FrbDaemonLink({frb.WatchHandle Function()? newHandle})
    : _newHandle = newHandle ?? frb.WatchHandle.new;

  final frb.WatchHandle Function() _newHandle;

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
}
