import 'package:flutter/foundation.dart';

import '../../app/app_state.dart';
import '../../bridge/link/model.dart' as dto;
import '../../design/status.dart';
import '../../l10n/generated/app_localizations.dart';

/// What `StatusHero` shows for an [AppState].
@immutable
class HeroModel {
  const HeroModel({
    required this.status,
    this.detail,
    this.uploads = 0,
    this.downloads = 0,
  });

  final LsStatus status;
  final String? detail;
  final int uploads;
  final int downloads;
}

/// Maps the daemon's state to the hero (status-and-notifications, Status
/// model). Pure: the same state and locale always give the same hero.
HeroModel heroFor(AppState state, AppLocalizations l) {
  final s = state.status;
  final status = state.lsStatus;
  if (s == null || status == null) {
    return const HeroModel(status: LsStatus.scanning);
  }
  final String? detail;
  if (state.playing) {
    detail = l.homePlaying(_gameName(state.session!), state.session!.player);
  } else {
    detail = switch (status) {
      LsStatus.offline when s.queued > 0 => l.homeOfflineQueued(s.queued),
      LsStatus.attention => _attention(s, l),
      LsStatus.error => _error(s.error, l),
      _ => null,
    };
  }
  return HeroModel(
    status: status,
    detail: detail,
    uploads: s.uploads,
    downloads: s.downloads,
  );
}

String _gameName(dto.SessionDto session) => session.name ?? session.game;

String? _attention(dto.StatusDto s, AppLocalizations l) {
  final lines = [
    if (s.conflicts > 0) l.homeConflicts(s.conflicts),
    if (s.foreignLeases > 0) l.homeLeases(s.foreignLeases),
  ];
  return lines.isEmpty ? null : lines.join('\n');
}

String? _error(dto.ErrorKindDto? kind, AppLocalizations l) => switch (kind) {
  dto.ErrorKindDto.auth => l.homeErrorAuth,
  dto.ErrorKindDto.quota => l.homeErrorQuota,
  dto.ErrorKindDto.permission => l.homeErrorPermission,
  dto.ErrorKindDto.config => l.homeErrorConfig,
  // Handled before the hero, by Home (set-up state).
  dto.ErrorKindDto.notSetUp || null => null,
};

/// "just now", "5 min ago", "2 hours ago", "3 days ago". [ms] is a Unix time
/// in milliseconds; a time in the future reads as "just now".
String relative(AppLocalizations l, DateTime now, int ms) {
  final diff = now.millisecondsSinceEpoch - ms;
  const minute = 60 * 1000;
  const hour = 60 * minute;
  const day = 24 * hour;
  if (diff < minute) return l.timeJustNow;
  if (diff < hour) return l.timeMinutesAgo(diff ~/ minute);
  if (diff < day) return l.timeHoursAgo(diff ~/ hour);
  return l.timeDaysAgo(diff ~/ day);
}

/// The display name of a game: its name, or the key when it has none.
String gameTitle(String? name, String key) => name ?? key;
