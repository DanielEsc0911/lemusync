import 'package:flutter/material.dart';
import 'package:flutter/semantics.dart';
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

    test(
      'dimensions cover the notch and every edge; notch is fixed on scale',
      () {
        const b = CartridgeBorder(side: BorderSide(width: 2));
        expect(
          b.dimensions,
          const EdgeInsets.fromLTRB(2, CartridgeBorder.notchDepth + 2, 2, 2),
        );
        expect(
          b.scale(2).dimensions,
          const EdgeInsets.fromLTRB(4, CartridgeBorder.notchDepth + 4, 4, 4),
        );
      },
    );

    test('copyWith and scale keep the shape', () {
      const b = CartridgeBorder();
      expect(b.copyWith(), b);
      expect(b.scale(2), isA<CartridgeBorder>());
    });

    test('lerps radius and side', () {
      const a = CartridgeBorder(radius: 10, side: BorderSide(width: 2));
      const b = CartridgeBorder(radius: 30, side: BorderSide(width: 4));
      final m = a.lerpTo(b, 0.5)! as CartridgeBorder;
      expect(m.radius, 20);
      expect(m.side.width, 3);
      final n = b.lerpFrom(a, 0.5)! as CartridgeBorder;
      expect(n.radius, 20);
      expect(ShapeBorder.lerp(a, b, 0.5), m);
    });

    test('a very narrow card still has a valid path', () {
      const narrow = Rect.fromLTWH(0, 0, 60, 120);
      final p = const CartridgeBorder().getOuterPath(narrow);
      expect(p.getBounds().width, closeTo(60, 0.01));
      expect(p.contains(narrow.center), isTrue);
      const tiny = Rect.fromLTWH(0, 0, 20, 120);
      final q = const CartridgeBorder().getOuterPath(tiny);
      expect(q.contains(tiny.center), isTrue);
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

  testWidgets('the card is a button only when onTap is set', (t) async {
    final h = t.ensureSemantics();
    await t.pumpWidget(
      _app(
        LsColors.light,
        GameCard(title: 'Zelda', emulator: 'Cemu', onTap: () {}),
      ),
    );
    var node = t.getSemantics(find.bySemanticsLabel(RegExp('^Zelda, Cemu')));
    expect(node.getSemanticsData().flagsCollection.isButton, isTrue);
    expect(node.getSemanticsData().hasAction(SemanticsAction.tap), isTrue);
    await t.pumpWidget(
      _app(LsColors.light, const GameCard(title: 'Zelda', emulator: 'Cemu')),
    );
    node = t.getSemantics(find.bySemanticsLabel(RegExp('^Zelda, Cemu')));
    expect(node.getSemanticsData().flagsCollection.isButton, isFalse);
    h.dispose();
  });

  testWidgets('height is intrinsic in a bounded parent', (t) async {
    await t.pumpWidget(
      MaterialApp(
        theme: lsTheme(LsColors.light),
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        home: const Align(
          alignment: Alignment.topLeft,
          child: SizedBox(
            width: 320,
            height: 600,
            child: Align(
              alignment: Alignment.topLeft,
              child: GameCard(title: 'Zelda', emulator: 'Cemu'),
            ),
          ),
        ),
      ),
    );
    expect(t.takeException(), isNull);
    expect(t.getSize(find.byType(GameCard)).height, lessThan(300));
  });

  testWidgets('label adds last synced; badge and info are not extra nodes', (
    t,
  ) async {
    final h = t.ensureSemantics();
    await t.pumpWidget(
      _app(
        LsColors.light,
        GameCard(
          title: 'Zelda',
          emulator: 'Cemu',
          lastSynced: '2 min ago',
          onTap: () {},
          onPlay: () {},
        ),
      ),
    );
    expect(
      find.bySemanticsLabel('Zelda, Cemu, All saves synced, Synced 2 min ago'),
      findsOneWidget,
    );
    expect(find.bySemanticsLabel('Zelda'), findsNothing);
    final play = t.getSemantics(find.byTooltip('Play'));
    expect(play.getSemanticsData().hasAction(SemanticsAction.tap), isTrue);
    expect(play.label, isNot(contains('Zelda')));
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
        await expectLater(t, meetsGuideline(labeledTapTargetGuideline));
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
