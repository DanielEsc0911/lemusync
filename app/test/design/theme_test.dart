import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/design/theme.dart';
import 'package:lemusync/design/tokens/colors.g.dart';
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
    test('lsTheme maps the tokens ($name)', () {
      final t = lsTheme(c);
      expect(t.scaffoldBackgroundColor, c.surface);
      expect(t.colorScheme.primary, c.accent);
      expect(t.colorScheme.brightness, c.brightness);
      expect(t.extension<LsColors>(), c);
      expect(t.textTheme.bodyMedium!.fontFamily, LsType.fontBody);
      expect(t.useMaterial3, isTrue);
      expect(t.splashFactory, NoSplash.splashFactory);
      expect(t.focusColor, c.focus);
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
}
