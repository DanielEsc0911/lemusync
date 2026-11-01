import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/design/components/empty_state.dart';
import 'package:lemusync/design/components/error_state.dart';
import 'package:lemusync/design/components/ls_banner.dart';
import 'package:lemusync/design/components/progress_ring.dart';
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
  bool reduceMotion = false,
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
    data: MediaQueryData(
      textScaler: TextScaler.linear(scale),
      disableAnimations: reduceMotion,
    ),
    child: Scaffold(
      body: SingleChildScrollView(child: SizedBox(width: 320, child: child)),
    ),
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
  const texts = {
    'en': 'Your saves are waiting for a storage to be connected',
    'es': 'Tus partidas esperan a que conectes un almacenamiento',
  };

  group('ProgressRing', () {
    testWidgets('determinate has no animation', (t) async {
      await t.pumpWidget(_app(LsColors.light, const ProgressRing(value: 0.5)));
      expect(t.hasRunningAnimations, isFalse);
      expect(find.byType(CustomPaint), findsWidgets);
    });

    testWidgets('indeterminate animates', (t) async {
      await t.pumpWidget(_app(LsColors.light, const ProgressRing()));
      await t.pump(const Duration(milliseconds: 100));
      expect(t.hasRunningAnimations, isTrue);
    });

    testWidgets('reduce motion stops it', (t) async {
      await t.pumpWidget(
        _app(LsColors.light, const ProgressRing(), reduceMotion: true),
      );
      await t.pump(const Duration(milliseconds: 100));
      expect(t.hasRunningAnimations, isFalse);
    });

    testWidgets('semantics label', (t) async {
      final h = t.ensureSemantics();
      await t.pumpWidget(
        _app(
          LsColors.light,
          const ProgressRing(value: 0.5, semanticsLabel: 'Uploading'),
        ),
      );
      expect(find.bySemanticsLabel('Uploading'), findsOneWidget);
      h.dispose();
    });
  });

  for (final MapEntry(key: themeName, value: c) in themes.entries) {
    for (final loc in texts.keys) {
      testWidgets('feedback widgets $themeName $loc', (t) async {
        final h = t.ensureSemantics();
        await t.binding.setSurfaceSize(const Size(320, 1600));
        addTearDown(() => t.binding.setSurfaceSize(null));
        var taps = 0;
        final body = texts[loc]!;
        for (final scale in [1.0, 2.0]) {
          await t.pumpWidget(
            _app(
              c,
              Column(
                children: [
                  ProgressRing(value: 0.4, semanticsLabel: body),
                  for (final s in LsStatus.values)
                    LsBanner(
                      tone: s,
                      message: body,
                      actionLabel: 'Action ${s.name}',
                      onAction: () => taps++,
                    ),
                  EmptyState(title: body, body: body),
                  ErrorState(title: body, body: body, onRetry: () => taps++),
                ],
              ),
              scale: scale,
              locale: loc,
            ),
          );
          expect(t.takeException(), isNull);
        }
        await expectLater(t, meetsGuideline(textContrastGuideline));
        await expectLater(t, meetsGuideline(androidTapTargetGuideline));
        await t.ensureVisible(find.text('Action error'));
        await t.tap(find.text('Action error'));
        await t.ensureVisible(find.byType(FilledButton));
        await t.tap(find.byType(FilledButton));
        expect(taps, 2);
        h.dispose();
      });
    }
  }

  testWidgets('ErrorState default retry label follows locale', (t) async {
    await t.pumpWidget(
      _app(
        LsColors.light,
        ErrorState(title: 'x', onRetry: () {}),
        locale: 'es',
      ),
    );
    expect(find.text('Reintentar'), findsOneWidget);
  });
}
