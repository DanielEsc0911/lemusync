import 'package:flutter/widgets.dart';

import '../l10n/generated/app_localizations.dart';
import 'icons.dart';
import 'tokens/colors.g.dart';

/// Sync state shown to the user. Same order and meaning as `api::StatusState`.
enum LsStatus { synced, scanning, syncing, paused, offline, attention, error }

/// Color role, icon and label of each status (brand spec, Palette).
extension LsStatusStyle on LsStatus {
  Color color(LsColors c) => switch (this) {
    LsStatus.synced => c.synced,
    LsStatus.scanning || LsStatus.syncing => c.syncing,
    LsStatus.paused || LsStatus.offline => c.offline,
    LsStatus.attention => c.attention,
    LsStatus.error => c.danger,
  };

  Color onColor(LsColors c) => switch (this) {
    LsStatus.synced => c.onSynced,
    LsStatus.scanning || LsStatus.syncing => c.onSyncing,
    LsStatus.paused || LsStatus.offline => c.onOffline,
    LsStatus.attention => c.onAttention,
    LsStatus.error => c.onDanger,
  };

  IconData get icon => switch (this) {
    LsStatus.synced => LsIcons.synced,
    LsStatus.scanning || LsStatus.syncing => LsIcons.syncing,
    LsStatus.paused => LsIcons.paused,
    LsStatus.offline => LsIcons.offline,
    LsStatus.attention => LsIcons.attention,
    LsStatus.error => LsIcons.error,
  };

  String label(AppLocalizations l) => switch (this) {
    LsStatus.synced => l.statusSynced,
    LsStatus.scanning => l.statusScanning,
    LsStatus.syncing => l.statusSyncing,
    LsStatus.paused => l.statusPaused,
    LsStatus.offline => l.statusOffline,
    LsStatus.attention => l.statusAttention,
    LsStatus.error => l.statusError,
  };
}
