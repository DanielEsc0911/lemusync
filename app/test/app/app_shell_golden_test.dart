@Tags(['golden'])
library;

import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/app/app_shell.dart';
import 'package:lemusync/app/destination.dart';

import '../support/fonts.dart';
import '../support/golden.dart';

void main() {
  setUpAll(loadBrandFonts);

  testWidgets('app shell', (t) async {
    await goldenMatrix(
      t,
      'app_shell',
      () => Builder(
        builder: (context) => SizedBox.fromSize(
          size: MediaQuery.sizeOf(context),
          child: AppShell(
            current: Destination.games,
            onSelect: (_) {},
            body: const SizedBox.expand(),
          ),
        ),
      ),
      sizes: const [Size(390, 700), Size(900, 700), Size(1400, 700)],
      locales: const [Locale('en')],
    );
  }, skip: !Platform.isLinux);
}
