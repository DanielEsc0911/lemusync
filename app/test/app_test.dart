import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/main.dart';

import 'support/fonts.dart';

void main() {
  setUpAll(loadBrandFonts);

  testWidgets('shows synced status in English', (tester) async {
    await tester.pumpWidget(const LemuSyncApp(locale: Locale('en')));
    await tester.pumpAndSettle();
    expect(find.text('All saves synced'), findsOneWidget);
  });

  testWidgets('shows synced status in Spanish', (tester) async {
    await tester.pumpWidget(const LemuSyncApp(locale: Locale('es')));
    await tester.pumpAndSettle();
    expect(find.text('Todas las partidas están sincronizadas'), findsOneWidget);
  });
}
