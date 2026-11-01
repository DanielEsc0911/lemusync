# Status and notifications

> **Status:** Draft · **Related:** [rules](../../rules.md) (P4, P6, F1), [architecture §Engine API](../architecture.md#engine-api), [sync-model](../sync-model.md#play-sessions), [navigation](navigation.md#deep-links), [platforms/desktop](../platforms/desktop.md), [platforms/android](../platforms/android.md), [platforms/ios](../platforms/ios.md), [i18n](../i18n.md) · **Code:** `crates/api/src/status.rs` (status model), `crates/daemon/src/{view,notices,platform,run}` (tray, notifications, the daemon's mapping from the status model), `app/android`, `app/ios` (planned)

## Purpose
Every surface (tray, notifications, Live Updates, Live Activities, the app) shows the same truth from one status model, using each platform's best surface (P4).

## Status model
Emitted by the engine as `StatusChanged` / `SessionChanged` events.

| State | Data | Example text (en / es) |
|---|---|---|
| `synced` | last sync time | "All saves synced" / "Todas las partidas están sincronizadas" |
| `scanning` | — | "Checking for changes" / "Buscando cambios" |
| `syncing` | uploads n, downloads m, bytes | "Uploading 3 changes" / "Subiendo 3 cambios" |
| `paused` | until | "Sync paused" / "Sincronización en pausa" |
| `offline` | queued n | "Offline · 2 changes waiting" / "Sin conexión · 2 cambios en espera" |
| `attention` | conflicts, foreign leases | "1 conflict needs you" / "1 conflicto necesita tu decisión" |
| `error` | kind (auth, quota, permission, config) | "Storage sign-in expired" / "La sesión del almacenamiento expiró" |
| session overlay | game, player, device, started | "Playing BotW · Daniel" / "Jugando BotW · Daniel" |

Priority when several apply: `error` > `attention` > `syncing` > `offline` > `paused` > `scanning` > `synced`. A manual sync (tray, CLI) or one started by new credentials clears the shown `error` and publishes `syncing` before its first storage request, so the tray changes at once; if the cause still holds the sync sets the `error` again.

`Status::from_facts` applies this priority to the engine's raw facts (`StatusFacts`); the counts travel with every state.

## Notification policy
- Notify only when something needs the player or changes their plan: conflicts, foreign leases at play time, errors, a takeover of their save. Optional "synced" confirmations are off by default.
- Coalesce bursts (one notification per stream per event type per 5 min). Never spam.
- Desktop daemon (`crates/daemon/src/run.rs`, `notices.rs`): `conflict_detected` (once per new conflict), a live `lease_warning`, `remote_newer` ("Newer save of BotW from Deck" · "Close Cemu and LemuSync updates it") and a change into the auth or quota `error` each become a notice; `notices::Coalescer` applies the 5-minute rule. Other events only update the tray.
- Every notification deep-links to the relevant screen ([navigation](navigation.md#deep-links)). Actions are offered where the platform supports them ("Review", "Wait", "Open").
- Channels/categories: **Sync progress** (silent), **Needs attention** (high), **Errors** (default).

## Desktop tray and menu bar
- Icon states: synced · syncing (low-rate frame animation, ≤ 4 fps, to save CPU) · paused · offline · attention · error. Monochrome template image on macOS, light/dark variants on Windows, symbolic icons on Linux.
- Tooltip = status text; during a play session the session line ("Playing BotW · Daniel") comes first.
- Menu: status line · Sync now · Pause (1 h / until resume) · Recent games ▸ Play · Open LemuSync · Quit (warns if a transfer is active).
- Linux GNOME shows tray icons only with the AppIndicator extension (Ubuntu ships it; Fedora Workstation doesn't). Without it: notifications + app only, and `doctor` explains. KDE and XFCE show them natively (measured: [desktop §Linux results](../platforms/desktop.md#linux-results-sp2-xfce)).
- Linux implementation: `ksni` StatusNotifierItem with freedesktop symbolic icons (`emblem-ok`, `emblem-synchronizing`, `media-playback-pause`, `network-offline`, `dialog-warning`, `dialog-error`, each `-symbolic`); `scanning` shows the syncing icon. The syncing icon is static for now: zero wakeups while idle, inside the ≤ 4 fps ceiling. Without a `org.kde.StatusNotifierWatcher` the tray waits and registers when one appears; without a session bus it stays off. Notifications: `notify-rust`, urgency critical for attention, normal for errors, low for info; a click comes back as the action id (`review`, `wait`, `open`). The view and notice texts are pure functions (`view::tray_view`, `notices::notice`) over local input types that the daemon loop fills from the status model. Code: `crates/daemon/src/view.rs`, `crates/daemon/src/notices.rs`, `crates/daemon/src/platform/`.

## Android
- **Ongoing progress notification** during transfers and play sessions (`NotificationCompat`, progress, Pause/Open actions).
- **Android 16+ Live Updates:** `Notification.ProgressStyle` + request promoted ongoing → status-bar chip. Samsung One UI 8 shows Live Updates in the **Now Bar**. Xiaomi HyperOS 3 shows them in **Super Island** on global builds. One API covers all three. Pre-16 devices get the ongoing notification. Samsung's One UI 7 Live Notifications and Xiaomi's China-only focus notifications need vendor allow-listing → out of scope unless granted.
- **Quick Settings tile:** status + tap to sync now.
- **App shortcuts** (long-press icon): recent games → Play.
- Later: home-screen widget.

## iOS
- **Live Activity + Dynamic Island** during play sessions and active syncs. Compact: lemur glyph + arrows/progress. Expanded: game, player, "Uploading 3 changes". Lock Screen and StandBy presentations. Updated locally by the app while it runs (including background tasks). Remote push updates aren't in v1 ([D13](../../decisions.md#d13-no-server-telemetry-or-accounts-in-v1-push-only-optional-and-later-2026-10-07)).
- **Local notifications** for conflicts and leases detected during app runs or background refresh.
- **App Intents:** Sync Now, Play Game, Pull Saves → Siri, Spotlight, Shortcuts. Recommended automation: "When ⟨emulator⟩ is closed → LemuSync: Sync Now".
- Later: widgets, Control Center control.

## Lock warnings without a server
Devices learn about foreign leases when they check: desktop daemons continuously, Android on app open / scheduled work / play start, iOS on app open / background refresh / intents. Instant push to idle phones needs a push service. That's an optional later phase (UnifiedPush on Android, optional relay), per [D13](../../decisions.md#d13-no-server-telemetry-or-accounts-in-v1-push-only-optional-and-later-2026-10-07).

## Verify
- **V-STAT-1** Live Update eligibility for sync/session notifications and the exact promotion requirements (SP4).
- **V-STAT-2** Now Bar (One UI 8) and Super Island (HyperOS 3 global) render our Live Update (SP4).
- **V-STAT-3** Live Activity update limits and duration for long play sessions, and updates from background tasks (SP5).
- **V-STAT-4** Tray icon + actionable notifications from the daemon on Windows, macOS, KDE and GNOME (SP2). XFCE is done, action click included ([desktop §Linux results](../platforms/desktop.md#linux-results-sp2-xfce)). The others were deferred by the maintainer on 2026-10-08.
