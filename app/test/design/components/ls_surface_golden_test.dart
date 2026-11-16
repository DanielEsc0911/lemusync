@Tags(['golden'])
library;

import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/design/components/ls_surface.dart';
import 'package:lemusync/design/tokens/space.g.dart';

import '../../support/fonts.dart';
import '../../support/golden.dart';

void main() {
  setUpAll(loadBrandFonts);

  testWidgets('ls_surface', (t) async {
    await goldenMatrix(
      t,
      'ls_surface',
      () => const Padding(
        padding: EdgeInsets.all(LsSpace.lg),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          spacing: LsSpace.lg,
          children: [
            LsSurface(level: LsSurfaceLevel.sunken, child: Text('Sunken')),
            LsSurface(child: Text('Base')),
            LsSurface(level: LsSurfaceLevel.raised, child: Text('Raised')),
          ],
        ),
      ),
    );
  }, skip: !Platform.isLinux);
}
