@Tags(['golden'])
library;

import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/design/components/status_badge.dart';
import 'package:lemusync/design/status.dart';
import 'package:lemusync/design/tokens/space.g.dart';

import '../../support/fonts.dart';
import '../../support/golden.dart';

void main() {
  setUpAll(loadBrandFonts);

  testWidgets('status_badge', (t) async {
    await goldenMatrix(
      t,
      'status_badge',
      () => Padding(
        padding: const EdgeInsets.all(LsSpace.lg),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.start,
          spacing: LsSpace.sm,
          children: [
            for (final s in LsStatus.values) StatusBadge(status: s),
            const SizedBox(height: LsSpace.sm),
            Wrap(
              spacing: LsSpace.sm,
              children: [
                for (final s in LsStatus.values)
                  StatusBadge(status: s, compact: true),
              ],
            ),
          ],
        ),
      ),
    );
  }, skip: !Platform.isLinux);
}
