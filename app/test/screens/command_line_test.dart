import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/app/app_controller.dart';
import 'package:lemusync/screens/command_line.dart';

import '../support/fake_link.dart';
import '../support/fonts.dart';
import 'harness.dart';

void main() {
  setUpAll(loadBrandFonts);

  testWidgets('shows the command, selectable, not tappable', (t) async {
    final c = AppController(FakeDaemonLink());
    addTearDown(c.dispose);
    await t.pumpWidget(
      harness(c, const Center(child: CommandLine(command: 'lemusync doctor'))),
    );
    expect(find.text('lemusync doctor'), findsOneWidget);
    expect(find.byType(SelectableText), findsOneWidget);
    expect(
      find.descendant(
        of: find.byType(CommandLine),
        matching: find.byType(InkWell),
      ),
      findsNothing,
    );
    expect(
      find.descendant(
        of: find.byType(CommandLine),
        matching: find.byType(ButtonStyleButton),
      ),
      findsNothing,
    );
  });

  for (final MapEntry(key: name, value: colors) in themes.entries) {
    testWidgets('contrast in $name', (t) async {
      final h = t.ensureSemantics();
      final c = AppController(FakeDaemonLink());
      addTearDown(c.dispose);
      await t.pumpWidget(
        harness(
          c,
          const Center(child: CommandLine(command: 'lemusync doctor')),
          colors: colors,
        ),
      );
      await expectLater(t, meetsGuideline(textContrastGuideline));
      h.dispose();
    });
  }
}
