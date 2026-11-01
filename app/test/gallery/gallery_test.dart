import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/design/tokens/colors.g.dart';
import 'package:lemusync/gallery/gallery_page.dart';

import '../support/fonts.dart';

Future<void> _open(WidgetTester t, Size size, LsColors c) async {
  t.view.physicalSize = size;
  t.view.devicePixelRatio = 1;
  addTearDown(t.view.reset);
  await t.pumpWidget(GalleryApp(initialTheme: c));
  await t.pump();
}

Future<void> _pick(WidgetTester t, String tooltip, String item) async {
  await t.tap(find.byTooltip(tooltip));
  await t.pump();
  await t.pump(const Duration(seconds: 1));
  await t.tap(find.text(item).last);
  await t.pump();
  await t.pump(const Duration(seconds: 1));
}

void main() {
  setUpAll(loadBrandFonts);

  for (final size in const [
    Size(390, 844),
    Size(900, 1000),
    Size(1400, 1000),
  ]) {
    for (final MapEntry(:key, :value) in galleryThemes.entries) {
      testWidgets('pumps at ${size.width.toInt()} in $key', (t) async {
        await _open(t, size, value);
        expect(t.takeException(), isNull);
        // Scroll through the whole page: every lazy section builds.
        await t.scrollUntilVisible(
          find.text('Empty and error states'),
          600,
          scrollable: find.byType(Scrollable).first,
        );
        for (var i = 0; i < 4; i++) {
          await t.drag(find.byType(CustomScrollView), const Offset(0, -600));
          await t.pump();
        }
        expect(t.takeException(), isNull);
      });
    }
  }

  testWidgets('switching theme and locale updates the samples', (t) async {
    await _open(t, const Size(1400, 1000), LsColors.light);
    expect(find.text('#F6F4F0'), findsWidgets);
    await _pick(t, 'Theme', 'Dark');
    expect(find.text('#F6F4F0'), findsNothing);
    expect(find.text('#2A2623'), findsWidgets);

    await t.scrollUntilVisible(
      find.text('Status badge'),
      600,
      scrollable: find.byType(Scrollable).first,
    );
    expect(find.text('All saves synced'), findsWidgets);
    await _pick(t, 'Language', 'es');
    expect(find.text('Todas las partidas están sincronizadas'), findsWidgets);
    expect(find.text('All saves synced'), findsNothing);
  });

  testWidgets('the performance overlay toggles', (t) async {
    await _open(t, const Size(900, 1000), LsColors.light);
    expect(find.byType(PerformanceOverlay), findsNothing);
    await t.tap(find.byTooltip('Performance overlay'));
    await t.pump();
    expect(find.byType(PerformanceOverlay), findsOneWidget);
  });
}
