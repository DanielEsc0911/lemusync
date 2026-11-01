import 'package:flutter/material.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/design/theme.dart';
import 'package:lemusync/design/tokens/colors.g.dart';
import 'package:lemusync/l10n/generated/app_localizations.dart';

/// Renders [build] in light and dark, en and es, at each size, and compares
/// each to `goldens/<name>.<theme>.<locale>.<w>x<h>.png`.
Future<void> goldenMatrix(
  WidgetTester tester,
  String name,
  Widget Function() build, {
  List<Size> sizes = const [Size(390, 844), Size(900, 1000), Size(1400, 1000)],
  List<ImageProvider> precache = const [],
}) async {
  const themes = {'light': LsColors.light, 'dark': LsColors.dark};
  for (final MapEntry(key: theme, value: colors) in themes.entries) {
    for (final locale in const ['en', 'es']) {
      for (final size in sizes) {
        tester.view.physicalSize = size;
        tester.view.devicePixelRatio = 1;
        addTearDown(tester.view.reset);
        await tester.pumpWidget(
          MaterialApp(
            key: ValueKey('$theme.$locale.$size'),
            debugShowCheckedModeBanner: false,
            theme: lsTheme(colors),
            locale: Locale(locale),
            localizationsDelegates: const [
              AppLocalizations.delegate,
              GlobalMaterialLocalizations.delegate,
              GlobalWidgetsLocalizations.delegate,
              GlobalCupertinoLocalizations.delegate,
            ],
            supportedLocales: AppLocalizations.supportedLocales,
            home: Builder(
              builder: (context) => MediaQuery(
                data: MediaQuery.of(context).copyWith(disableAnimations: true),
                child: Material(child: Center(child: build())),
              ),
            ),
          ),
        );
        for (final p in precache) {
          await tester.runAsync(
            () => precacheImage(p, tester.element(find.byType(Material).first)),
          );
        }
        await tester.pump();
        await expectLater(
          find.byType(MaterialApp),
          matchesGoldenFile(
            'goldens/$name.$theme.$locale.${size.width.toInt()}x${size.height.toInt()}.png',
          ),
        );
      }
    }
  }
}
