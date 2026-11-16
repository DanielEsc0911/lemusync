import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/design/components/progress_ring.dart';
import 'package:lemusync/design/components/status_hero.dart';
import 'package:lemusync/design/status.dart';
import 'package:lemusync/design/theme.dart';
import 'package:lemusync/design/tokens/colors.g.dart';
import 'package:lemusync/l10n/generated/app_localizations.dart';

import '../../support/fonts.dart';

Widget _app(
  LsColors c,
  Widget child, {
  double scale = 1,
  String locale = 'en',
  double width = 320,
}) => MaterialApp(
  theme: lsTheme(c),
  locale: Locale(locale),
  localizationsDelegates: const [
    AppLocalizations.delegate,
    GlobalMaterialLocalizations.delegate,
    GlobalWidgetsLocalizations.delegate,
    GlobalCupertinoLocalizations.delegate,
  ],
  supportedLocales: AppLocalizations.supportedLocales,
  home: MediaQuery(
    data: MediaQueryData(textScaler: TextScaler.linear(scale)),
    child: Scaffold(
      body: SingleChildScrollView(
        child: SizedBox(width: width, child: child),
      ),
    ),
  ),
);

void main() {
  setUpAll(loadBrandFonts);

  const cases = <(String, int, int, String, String?)>[
    ('en', 1, 0, 'Uploading 1 change', null),
    ('en', 3, 0, 'Uploading 3 changes', null),
    ('en', 0, 2, 'Downloading 2 changes', null),
    ('en', 3, 2, 'Uploading 3 changes', 'Downloading 2 changes'),
    ('es', 1, 0, 'Subiendo 1 cambio', null),
    ('es', 3, 0, 'Subiendo 3 cambios', null),
    ('es', 0, 2, 'Descargando 2 cambios', null),
    ('es', 3, 2, 'Subiendo 3 cambios', 'Descargando 2 cambios'),
  ];
  for (final (loc, up, down, headline, extra) in cases) {
    testWidgets('syncing $loc up=$up down=$down', (t) async {
      await t.pumpWidget(
        _app(
          LsColors.light,
          StatusHero(
            status: LsStatus.syncing,
            uploads: up,
            downloads: down,
            detail: 'Zelda',
          ),
          locale: loc,
        ),
      );
      expect(find.text(headline), findsOneWidget);
      if (extra != null) {
        expect(find.textContaining(extra), findsOneWidget);
      }
      expect(find.textContaining('Zelda'), findsOneWidget);
    });
  }

  testWidgets('non-syncing status shows its label', (t) async {
    await t.pumpWidget(
      _app(
        LsColors.light,
        const StatusHero(status: LsStatus.synced, uploads: 4),
      ),
    );
    expect(find.text('All saves synced'), findsOneWidget);
  });

  for (final s in LsStatus.values) {
    testWidgets('ring only while scanning or syncing: ${s.name}', (t) async {
      await t.pumpWidget(_app(LsColors.light, StatusHero(status: s)));
      final busy = s == LsStatus.scanning || s == LsStatus.syncing;
      expect(find.byType(ProgressRing), busy ? findsOneWidget : findsNothing);
    });
  }

  testWidgets('onSync fires and the target is large enough', (t) async {
    var taps = 0;
    await t.pumpWidget(
      _app(
        LsColors.light,
        StatusHero(status: LsStatus.synced, onSync: () => taps++),
      ),
    );
    await expectLater(t, meetsGuideline(androidTapTargetGuideline));
    await t.tap(find.text('Sync now'));
    expect(taps, 1);
  });

  for (final loc in ['en', 'es']) {
    testWidgets('200% scale, narrow, no overflow $loc', (t) async {
      final h = t.ensureSemantics();
      for (final c in [LsColors.light, LsColors.darkHighContrast]) {
        await t.pumpWidget(
          _app(
            c,
            StatusHero(
              status: LsStatus.attention,
              detail: 'Elige qué partida conservar entre los dos equipos',
              onSync: () {},
            ),
            scale: 2,
            locale: loc,
          ),
        );
        expect(t.takeException(), isNull);
        await expectLater(t, meetsGuideline(textContrastGuideline));
        await expectLater(t, meetsGuideline(androidTapTargetGuideline));
      }
      h.dispose();
    });
  }
}
