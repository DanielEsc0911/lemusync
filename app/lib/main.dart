import 'package:flutter/material.dart';

import 'design/licenses.dart';
import 'design/theme.dart';
import 'design/tokens/colors.g.dart';
import 'l10n/generated/app_localizations.dart';

void main() {
  registerBrandLicenses();
  runApp(const LemuSyncApp());
}

/// Root widget. The design system and navigation shell replace the
/// placeholder home in Phase 3 (docs/specs/ui/).
class LemuSyncApp extends StatelessWidget {
  const LemuSyncApp({super.key, this.locale});

  /// Forces a locale (tests, user override). Null follows the OS.
  final Locale? locale;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      onGenerateTitle: (context) => AppLocalizations.of(context).appTitle,
      locale: locale,
      theme: lsTheme(LsColors.light),
      darkTheme: lsTheme(LsColors.dark),
      highContrastTheme: lsTheme(LsColors.lightHighContrast),
      highContrastDarkTheme: lsTheme(LsColors.darkHighContrast),
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      home: const _StatusPlaceholder(),
    );
  }
}

class _StatusPlaceholder extends StatelessWidget {
  const _StatusPlaceholder();

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: Center(
        child: Text(
          AppLocalizations.of(context).statusSynced,
          style: Theme.of(context).textTheme.headlineMedium,
        ),
      ),
    );
  }
}
