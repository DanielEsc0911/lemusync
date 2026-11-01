import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/design/components/status_badge.dart';
import 'package:lemusync/design/status.dart';
import 'package:lemusync/design/theme.dart';
import 'package:lemusync/design/tokens/colors.g.dart';
import 'package:lemusync/l10n/generated/app_localizations.dart';
import 'package:lemusync/l10n/generated/app_localizations_en.dart';
import 'package:lemusync/l10n/generated/app_localizations_es.dart';

import '../../support/fonts.dart';

Widget _app(
  LsColors c,
  Widget child, {
  double scale = 1,
  String locale = 'en',
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
    child: Scaffold(body: SizedBox(width: 320, child: child)),
  ),
);

void main() {
  setUpAll(loadBrandFonts);

  const themes = {
    'light': LsColors.light,
    'dark': LsColors.dark,
    'lightHighContrast': LsColors.lightHighContrast,
    'darkHighContrast': LsColors.darkHighContrast,
  };
  final locales = {'en': AppLocalizationsEn(), 'es': AppLocalizationsEs()};

  for (final MapEntry(key: themeName, value: c) in themes.entries) {
    for (final MapEntry(key: loc, value: l10n) in locales.entries) {
      for (final s in LsStatus.values) {
        testWidgets('${s.name} $themeName $loc', (t) async {
          final h = t.ensureSemantics();
          await t.binding.setSurfaceSize(const Size(320, 800));
          addTearDown(() => t.binding.setSurfaceSize(null));
          for (final scale in [1.0, 2.0]) {
            await t.pumpWidget(
              _app(
                c,
                StatusBadge(status: s),
                scale: scale,
                locale: loc,
              ),
            );
            expect(t.takeException(), isNull);
          }
          final label = s.label(l10n);
          expect(find.bySemanticsLabel(label), findsOneWidget);
          await expectLater(t, meetsGuideline(textContrastGuideline));
          await t.pumpWidget(
            _app(c, StatusBadge(status: s, compact: true), locale: loc),
          );
          expect(find.bySemanticsLabel(label), findsOneWidget);
          h.dispose();
        });
      }
    }
  }
}
