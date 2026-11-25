import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/app/app_state.dart';
import 'package:lemusync/bridge/link/model.dart' as dto;
import 'package:lemusync/design/status.dart';

dto.StatusDto status(dto.StateDto s, {dto.ErrorKindDto? error}) =>
    dto.StatusDto(
      state: s,
      uploads: 0,
      downloads: 0,
      bytes: BigInt.zero,
      queued: 0,
      conflicts: 0,
      foreignLeases: 0,
      error: error,
    );

final session = dto.SessionDto(
  game: 'g',
  emulator: 'e',
  player: 'p',
  device: 'd',
  startedMs: BigInt.one,
);

void main() {
  test('initial state has no status and is not online', () {
    final s = AppState.initial();
    expect(s.lsStatus, isNull);
    expect(s.online, isFalse);
    expect(s.loaded, isFalse);
    expect(s.playing, isFalse);
  });

  test('maps the seven states one to one', () {
    expect(dto.StateDto.values.length, 7);
    for (final d in dto.StateDto.values) {
      final s = AppState.initial().copyWith(status: status(d));
      expect(s.lsStatus, LsStatus.values.byName(d.name));
    }
  });

  test('playing: synced + session only', () {
    final base = AppState.initial();
    expect(
      base
          .copyWith(status: status(dto.StateDto.synced), session: session)
          .playing,
      isTrue,
    );
    expect(
      base
          .copyWith(status: status(dto.StateDto.syncing), session: session)
          .playing,
      isFalse,
    );
    expect(base.copyWith(status: status(dto.StateDto.synced)).playing, isFalse);
  });

  test('notSetUp', () {
    final s = AppState.initial().copyWith(
      status: status(dto.StateDto.error, error: dto.ErrorKindDto.notSetUp),
    );
    expect(s.notSetUp, isTrue);
    expect(s.copyWith(status: status(dto.StateDto.error)).notSetUp, isFalse);
  });

  test('copyWith can clear session and equality is by value', () {
    final a = AppState.initial().copyWith(session: session);
    final same = AppState.initial().copyWith(session: session);
    final b = a.copyWith(clearSession: true);
    expect(b.session, isNull);
    expect(a, same);
    expect(a.hashCode, same.hashCode);
    expect(a == b, isFalse);
  });

  test('online follows the connection', () {
    final s = AppState.initial().copyWith(
      connection: const dto.ConnectionState.online(daemonVersion: '1'),
    );
    expect(s.online, isTrue);
  });
}
