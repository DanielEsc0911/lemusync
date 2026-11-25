import 'package:flutter/widgets.dart';

import '../design/icons.dart';
import '../l10n/generated/app_localizations.dart';

/// The places the shell shows, in the spec's order on every platform (F3).
/// Only built destinations are listed; 3d inserts Activity, Devices and
/// Settings where navigation.md puts them. The shortcut digit is `index + 1`.
enum Destination { home, games }

extension DestinationInfo on Destination {
  IconData get icon => switch (this) {
    Destination.home => LsIcons.home,
    Destination.games => LsIcons.games,
  };

  String label(AppLocalizations l10n) => switch (this) {
    Destination.home => l10n.navHome,
    Destination.games => l10n.navGames,
  };

  /// Ctrl/Cmd + this digit selects the destination.
  int get shortcutDigit => index + 1;
}
