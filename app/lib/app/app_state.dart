import 'package:flutter/foundation.dart';

import '../bridge/link/model.dart' as dto;
import '../design/status.dart';

/// Everything the screens know about the daemon, immutable (app-shell spec,
/// App state). Built by `AppController` from the link's events.
@immutable
class AppState {
  const AppState({
    required this.connection,
    this.status,
    this.session,
    this.games = const [],
    this.recent = const [],
    this.liveTransfers = const {},
    this.device,
    this.players = const [],
    this.loaded = false,
  });

  /// Before the first answer: connecting, nothing cached.
  factory AppState.initial() =>
      const AppState(connection: dto.ConnectionState.connecting());

  final dto.ConnectionState connection;
  final dto.StatusDto? status;
  final dto.SessionDto? session;
  final List<dto.GameEntry> games;
  final List<dto.RecentDto> recent;

  /// "emulator/game" keys with progress since the last non-syncing status.
  final Set<String> liveTransfers;
  final String? device;
  final List<String> players;

  /// True once a snapshot has arrived.
  final bool loaded;

  /// The daemon's state as the design system's status; null until known.
  LsStatus? get lsStatus {
    final s = status;
    return s == null ? null : LsStatus.values.byName(s.state.name);
  }

  bool get online => connection is dto.ConnectionState_Online;

  bool get notSetUp => status?.error == dto.ErrorKindDto.notSetUp;

  /// A session is active and everything is synced: the hero shows the playing
  /// line. Any other status keeps its own headline.
  bool get playing => session != null && lsStatus == LsStatus.synced;

  /// [clearSession] sets the session to null (a null argument means "keep").
  AppState copyWith({
    dto.ConnectionState? connection,
    dto.StatusDto? status,
    dto.SessionDto? session,
    bool clearSession = false,
    List<dto.GameEntry>? games,
    List<dto.RecentDto>? recent,
    Set<String>? liveTransfers,
    String? device,
    List<String>? players,
    bool? loaded,
  }) => AppState(
    connection: connection ?? this.connection,
    status: status ?? this.status,
    session: clearSession ? null : (session ?? this.session),
    games: games ?? this.games,
    recent: recent ?? this.recent,
    liveTransfers: liveTransfers ?? this.liveTransfers,
    device: device ?? this.device,
    players: players ?? this.players,
    loaded: loaded ?? this.loaded,
  );

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is AppState &&
          connection == other.connection &&
          status == other.status &&
          session == other.session &&
          listEquals(games, other.games) &&
          listEquals(recent, other.recent) &&
          setEquals(liveTransfers, other.liveTransfers) &&
          device == other.device &&
          listEquals(players, other.players) &&
          loaded == other.loaded;

  @override
  int get hashCode => Object.hash(
    connection,
    status,
    session,
    Object.hashAll(games),
    Object.hashAll(recent),
    Object.hashAllUnordered(liveTransfers),
    device,
    Object.hashAll(players),
    loaded,
  );
}
