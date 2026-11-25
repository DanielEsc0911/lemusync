import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/design/theme.dart';
import 'package:lemusync/design/tokens/colors.g.dart';
import 'package:lemusync/design/tokens/opacity.g.dart';
import 'package:lemusync/design/tokens/shape.g.dart';
import 'package:lemusync/design/tokens/size.g.dart';
import 'package:lemusync/design/tokens/space.g.dart';
import 'package:lemusync/design/tokens/type.g.dart';

import '../support/fonts.dart';

void main() {
  setUpAll(loadBrandFonts);

  const all = {
    'light': LsColors.light,
    'dark': LsColors.dark,
    'lightHighContrast': LsColors.lightHighContrast,
    'darkHighContrast': LsColors.darkHighContrast,
  };

  for (final MapEntry(key: name, value: c) in all.entries) {
    test('lsTheme styles the navigation from tokens ($name)', () {
      final t = lsTheme(c);
      final bar = t.navigationBarTheme;
      final rail = t.navigationRailTheme;
      expect(bar.backgroundColor, c.surfaceSunken);
      expect(rail.backgroundColor, c.surfaceSunken);
      expect(bar.indicatorColor, c.accent);
      expect(rail.indicatorColor, c.accent);
      expect(bar.height, LsSize.target + LsSpace.lg);
      expect(bar.surfaceTintColor, Colors.transparent);
      expect(bar.elevation, 0);
      expect(rail.elevation, 0);
      expect(rail.minWidth, LsSize.railWidth);
      const selected = {WidgetState.selected};
      expect(bar.iconTheme!.resolve(selected)!.color, c.onAccent);
      expect(bar.iconTheme!.resolve({})!.color, c.onSurfaceMuted);
      expect(bar.labelTextStyle!.resolve(selected)!.color, c.onSurface);
      expect(
        bar.labelTextStyle!.resolve(selected)!.fontWeight,
        isNot(bar.labelTextStyle!.resolve({})!.fontWeight),
      );
      expect(rail.selectedIconTheme!.color, c.onAccent);
      expect(rail.unselectedIconTheme!.color, c.onSurfaceMuted);
      expect(rail.selectedLabelTextStyle!.color, c.onSurface);
      expect(rail.unselectedLabelTextStyle!.color, c.onSurfaceMuted);
    });

    test('lsTheme maps the tokens ($name)', () {
      final t = lsTheme(c);
      expect(t.scaffoldBackgroundColor, c.surface);
      expect(t.colorScheme.primary, c.accent);
      expect(t.colorScheme.brightness, c.brightness);
      expect(t.extension<LsColors>(), c);
      expect(t.textTheme.bodyMedium!.fontFamily, LsType.fontBody);
      expect(t.useMaterial3, isTrue);
      expect(t.splashFactory, NoSplash.splashFactory);
      expect(t.focusColor, c.focus.withValues(alpha: LsOpacity.focus));
      expect(t.hoverColor, c.onSurface.withValues(alpha: LsOpacity.hover));
      expect(c.focus, c.onSurface);
      expect(t.visualDensity, VisualDensity.standard);
      expect(t.materialTapTargetSize, MaterialTapTargetSize.padded);
    });
  }

  double width(FontWeight w) {
    final p = TextPainter(
      text: TextSpan(
        text: 'Sync',
        style: TextStyle(
          fontFamily: LsType.fontBody,
          fontSize: 40,
          fontWeight: w,
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    return p.width;
  }

  test('the bundled body font is variable: w700 is wider than w400', () {
    expect(width(FontWeight.w700), greaterThan(width(FontWeight.w400)));
  });

  Widget host(LsColors c, Widget child) => MaterialApp(
    theme: lsTheme(c),
    home: Scaffold(body: Center(child: child)),
  );

  Material materialOf(WidgetTester tester, Finder f) => tester.widget<Material>(
    find.descendant(of: f, matching: find.byType(Material)).first,
  );

  for (final platform in [
    TargetPlatform.linux,
    TargetPlatform.macOS,
    TargetPlatform.windows,
  ]) {
    testWidgets('buttons keep the target size on $platform', (tester) async {
      debugDefaultTargetPlatformOverride = platform;
      try {
        final keys = List.generate(4, (_) => UniqueKey());
        await tester.pumpWidget(
          host(
            LsColors.light,
            Column(
              children: [
                FilledButton(
                  key: keys[0],
                  onPressed: () {},
                  child: const Text('a'),
                ),
                OutlinedButton(
                  key: keys[1],
                  onPressed: () {},
                  child: const Text('a'),
                ),
                TextButton(
                  key: keys[2],
                  onPressed: () {},
                  child: const Text('a'),
                ),
                IconButton(
                  key: keys[3],
                  onPressed: () {},
                  icon: const Icon(Icons.add),
                ),
              ],
            ),
          ),
        );
        for (final k in keys) {
          final size = tester.getSize(find.byKey(k));
          expect(size.width, greaterThanOrEqualTo(LsSize.target));
          expect(size.height, greaterThanOrEqualTo(LsSize.target));
        }
      } finally {
        debugDefaultTargetPlatformOverride = null;
      }
    });
  }

  for (final MapEntry(key: name, value: c) in all.entries) {
    testWidgets('disabled filled button is muted ($name)', (tester) async {
      final key = UniqueKey();
      await tester.pumpWidget(
        host(
          c,
          FilledButton(key: key, onPressed: null, child: const Text('Go')),
        ),
      );
      final m = materialOf(tester, find.byKey(key));
      expect(m.color, isNot(c.accent));
      expect(m.color, c.onSurface.withValues(alpha: LsOpacity.pressed));
      final style = tester
          .widget<DefaultTextStyle>(
            find
                .descendant(
                  of: find.byKey(key),
                  matching: find.byType(DefaultTextStyle),
                )
                .first,
          )
          .style;
      expect(style.color!.a, closeTo(LsOpacity.disabled, 0.01));
    });

    testWidgets('focused buttons draw the ring outside ($name)', (
      tester,
    ) async {
      final fn1 = FocusNode();
      final fn2 = FocusNode();
      addTearDown(fn1.dispose);
      addTearDown(fn2.dispose);
      final k1 = UniqueKey();
      final k2 = UniqueKey();
      final k3 = UniqueKey();
      await tester.pumpWidget(
        host(
          c,
          Column(
            children: [
              OutlinedButton(
                key: k1,
                focusNode: fn1,
                onPressed: () {},
                child: const Text('a'),
              ),
              FilledButton(
                key: k2,
                focusNode: fn2,
                onPressed: () {},
                child: const Text('a'),
              ),
              OutlinedButton(key: k3, onPressed: () {}, child: const Text('a')),
            ],
          ),
        ),
      );
      BorderSide sideOf(Key k) =>
          (materialOf(tester, find.byKey(k)).shape! as RoundedRectangleBorder)
              .side;
      final idle = sideOf(k3);
      expect(idle.color, c.outline);
      expect(idle.width, LsSize.bevel);
      FocusManager.instance.highlightStrategy =
          FocusHighlightStrategy.alwaysTraditional;
      fn1.requestFocus();
      await tester.pump();
      await tester.pump(const Duration(seconds: 1));
      for (final (k, fn) in [(k1, fn1)]) {
        expect(fn.hasFocus, isTrue);
        final side = sideOf(k);
        expect(side.color, c.focus);
        expect(side.width, LsSize.focusRing);
        expect(side.strokeAlign, BorderSide.strokeAlignOutside);
      }
      fn2.requestFocus();
      await tester.pump();
      await tester.pump(const Duration(seconds: 1));
      final side = sideOf(k2);
      expect(side.color, c.focus);
      expect(side.width, LsSize.focusRing);
      expect(side.strokeAlign, BorderSide.strokeAlignOutside);
      expect(
        (materialOf(tester, find.byKey(k2)).shape! as RoundedRectangleBorder)
            .borderRadius,
        BorderRadius.circular(LsShape.radiusMd),
      );
    });
  }

  testWidgets('icon buttons are 48 px and the tonal icon is legible', (
    tester,
  ) async {
    final k = UniqueKey();
    final kt = UniqueKey();
    const c = LsColors.light;
    await tester.pumpWidget(
      host(
        c,
        Column(
          children: [
            IconButton(key: k, onPressed: () {}, icon: const Icon(Icons.add)),
            IconButton.filledTonal(
              key: kt,
              onPressed: () {},
              icon: const Icon(Icons.add),
            ),
          ],
        ),
      ),
    );
    expect(tester.getSize(find.byKey(k)), const Size.square(LsSize.target));
    final icon = tester.widget<Icon>(
      find.descendant(of: find.byKey(kt), matching: find.byType(Icon)),
    );
    final iconColor = IconTheme.of(
      tester.element(
        find.descendant(of: find.byKey(kt), matching: find.byType(Icon)),
      ),
    ).color!;
    expect(icon, isNotNull);
    final bg = materialOf(tester, find.byKey(kt)).color!;
    double lum(Color x) => x.computeLuminance();
    final hi = lum(iconColor) > lum(bg) ? lum(iconColor) : lum(bg);
    final lo = lum(iconColor) > lum(bg) ? lum(bg) : lum(iconColor);
    expect((hi + 0.05) / (lo + 0.05), greaterThanOrEqualTo(3.0));
    expect(iconColor, c.onAccent);
  });

  test('primary text theme and input borders', () {
    const c = LsColors.light;
    final t = lsTheme(c);
    expect(t.primaryTextTheme.bodyMedium!.color, c.onAccent);
    final d = t.inputDecorationTheme;
    expect(
      (d.focusedErrorBorder! as OutlineInputBorder).borderSide,
      BorderSide(color: c.danger, width: LsSize.focusRing),
    );
    expect(
      (d.disabledBorder! as OutlineInputBorder).borderSide,
      BorderSide(
        color: c.outline.withValues(alpha: LsOpacity.disabled),
        width: LsSize.bevel,
      ),
    );
  });
}
