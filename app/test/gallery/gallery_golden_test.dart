@Tags(['golden'])
library;

import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/design/tokens/colors.g.dart';
import 'package:lemusync/gallery/gallery_page.dart';

import '../support/fonts.dart';
import '../support/golden.dart';

Widget _page() => Builder(
  builder: (context) => SizedBox.fromSize(
    size: MediaQuery.sizeOf(context),
    child: GalleryPage(
      colors: Theme.of(context).extension<LsColors>()!,
      onColors: (_) {},
      locale: Localizations.localeOf(context),
      onLocale: (_) {},
      overlay: false,
      onOverlay: (_) {},
    ),
  ),
);

/// Scrolls so the section titled [title] sits at the top of the viewport.
Future<void> _scrollTo(WidgetTester t, String title) async {
  final scrollable = find.byType(Scrollable).first;
  await t.scrollUntilVisible(find.text(title), 400, scrollable: scrollable);
  await Scrollable.ensureVisible(t.element(find.text(title)), alignment: 0.1);
  await t.pump();
}

void main() {
  setUpAll(loadBrandFonts);

  // Top of the page: the token sections have no text to translate, en only.
  testWidgets('gallery', (t) async {
    await goldenMatrix(t, 'gallery', _page, locales: const [Locale('en')]);
  }, skip: !Platform.isLinux);

  // The sections with localized strings, in en and es.
  for (final (name, title) in const [
    ('gallery-hero', 'Status hero'),
    ('gallery-cards', 'Game card'),
  ]) {
    testWidgets(name, (t) async {
      await goldenMatrix(t, name, _page, settle: (t) => _scrollTo(t, title));
    }, skip: !Platform.isLinux);
  }
}
