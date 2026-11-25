import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/design/tokens/colors.g.dart';
import 'package:lemusync/main.dart';

import 'support/fake_link.dart';
import 'support/fonts.dart';

void main() {
  setUpAll(loadBrandFonts);

  testWidgets('opens on Home in English', (tester) async {
    await tester.pumpWidget(
      LemuSyncApp(link: FakeDaemonLink(), locale: const Locale('en')),
    );
    await tester.pumpAndSettle();
    expect(find.text('Home'), findsWidgets);
  });

  testWidgets('opens on Inicio in Spanish', (tester) async {
    await tester.pumpWidget(
      LemuSyncApp(link: FakeDaemonLink(), locale: const Locale('es')),
    );
    await tester.pumpAndSettle();
    expect(find.text('Inicio'), findsWidgets);
  });

  testWidgets('high contrast uses the high-contrast color set', (tester) async {
    await tester.pumpWidget(
      MediaQuery(
        data: const MediaQueryData(highContrast: true),
        child: LemuSyncApp(link: FakeDaemonLink(), locale: const Locale('en')),
      ),
    );
    await tester.pumpAndSettle();
    final ctx = tester.element(find.text('Home').first);
    expect(
      Theme.of(ctx).scaffoldBackgroundColor,
      LsColors.lightHighContrast.surface,
    );
  });
}
