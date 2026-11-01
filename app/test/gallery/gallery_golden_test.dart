@Tags(['golden'])
library;

import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/design/tokens/colors.g.dart';
import 'package:lemusync/gallery/gallery_page.dart';

import '../support/fonts.dart';
import '../support/golden.dart';

void main() {
  setUpAll(loadBrandFonts);

  testWidgets('gallery', (t) async {
    await goldenMatrix(
      t,
      'gallery',
      () => Builder(
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
      ),
    );
  }, skip: !Platform.isLinux);
}
