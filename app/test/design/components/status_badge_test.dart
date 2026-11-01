import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/design/components/status_badge.dart';
import 'package:lemusync/design/status.dart';
import 'package:lemusync/design/theme.dart';
import 'package:lemusync/design/tokens/colors.g.dart';
import 'package:lemusync/l10n/generated/app_localizations.dart';
import 'package:lemusync/l10n/generated/app_localizations_en.dart';

import '../../support/fonts.dart';

Widget _app(LsColors c, Widget child, {double scale = 1}) => MaterialApp(
  theme: lsTheme(c),
  localizationsDelegates: const [
    AppLocalizations.delegate,
    GlobalMaterialLocalizations.delegate,
    GlobalWidgetsLocalizations.delegate,
  ],
  supportedLocales: AppLocalizations.supportedLocales,
  home: MediaQuery(
    data: MediaQueryData(textScaler: TextScaler.linear(scale)),
    child: Scaffold(body: SizedBox(width: 320, child: child)),
  ),
);

void main() {
  setUpAll(loadBrandFonts);

  const themes = [
    LsColors.light,
    LsColors.dark,
    LsColors.lightHighContrast,
    LsColors.darkHighContrast,
  ];

  for (final c in themes) {
    for (final s in LsStatus.values) {
      testWidgets('${s.name} ${c.brightness} ${c.hashCode}', (t) async {
        final h = t.ensureSemantics();
        await t.binding.setSurfaceSize(const Size(320, 800));
        addTearDown(() => t.binding.setSurfaceSize(null));
        for (final scale in [1.0, 2.0]) {
          await t.pumpWidget(_app(c, StatusBadge(status: s), scale: scale));
          expect(t.takeException(), isNull);
        }
        final label = s.label(AppLocalizationsEn());
        expect(find.bySemanticsLabel(label), findsOneWidget);
        await expectLater(t, meetsGuideline(textContrastGuideline));
        await t.pumpWidget(_app(c, StatusBadge(status: s, compact: true)));
        expect(find.bySemanticsLabel(label), findsOneWidget);
        h.dispose();
      });
    }
  }
}
