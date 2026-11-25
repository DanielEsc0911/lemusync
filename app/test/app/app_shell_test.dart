import 'dart:ui' show Tristate;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/app/app_shell.dart';
import 'package:lemusync/app/destination.dart';
import 'package:lemusync/design/theme.dart';
import 'package:lemusync/design/tokens/colors.g.dart';
import 'package:lemusync/l10n/generated/app_localizations.dart';
import 'package:lemusync/main.dart';

import '../support/fake_link.dart';
import '../support/fonts.dart';

Widget _app(
  Widget home, {
  LsColors colors = LsColors.light,
  Locale locale = const Locale('en'),
  double textScale = 1,
  bool disableAnimations = false,
}) {
  return MaterialApp(
    theme: lsTheme(colors),
    locale: locale,
    localizationsDelegates: const [
      AppLocalizations.delegate,
      GlobalMaterialLocalizations.delegate,
      GlobalWidgetsLocalizations.delegate,
      GlobalCupertinoLocalizations.delegate,
    ],
    supportedLocales: AppLocalizations.supportedLocales,
    builder: (context, child) => MediaQuery(
      data: MediaQuery.of(context).copyWith(
        textScaler: TextScaler.linear(textScale),
        disableAnimations: disableAnimations,
      ),
      child: child!,
    ),
    home: home,
  );
}

void _size(WidgetTester t, double w) {
  t.view.physicalSize = Size(w, 800);
  t.view.devicePixelRatio = 1;
  addTearDown(t.view.reset);
}

void main() {
  setUpAll(loadBrandFonts);

  Future<List<Destination>> pump(
    WidgetTester t,
    double w, {
    Locale locale = const Locale('en'),
    double scale = 1,
    LsColors colors = LsColors.light,
  }) async {
    _size(t, w);
    final picked = <Destination>[];
    await t.pumpWidget(
      _app(
        AppShell(
          current: Destination.home,
          onSelect: picked.add,
          body: const TextField(),
        ),
        locale: locale,
        textScale: scale,
        colors: colors,
      ),
    );
    return picked;
  }

  testWidgets('bar at 390, rail at 900, sidebar at 1400', (t) async {
    await pump(t, 390);
    expect(find.byType(NavigationBar), findsOneWidget);
    expect(find.byType(NavigationRail), findsNothing);
    await pump(t, 900);
    expect(find.byType(NavigationBar), findsNothing);
    expect(
      t.widget<NavigationRail>(find.byType(NavigationRail)).extended,
      isFalse,
    );
    await pump(t, 1400);
    expect(
      t.widget<NavigationRail>(find.byType(NavigationRail)).extended,
      isTrue,
    );
  });

  for (final w in [390.0, 900.0, 1400.0]) {
    testWidgets('only Home then Games at $w', (t) async {
      await pump(t, w);
      final home = t.getTopLeft(find.text('Home'));
      final games = t.getTopLeft(find.text('Games'));
      expect(w < 600 ? home.dx < games.dx : home.dy < games.dy, isTrue);
      for (final other in ['Activity', 'Devices', 'Settings']) {
        expect(find.text(other), findsNothing);
      }
    });

    testWidgets('tapping selects at $w', (t) async {
      final picked = await pump(t, w);
      await t.tap(find.text('Games'));
      await t.tap(find.text('Home'));
      expect(picked, [Destination.games, Destination.home]);
    });
  }

  testWidgets('Ctrl and Meta digits select; 3 does nothing', (t) async {
    final picked = await pump(t, 900);
    await t.sendKeyDownEvent(LogicalKeyboardKey.controlLeft);
    await t.sendKeyEvent(LogicalKeyboardKey.digit2);
    await t.sendKeyEvent(LogicalKeyboardKey.digit1);
    await t.sendKeyEvent(LogicalKeyboardKey.digit3);
    await t.sendKeyUpEvent(LogicalKeyboardKey.controlLeft);
    await t.sendKeyDownEvent(LogicalKeyboardKey.metaLeft);
    await t.sendKeyEvent(LogicalKeyboardKey.digit2);
    await t.sendKeyUpEvent(LogicalKeyboardKey.metaLeft);
    expect(picked, [Destination.games, Destination.home, Destination.games]);
  });

  testWidgets('plain digits in a text field do not select', (t) async {
    final picked = await pump(t, 900);
    await t.tap(find.byType(TextField));
    await t.pump();
    await t.sendKeyEvent(LogicalKeyboardKey.digit2);
    expect(picked, isEmpty);
  });

  testWidgets('tap targets, contrast and semantics', (t) async {
    for (final w in [390.0, 900.0]) {
      final h = t.ensureSemantics();
      await pump(t, w);
      await expectLater(t, meetsGuideline(androidTapTargetGuideline));
      await expectLater(t, meetsGuideline(textContrastGuideline));
      final data = t.getSemantics(find.text('Home').first);
      expect(data.label, contains('Home'));
      expect(data.flagsCollection.isSelected, Tristate.isTrue);
      h.dispose();
    }
  });

  testWidgets('200 percent text does not overflow, four themes, es', (t) async {
    for (final c in [
      LsColors.light,
      LsColors.dark,
      LsColors.lightHighContrast,
      LsColors.darkHighContrast,
    ]) {
      await pump(t, 390, scale: 2, colors: c, locale: const Locale('es'));
      expect(t.takeException(), isNull);
    }
  });

  testWidgets('reduce motion: no animation keeps running after a change', (
    t,
  ) async {
    _size(t, 900);
    var current = Destination.home;
    await t.pumpWidget(
      _app(
        AppShell(current: current, onSelect: (_) {}, body: const SizedBox()),
        disableAnimations: true,
      ),
    );
    current = Destination.games;
    await t.pumpWidget(
      _app(
        AppShell(current: current, onSelect: (_) {}, body: const SizedBox()),
        disableAnimations: true,
      ),
    );
    // NavigationRail runs its own short selection animation and ignores
    // disableAnimations; the shell adds none and nothing keeps running.
    await t.pump(const Duration(milliseconds: 500));
    expect(t.binding.transientCallbackCount, 0);
  });

  testWidgets('main wiring: opens on Home, Games shows the Games stub', (
    t,
  ) async {
    _size(t, 900);
    final link = FakeDaemonLink();
    await t.pumpWidget(LemuSyncApp(link: link, locale: const Locale('en')));
    await t.pump();
    expect(link.hasListener, isTrue);
    // Home is the body (waiting for the daemon); Games is still a stub.
    expect(find.text('Connecting…'), findsOneWidget);
    expect(find.text('Home'), findsOneWidget);
    await t.tap(find.text('Games'));
    await t.pump();
    expect(find.text('Connecting…'), findsNothing);
    expect(find.text('Games'), findsNWidgets(2));
    expect(find.text('Home'), findsOneWidget);
    await t.pumpWidget(const SizedBox());
  });
}
