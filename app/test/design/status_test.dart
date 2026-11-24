import 'dart:math' as math;

import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/design/icons.dart';
import 'package:lemusync/design/status.dart';
import 'package:lemusync/design/tokens/colors.g.dart';
import 'package:lemusync/l10n/generated/app_localizations_en.dart';
import 'package:lemusync/l10n/generated/app_localizations_es.dart';

import '../support/fonts.dart';

void main() {
  setUpAll(loadBrandFonts);

  const themes = {
    'light': LsColors.light,
    'dark': LsColors.dark,
    'lightHighContrast': LsColors.lightHighContrast,
    'darkHighContrast': LsColors.darkHighContrast,
  };

  double contrast(Color a, Color b) {
    final la = a.computeLuminance(), lb = b.computeLuminance();
    return (math.max(la, lb) + 0.05) / (math.min(la, lb) + 0.05);
  }

  test('order matches api::StatusState', () {
    expect(LsStatus.values.map((s) => s.name), [
      'synced',
      'scanning',
      'syncing',
      'paused',
      'offline',
      'attention',
      'error',
    ]);
  });

  for (final MapEntry(key: name, value: c) in themes.entries) {
    test('onColor contrast over color ($name)', () {
      final min = name.endsWith('HighContrast') ? 7.0 : 4.5;
      for (final s in LsStatus.values) {
        expect(
          contrast(s.onColor(c), s.color(c)),
          greaterThanOrEqualTo(min),
          reason: s.name,
        );
      }
    });

    test('roles per brand palette ($name)', () {
      final expected = <LsStatus, (Color, Color)>{
        LsStatus.synced: (c.synced, c.onSynced),
        LsStatus.scanning: (c.syncing, c.onSyncing),
        LsStatus.syncing: (c.syncing, c.onSyncing),
        LsStatus.paused: (c.offline, c.onOffline),
        LsStatus.offline: (c.offline, c.onOffline),
        LsStatus.attention: (c.attention, c.onAttention),
        LsStatus.error: (c.danger, c.onDanger),
      };
      for (final s in LsStatus.values) {
        expect(s.color(c), expected[s]!.$1, reason: s.name);
        expect(s.onColor(c), expected[s]!.$2, reason: s.name);
      }
    });
  }

  test('icons', () {
    expect(LsStatus.values.map((s) => s.icon), [
      LsIcons.synced,
      LsIcons.syncing,
      LsIcons.syncing,
      LsIcons.paused,
      LsIcons.offline,
      LsIcons.attention,
      LsIcons.error,
    ]);
  });

  test('labels in en and es', () {
    final en = AppLocalizationsEn();
    final es = AppLocalizationsEs();
    expect(LsStatus.values.map((s) => s.label(en)), [
      'All saves synced',
      'Checking for changes',
      'Syncing',
      'Sync paused',
      'Offline',
      'Needs your attention',
      'Something went wrong',
    ]);
    expect(LsStatus.values.map((s) => s.label(es)), [
      'Todas las partidas están sincronizadas',
      'Buscando cambios',
      'Sincronizando',
      'Sincronización en pausa',
      'Sin conexión',
      'Necesita tu atención',
      'Algo salió mal',
    ]);
  });
}
