import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/app/app_controller.dart';
import 'package:lemusync/app/app_scope.dart';
import 'package:lemusync/app/app_shell.dart';
import 'package:lemusync/app/destination.dart';
import 'package:lemusync/bridge/link/model.dart' as dto;
import 'package:lemusync/design/theme.dart';
import 'package:lemusync/design/tokens/colors.g.dart';
import 'package:lemusync/l10n/generated/app_localizations.dart';

import '../support/fake_link.dart';
import '../support/fonts.dart';

void main() {
  setUpAll(loadBrandFonts);

  late FakeDaemonLink link;
  late AppController controller;
  late List<Destination> picked;

  Future<void> pump(WidgetTester t, {required bool online}) async {
    t.view.physicalSize = const Size(900, 700);
    t.view.devicePixelRatio = 1;
    addTearDown(t.view.reset);
    link = FakeDaemonLink();
    if (online) {
      link.next = const dto.Snapshot(
        connection: dto.ConnectionState.online(daemonVersion: '1'),
        players: [],
      );
    }
    controller = AppController(link)..start();
    addTearDown(controller.dispose);
    await controller.refresh();
    picked = [];
    await t.pumpWidget(
      AppScope(
        controller: controller,
        child: MaterialApp(
          theme: lsTheme(LsColors.light),
          locale: const Locale('en'),
          localizationsDelegates: const [
            AppLocalizations.delegate,
            GlobalMaterialLocalizations.delegate,
            GlobalWidgetsLocalizations.delegate,
            GlobalCupertinoLocalizations.delegate,
          ],
          supportedLocales: AppLocalizations.supportedLocales,
          home: AppShell(
            current: Destination.home,
            onSelect: picked.add,
            body: const TextField(),
          ),
        ),
      ),
    );
    await t.pump();
  }

  Future<void> chord(
    WidgetTester t,
    LogicalKeyboardKey mod,
    LogicalKeyboardKey k,
  ) async {
    await t.sendKeyDownEvent(mod);
    await t.sendKeyEvent(k);
    await t.sendKeyUpEvent(mod);
    await t.pump();
  }

  for (final (name, mod) in [
    ('Ctrl', LogicalKeyboardKey.controlLeft),
    ('Cmd', LogicalKeyboardKey.metaLeft),
  ]) {
    testWidgets('$name+R syncs once when online', (t) async {
      await pump(t, online: true);
      await chord(t, mod, LogicalKeyboardKey.keyR);
      expect(link.syncNowCalls, 1);
    });

    testWidgets('$name+R does nothing when not online', (t) async {
      await pump(t, online: false);
      await chord(t, mod, LogicalKeyboardKey.keyR);
      expect(link.syncNowCalls, 0);
    });
  }

  testWidgets('plain digits and R in a text field trigger nothing', (t) async {
    await pump(t, online: true);
    await t.tap(find.byType(TextField));
    await t.pump();
    await t.sendKeyEvent(LogicalKeyboardKey.digit2);
    await t.sendKeyEvent(LogicalKeyboardKey.keyR);
    await t.pump();
    expect(picked, isEmpty);
    expect(link.syncNowCalls, 0);
  });

  testWidgets('maybeRead gives the controller without rebuilding on change', (
    t,
  ) async {
    link = FakeDaemonLink();
    controller = AppController(link)..start();
    addTearDown(controller.dispose);
    var builds = 0;
    AppController? seen;
    await t.pumpWidget(
      AppScope(
        controller: controller,
        child: Builder(
          builder: (context) {
            builds++;
            seen = AppScope.maybeRead(context);
            return const SizedBox();
          },
        ),
      ),
    );
    link.emit(
      const dto.LinkEvent.connection(
        dto.ConnectionState.online(daemonVersion: '1'),
      ),
    );
    await t.pump();
    expect(seen, same(controller));
    expect(controller.state.online, isTrue);
    expect(builds, 1);
  });
}
