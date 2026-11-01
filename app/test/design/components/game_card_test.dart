import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/design/components/cartridge_border.dart';
import 'package:lemusync/design/components/game_card.dart';
import 'package:lemusync/design/status.dart';
import 'package:lemusync/design/theme.dart';
import 'package:lemusync/design/tokens/colors.g.dart';
import 'package:lemusync/design/tokens/space.g.dart';
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
        child: Center(
          child: SizedBox(width: width, child: child),
        ),
      ),
    ),
  ),
);

void main() {
  setUpAll(loadBrandFonts);

  group('CartridgeBorder', () {
    const rect = Rect.fromLTWH(0, 0, 300, 200);
    final path = const CartridgeBorder().getOuterPath(rect);

    test('contains the centre but not the notch', () {
      expect(path.contains(rect.center), isTrue);
      expect(path.contains(const Offset(150, LsSpace.md / 2)), isFalse);
      expect(path.contains(const Offset(150, LsSpace.md + 2)), isTrue);
    });

    test('is whole outside the notch', () {
      expect(path.contains(const Offset(20, 1)), isTrue);
      expect(path.contains(const Offset(280, 1)), isTrue);
      expect(path.contains(const Offset(0, 0)), isFalse); // rounded corner
    });

    test('copyWith and scale keep the shape', () {
      const b = CartridgeBorder();
      expect(b.copyWith(), b);
      expect(b.scale(2), isA<CartridgeBorder>());
    });
  });

  testWidgets('semantics label merges title, emulator, status and lease', (
    t,
  ) async {
    final h = t.ensureSemantics();
    await t.pumpWidget(
      _app(
        LsColors.light,
        const GameCard(
          title: 'Zelda',
          emulator: 'Cemu',
          status: LsStatus.attention,
          lease: 'In use on Desktop',
        ),
      ),
    );
    expect(
      find.bySemanticsLabel(
        'Zelda, Cemu, Needs your attention, In use on Desktop',
      ),
      findsOneWidget,
    );
    h.dispose();
  });

  testWidgets('long Spanish title ellipsizes at width 160', (t) async {
    await t.pumpWidget(
      _app(
        LsColors.light,
        const GameCard(
          title: 'La leyenda de Zelda: el aliento de lo salvaje edición coleccionista',
          emulator: 'Cemu',
          lastSynced: 'hace 2 min',
        ),
        locale: 'es',
        width: 160,
      ),
    );
    expect(t.takeException(), isNull);
    final text = t.widget<Text>(find.textContaining('La leyenda'));
    expect(text.maxLines, 2);
    expect(text.overflow, TextOverflow.ellipsis);
    expect(find.text('Sincronizado hace 2 min'), findsOneWidget);
  });

  for (final loc in ['en', 'es']) {
    testWidgets('200% scale, contrast, targets $loc', (t) async {
      final h = t.ensureSemantics();
      for (final c in [
        LsColors.light,
        LsColors.dark,
        LsColors.lightHighContrast,
        LsColors.darkHighContrast,
      ]) {
        await t.pumpWidget(
          _app(
            c,
            GameCard(
              title: 'Mario Kart 8 Deluxe',
              emulator: 'Ryujinx',
              lastSynced: '2 min',
              lease: 'In use on Desktop',
              status: LsStatus.attention,
              onPlay: () {},
              onTap: () {},
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

  testWidgets('Play and tap fire; Play has its tooltip', (t) async {
    var plays = 0;
    var taps = 0;
    await t.pumpWidget(
      _app(
        LsColors.light,
        GameCard(
          title: 'Zelda',
          emulator: 'Cemu',
          onPlay: () => plays++,
          onTap: () => taps++,
        ),
      ),
    );
    expect(find.byTooltip('Play'), findsOneWidget);
    await t.tap(find.byTooltip('Play'));
    expect(plays, 1);
    expect(taps, 0);
    await t.tap(find.text('Zelda'));
    expect(taps, 1);
  });
}
