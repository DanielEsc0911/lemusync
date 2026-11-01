import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/design/components/ls_surface.dart';
import 'package:lemusync/design/tokens/colors.g.dart';
import 'package:lemusync/design/tokens/shape.g.dart';

import '../../support/fonts.dart';

void main() {
  setUpAll(loadBrandFonts);

  const fills = {
    LsSurfaceLevel.sunken: 'sunken',
    LsSurfaceLevel.base: 'base',
    LsSurfaceLevel.raised: 'raised',
  };

  for (final level in fills.keys) {
    for (final radius in [0.0, LsShape.radiusLg]) {
      testWidgets('paints fill and bevel (${level.name}, r=$radius)', (
        t,
      ) async {
        const c = LsColors.dark;
        await t.pumpWidget(
          MaterialApp(
            theme: ThemeData(extensions: const [c]),
            home: LsSurface(
              level: level,
              radius: radius,
              child: const Text('x'),
            ),
          ),
        );
        expect(t.takeException(), isNull);
        final boxes = t
            .widgetList<DecoratedBox>(
              find.descendant(
                of: find.byType(LsSurface),
                matching: find.byType(DecoratedBox),
              ),
            )
            .map((b) => b.decoration as BoxDecoration)
            .toList();
        final outer = boxes.first.gradient! as LinearGradient;
        expect(outer.colors, [c.bevelLight, c.bevelDark]);
        final fill = switch (level) {
          LsSurfaceLevel.sunken => c.surfaceSunken,
          LsSurfaceLevel.base => c.surface,
          LsSurfaceLevel.raised => c.surfaceRaised,
        };
        expect(boxes.any((b) => b.color == fill), isTrue);
      });
    }
  }
}
