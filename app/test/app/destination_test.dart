import 'package:flutter/widgets.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:lemusync/app/destination.dart';
import 'package:lemusync/l10n/generated/app_localizations.dart';

void main() {
  test('order is the spec order and digits are index + 1', () {
    expect(Destination.values, [Destination.home, Destination.games]);
    expect(Destination.values.map((d) => d.shortcutDigit), [1, 2]);
  });

  test('icons are distinct', () {
    final icons = Destination.values.map((d) => d.icon).toSet();
    expect(icons.length, Destination.values.length);
  });

  test('labels in en and es', () async {
    final en = await AppLocalizations.delegate.load(const Locale('en'));
    final es = await AppLocalizations.delegate.load(const Locale('es'));
    expect(Destination.values.map((d) => d.label(en)), ['Home', 'Games']);
    expect(Destination.values.map((d) => d.label(es)), ['Inicio', 'Juegos']);
  });
}
