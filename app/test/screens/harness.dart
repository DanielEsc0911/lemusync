import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/app/app_controller.dart';
import 'package:lemusync/app/app_scope.dart';
import 'package:lemusync/bridge/link/model.dart' as dto;
import 'package:lemusync/design/theme.dart';
import 'package:lemusync/design/tokens/colors.g.dart';
import 'package:lemusync/l10n/generated/app_localizations.dart';

import '../support/fake_link.dart';

const online = dto.ConnectionState.online(daemonVersion: '1');

/// A fixed "now" so relative times are exact.
final DateTime now = DateTime.fromMillisecondsSinceEpoch(1700000000000);
int ago(Duration d) => now.millisecondsSinceEpoch - d.inMilliseconds;

dto.StatusDto status(
  dto.StateDto state, {
  int? lastSyncMs,
  int uploads = 0,
  int downloads = 0,
  int queued = 0,
  int conflicts = 0,
  int leases = 0,
  dto.ErrorKindDto? error,
}) => dto.StatusDto(
  state: state,
  lastSyncMs: lastSyncMs == null ? null : BigInt.from(lastSyncMs),
  uploads: uploads,
  downloads: downloads,
  bytes: BigInt.zero,
  queued: queued,
  conflicts: conflicts,
  foreignLeases: leases,
  error: error,
);

dto.SessionDto session({String? name = 'BotW', String player = 'Daniel'}) =>
    dto.SessionDto(
      game: 'botw-key',
      name: name,
      emulator: 'ryujinx',
      player: player,
      device: 'deck',
      startedMs: BigInt.one,
    );

dto.RecentDto recentGame(
  String key, {
  String? name,
  int? ms,
  bool launchable = true,
  String emulator = 'ryujinx',
  String player = 'Daniel',
}) => dto.RecentDto(
  emulator: emulator,
  key: key,
  name: name,
  player: player,
  lastVersionMs: ms == null ? null : BigInt.from(ms),
  launchable: launchable,
);

/// Puts the controller behind [link] in the given state through events.
Future<void> feed(
  WidgetTester t,
  FakeDaemonLink link, {
  dto.ConnectionState connection = online,
  dto.StatusDto? st,
  dto.SessionDto? sess,
  List<dto.RecentDto> recent = const [],
}) async {
  link.emit(
    dto.LinkEvent.snapshot(
      dto.Snapshot(
        connection: connection,
        status: st,
        session: sess,
        device: 'deck',
        players: const ['Daniel'],
      ),
    ),
  );
  link.emit(dto.LinkEvent.games(dto.GamesDto(games: const [], recent: recent)));
  await t.pump();
}

Widget harness(
  AppController controller,
  Widget child, {
  LsColors colors = LsColors.light,
  String locale = 'en',
  double scale = 1,
  double width = 390,
  double height = 800,
}) => AppScope(
  controller: controller,
  child: MaterialApp(
    theme: lsTheme(colors),
    locale: Locale(locale),
    localizationsDelegates: const [
      AppLocalizations.delegate,
      GlobalMaterialLocalizations.delegate,
      GlobalWidgetsLocalizations.delegate,
      GlobalCupertinoLocalizations.delegate,
    ],
    supportedLocales: AppLocalizations.supportedLocales,
    home: MediaQuery(
      data: MediaQueryData(
        size: Size(width, height),
        textScaler: TextScaler.linear(scale),
      ),
      child: Scaffold(
        body: SizedBox(width: width, height: height, child: child),
      ),
    ),
  ),
);

const themes = {
  'light': LsColors.light,
  'dark': LsColors.dark,
  'lightHC': LsColors.lightHighContrast,
  'darkHC': LsColors.darkHighContrast,
};
