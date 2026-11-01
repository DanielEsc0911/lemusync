import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/design/components/ls_surface.dart';
import 'package:lemusync/design/tokens/colors.g.dart';
import 'package:lemusync/design/tokens/shape.g.dart';
import 'package:lemusync/design/tokens/size.g.dart';
import 'package:lemusync/design/tokens/space.g.dart';

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
        expect(boxes, hasLength(2));
        final outer = boxes[0];
        final inner = boxes[1];
        final grad = outer.gradient! as LinearGradient;
        expect(
          grad.colors,
          level == LsSurfaceLevel.sunken
              ? [c.bevelDark, c.bevelLight]
              : [c.bevelLight, c.bevelDark],
        );
        final fill = switch (level) {
          LsSurfaceLevel.sunken => c.surfaceSunken,
          LsSurfaceLevel.base => c.surface,
          LsSurfaceLevel.raised => c.surfaceRaised,
        };
        expect(inner.color, fill);
        expect(outer.color, isNull);
        expect(outer.borderRadius, BorderRadius.circular(radius));
        expect(
          inner.borderRadius,
          BorderRadius.circular(math.max(0, radius - LsSize.bevel)),
        );
        for (final b in boxes) {
          expect(b.boxShadow, isNull);
          expect(b.border, isNull);
        }
        final pads = t
            .widgetList<Padding>(
              find.descendant(
                of: find.byType(LsSurface),
                matching: find.byType(Padding),
              ),
            )
            .map((p) => p.padding)
            .toList();
        expect(
          pads,
          containsAllInOrder(const [
            EdgeInsets.all(LsSize.bevel),
            EdgeInsets.all(LsSpace.lg),
          ]),
        );
      });
    }
  }
}
