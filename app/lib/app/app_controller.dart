import 'dart:async';

import 'package:flutter/foundation.dart';

import '../bridge/link/calls.dart';
import '../bridge/link/model.dart' as dto;
import 'app_state.dart';
import 'daemon_link.dart';

/// Holds the live [AppState], fed by the daemon link (app-shell spec, App
/// state). Never throws to the UI; notifies only when the state changed.
class AppController extends ChangeNotifier {
  AppController(this._link);

  final DaemonLink _link;
  AppState _state = AppState.initial();
  StreamSubscription<dto.LinkEvent>? _sub;
  bool _disposed = false;

  AppState get state => _state;

  /// Starts listening to the link's events. Calling it twice does nothing.
  void start() {
    if (_sub != null || _disposed) return;
    _sub = _link.events().listen(
      _apply,
      // A broken stream is "not running", never an exception for the UI.
      onError: (Object _) => _set(
        _state.copyWith(connection: const dto.ConnectionState.notRunning()),
      ),
    );
  }

  /// Asks the daemon to sync now.
  Future<SyncNow> syncNow() => _link.syncNow();

  /// Fetches the snapshot and the games again (app resume, Retry).
  Future<void> refresh() async {
    final snap = await _link.snapshot();
    _apply(dto.LinkEvent.snapshot(snap));
    final games = await _link.games();
    if (games != null) _apply(dto.LinkEvent.games(games));
  }

  void _apply(dto.LinkEvent event) {
    final s = _state;
    _set(switch (event) {
      dto.LinkEvent_Snapshot(:final field0) => s.copyWith(
        connection: field0.connection,
        status: field0.status,
        session: field0.session,
        clearSession: field0.session == null,
        device: field0.device,
        players: field0.players,
        liveTransfers: const {},
        loaded: true,
      ),
      dto.LinkEvent_Games(:final field0) => s.copyWith(
        games: field0.games,
        recent: field0.recent,
      ),
      dto.LinkEvent_Status(:final field0) => s.copyWith(
        status: field0,
        liveTransfers: field0.state == dto.StateDto.syncing
            ? s.liveTransfers
            : const {},
      ),
      dto.LinkEvent_Session(:final field0) => s.copyWith(
        session: field0,
        clearSession: field0 == null,
      ),
      dto.LinkEvent_Progress(:final field0) => s.copyWith(
        liveTransfers: {
          ...s.liveTransfers,
          '${field0.emulator}/${field0.game}',
        },
      ),
      // Stale status and games stay; the UI labels them.
      dto.LinkEvent_Connection(:final field0) => s.copyWith(connection: field0),
    });
  }

  void _set(AppState next) {
    if (_disposed || next == _state) return;
    _state = next;
    notifyListeners();
  }

  @override
  void dispose() {
    if (_disposed) return;
    _disposed = true;
    unawaited(_sub?.cancel());
    _sub = null;
    super.dispose();
  }
}
