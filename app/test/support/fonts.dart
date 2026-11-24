import 'dart:convert';

import 'package:flutter/services.dart';
import 'package:flutter_test/flutter_test.dart';

bool _loaded = false;

/// Loads every font family of FontManifest.json (brand fonts and the icon
/// package's) so tests measure real metrics. Safe to call repeatedly.
Future<void> loadBrandFonts() async {
  TestWidgetsFlutterBinding.ensureInitialized();
  if (_loaded) return;
  final manifest = json.decode(
    await rootBundle.loadString('FontManifest.json'),
  ) as List<dynamic>;
  for (final entry in manifest) {
    final family = (entry as Map<String, dynamic>)['family'] as String;
    final loader = FontLoader(family);
    for (final font in entry['fonts'] as List<dynamic>) {
      final asset = (font as Map<String, dynamic>)['asset'] as String;
      loader.addFont(rootBundle.load(asset));
    }
    await loader.load();
  }
  _loaded = true;
}
