import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';

const _entries = <(String, String)>[
  ('Space Grotesk', 'assets/fonts/SpaceGrotesk/OFL.txt'),
  ('Inter', 'assets/fonts/Inter/OFL.txt'),
  ('JetBrains Mono', 'assets/fonts/JetBrainsMono/OFL.txt'),
  ('Lucide', 'assets/licenses/lucide-ISC.txt'),
];

/// Adds the font and icon license texts to the license page.
void registerBrandLicenses() {
  for (final (package, asset) in _entries) {
    LicenseRegistry.addLicense(() async* {
      yield LicenseEntryWithLineBreaks([
        package,
      ], await rootBundle.loadString(asset));
    });
  }
}
