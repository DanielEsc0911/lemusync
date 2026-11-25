import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/app/app_state.dart';
import 'package:lemusync/bridge/link/model.dart' as dto;
import 'package:lemusync/design/status.dart';
import 'package:lemusync/l10n/generated/app_localizations.dart';
import 'package:lemusync/screens/home/status_mapper.dart';

import '../harness.dart';

AppState st(dto.StatusDto s, {dto.SessionDto? sess}) =>
    AppState(connection: online, status: s, session: sess);

void main() {
  late AppLocalizations en;
  late AppLocalizations es;
  setUpAll(() async {
    en = await AppLocalizations.delegate.load(const Locale('en'));
    es = await AppLocalizations.delegate.load(const Locale('es'));
  });

  test('synced has no detail', () {
    final h = heroFor(st(status(dto.StateDto.synced)), en);
    expect(h.status, LsStatus.synced);
    expect(h.detail, isNull);
  });

  test('scanning keeps its status, no detail', () {
    final h = heroFor(st(status(dto.StateDto.scanning)), en);
    expect(h.status, LsStatus.scanning);
    expect(h.detail, isNull);
  });

  test('syncing carries uploads and downloads', () {
    final a = heroFor(st(status(dto.StateDto.syncing, uploads: 1)), en);
    expect((a.uploads, a.downloads), (1, 0));
    final b = heroFor(
      st(status(dto.StateDto.syncing, uploads: 3, downloads: 2)),
      en,
    );
    expect((b.uploads, b.downloads), (3, 2));
    expect(b.status, LsStatus.syncing);
  });

  test('paused has the label only', () {
    final h = heroFor(st(status(dto.StateDto.paused)), en);
    expect(h.status, LsStatus.paused);
    expect(h.detail, isNull);
  });

  test('offline with queued changes', () {
    final h = heroFor(st(status(dto.StateDto.offline, queued: 2)), en);
    expect(h.detail, 'Offline · 2 changes waiting');
    expect(
      heroFor(st(status(dto.StateDto.offline, queued: 2)), es).detail,
      'Sin conexión · 2 cambios en espera',
    );
    expect(
      heroFor(st(status(dto.StateDto.offline, queued: 1)), en).detail,
      'Offline · 1 change waiting',
    );
  });

  test('offline with nothing queued has no detail', () {
    expect(heroFor(st(status(dto.StateDto.offline)), en).detail, isNull);
  });

  test('attention: conflicts, leases, both (conflicts first)', () {
    expect(
      heroFor(st(status(dto.StateDto.attention, conflicts: 1)), en).detail,
      '1 conflict needs you',
    );
    expect(
      heroFor(st(status(dto.StateDto.attention, leases: 2)), en).detail,
      '2 games are in use on other devices',
    );
    expect(
      heroFor(
        st(status(dto.StateDto.attention, conflicts: 2, leases: 1)),
        en,
      ).detail,
      '2 conflicts need you\n1 game is in use on another device',
    );
    expect(
      heroFor(st(status(dto.StateDto.attention, conflicts: 1)), es).detail,
      '1 conflicto necesita tu decisión',
    );
  });

  test('error: text by kind', () {
    String? d(dto.ErrorKindDto k) =>
        heroFor(st(status(dto.StateDto.error, error: k)), en).detail;
    expect(d(dto.ErrorKindDto.auth), 'Storage sign-in expired');
    expect(d(dto.ErrorKindDto.quota), 'Your storage is full');
    expect(
      d(dto.ErrorKindDto.permission),
      "LemuSync can't read or write a folder",
    );
    expect(d(dto.ErrorKindDto.config), "The configuration isn't valid");
    expect(d(dto.ErrorKindDto.notSetUp), isNull);
  });

  test('playing: synced with a session', () {
    final h = heroFor(st(status(dto.StateDto.synced), sess: session()), en);
    expect(h.status, LsStatus.synced);
    expect(h.detail, 'Playing BotW · Daniel');
    expect(
      heroFor(st(status(dto.StateDto.synced), sess: session()), es).detail,
      'Jugando BotW · Daniel',
    );
  });

  test('playing falls back to the game key without a name', () {
    final h = heroFor(
      st(status(dto.StateDto.synced), sess: session(name: null)),
      en,
    );
    expect(h.detail, 'Playing botw-key · Daniel');
  });

  test('syncing with a session keeps the syncing hero', () {
    final h = heroFor(
      st(status(dto.StateDto.syncing, uploads: 1), sess: session()),
      en,
    );
    expect(h.status, LsStatus.syncing);
    expect(h.detail, isNull);
  });

  group('relative', () {
    String r(AppLocalizations l, Duration d) => relative(l, now, ago(d));
    test('en', () {
      expect(r(en, const Duration(seconds: 30)), 'just now');
      expect(r(en, const Duration(minutes: 5)), '5 min ago');
      expect(r(en, const Duration(hours: 2)), '2 hours ago');
      expect(r(en, const Duration(hours: 1)), '1 hour ago');
      expect(r(en, const Duration(days: 3)), '3 days ago');
    });
    test('es', () {
      expect(r(es, const Duration(seconds: 30)), 'hace un momento');
      expect(r(es, const Duration(minutes: 5)), 'hace 5 min');
      expect(r(es, const Duration(hours: 2)), 'hace 2 horas');
      expect(r(es, const Duration(days: 3)), 'hace 3 días');
    });
    test('a time in the future reads as just now', () {
      expect(relative(en, now, ago(const Duration(minutes: -5))), 'just now');
    });
  });
}
