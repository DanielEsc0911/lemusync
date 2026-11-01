@Tags(['golden'])
library;

import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/design/components/empty_state.dart';
import 'package:lemusync/design/components/error_state.dart';
import 'package:lemusync/design/components/ls_banner.dart';
import 'package:lemusync/design/components/progress_ring.dart';
import 'package:lemusync/design/status.dart';
import 'package:lemusync/design/tokens/size.g.dart';
import 'package:lemusync/design/tokens/space.g.dart';

import '../../support/fonts.dart';
import '../../support/golden.dart';

String _t(BuildContext c, String en, String es) =>
    Localizations.localeOf(c).languageCode == 'es' ? es : en;

Future<void> _localized(
  WidgetTester t,
  String name,
  Widget Function(BuildContext) build, {
  List<Size>? sizes,
  List<ImageProvider> precache = const [],
}) => goldenMatrix(
  t,
  name,
  () => Builder(builder: build),
  sizes: sizes!,
  precache: precache,
);

void main() {
  setUpAll(loadBrandFonts);

  testWidgets('progress_ring', (t) async {
    await goldenMatrix(
      t,
      'progress_ring',
      () => const Padding(
        padding: EdgeInsets.all(LsSpace.lg),
        child: Wrap(
          spacing: LsSpace.lg,
          runSpacing: LsSpace.lg,
          children: [
            ProgressRing(value: 0),
            ProgressRing(value: 0.3),
            ProgressRing(value: 0.5, status: LsStatus.synced),
            ProgressRing(value: 1, status: LsStatus.synced),
            ProgressRing(),
            ProgressRing(value: 0.5, size: LsSize.mark),
            ProgressRing(size: LsSize.mark),
          ],
        ),
      ),
      sizes: const [Size(390, 300)],
      locales: const [Locale('en')],
    );
  }, skip: !Platform.isLinux);

  testWidgets('ls_banner', (t) async {
    await _localized(
      t,
      'ls_banner',
      (c) => Padding(
        padding: const EdgeInsets.all(LsSpace.lg),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          spacing: LsSpace.sm,
          children: [
            LsBanner(
              tone: LsStatus.synced,
              message: _t(c, 'Everything is saved', 'Todo está guardado'),
              actionLabel: _t(c, 'View', 'Ver'),
              onAction: () {},
            ),
            LsBanner(
              tone: LsStatus.attention,
              message: _t(
                c,
                'Choose which save to keep',
                'Elige qué partida conservar',
              ),
            ),
            LsBanner(
              tone: LsStatus.error,
              message: _t(
                c,
                'Could not reach your storage',
                'No se pudo conectar con tu almacenamiento',
              ),
              actionLabel: _t(c, 'Try again', 'Reintentar'),
              onAction: () {},
            ),
            LsBanner(
              tone: LsStatus.offline,
              message: _t(c, 'You are offline', 'Sin conexión'),
            ),
          ],
        ),
      ),
      sizes: const [Size(390, 500), Size(900, 500)],
    );
  }, skip: !Platform.isLinux);

  testWidgets('empty_state', (t) async {
    await _localized(
      t,
      'empty_state',
      (c) => EmptyState(
        title: _t(c, 'No games yet', 'Aún no hay juegos'),
        body: _t(
          c,
          'Add an emulator and its saves will show up here.',
          'Agrega un emulador y sus partidas aparecerán aquí.',
        ),
        action: FilledButton(
          onPressed: () {},
          child: Text(_t(c, 'Add', 'Agregar')),
        ),
      ),
      sizes: const [Size(390, 500), Size(900, 500)],
      precache: const [AssetImage('assets/brand/mark.png')],
    );
  }, skip: !Platform.isLinux);

  testWidgets('error_state', (t) async {
    await _localized(
      t,
      'error_state',
      (c) => ErrorState(
        title: _t(c, 'Something went wrong', 'Algo salió mal'),
        body: _t(
          c,
          'We could not reach your storage.',
          'No pudimos conectar con tu almacenamiento.',
        ),
        onRetry: () {},
      ),
      sizes: const [Size(390, 500), Size(900, 500)],
    );
  }, skip: !Platform.isLinux);
}
