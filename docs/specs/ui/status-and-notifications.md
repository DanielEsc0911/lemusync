# Status and notifications

> **Status:** Draft · **Related:** [rules](../../rules.md) (P4, P6, F1), [architecture §Engine API](../architecture.md#engine-api), [sync-model](../sync-model.md#play-sessions), [navigation](navigation.md#deep-links), [platforms/desktop](../platforms/desktop.md), [platforms/android](../platforms/android.md), [platforms/ios](../platforms/ios.md), [i18n](../i18n.md), [brand](brand.md) · **Code:** `crates/api/src/status.rs` (status model), `crates/daemon/src/{view,notices,platform,run}` (tray, notifications, the daemon's mapping from the status model), `app/android`, `app/ios` (planned)

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
- Icon states: synced · playing · syncing (low-rate frame animation, ≤ 4 fps, to save CPU) · paused · offline · attention · error. **Playing** shows while a play session is active and the status is otherwise `synced`: it is a tray-only refinement of `synced`, not a status state. The tray follows the status priority above; playing replaces only `synced`, and only while a session is active, so a problem is never hidden and a background upload during a session stays visible (playing returns when it ends). The tooltip already says "Playing …". Monochrome template image on macOS, light/dark variants on Windows, brand ARGB32 pixmaps on Linux ([brand](brand.md), D42).
- Tooltip = status text; during a play session the session line ("Playing BotW · Daniel") comes first.
- Menu: status line · Sync now · Pause (1 h / until resume) · Recent games ▸ Play · Open LemuSync · Quit (warns if a transfer is active).
- **Recent games** lists every synced stream of this device's repo, newest first by the time of the last version this device synced, uploaded or restored (the stream base's embedded time, `StreamStatus.last_version_ms`; a session that changes no save doesn't reorder; a restored version ranks by when it was made, possibly on another device; streams with none go last, then by name and player), at most 8. The rule (which streams, their order, launchable or not, the cap) is pure code in `lemusync-engine` (`crates/engine/src/library.rs`: `library`, `recent`), shared with the app; the daemon only builds the labels and ids. The label is the game's name on this device, else the stream's `name`, else its key, plus ` · player` when the repo has several players. A game this device has no file to launch (the catalog has no path: for example a Cemu `.wua`/`.wux` title, which the profile doesn't open yet) stays in the list, disabled, with the suffix "(no game file)" ("(sin archivo del juego)", `tray-recent-no-file`), so the user sees why it can't be played. To enable it, give the game a file: `lemusync config set emulators.<id>.games.<key> <absolute path>` or `[emulators.<id>] games."<key>" = "<path>"` in `config.toml` ([config-and-cli](../config-and-cli.md#config-file)). The list is rebuilt at start, after each sync and when a launcher-mode session ends. A click on a disabled or stale item does nothing.
- Linux GNOME shows tray icons only with the AppIndicator extension (Ubuntu ships it; Fedora Workstation doesn't). Without it: notifications + app only, and `doctor` explains. KDE and XFCE show them natively (measured: [desktop §Linux results](../platforms/desktop.md#linux-results-sp2-xfce)).
- Linux implementation: `ksni` StatusNotifierItem with the brand pixmaps (`TrayIcon::pixmaps`: 24, 32 and 48 px ARGB32 per state, generated by `cargo xtask brand` and embedded with `include_bytes!`, so nothing is decoded; `icon_name` is empty, which the StatusNotifierItem spec defines as "use `IconPixmap`"). The 15 616 B of pixmaps per state (seven states) (4·(24²+32²+48²)) are copied into a `Vec<ksni::Icon>` once per icon change and cached in the tray item; `ksni` 0.3.7 reads `icon_pixmap` on every `handle.update`, so the daemon skips the update when the view is unchanged and a changed tooltip or menu with the same icon clones the cached vector (15 616 B) instead of rebuilding it; `scanning` shows the syncing icon. The syncing icon is a single static pixmap on Linux for now: zero wakeups while idle, inside the ≤ 4 fps ceiling. Without a `org.kde.StatusNotifierWatcher` the tray waits and registers when one appears; without a session bus it stays off. Notifications: `notify-rust` with the app icon name `io.github.danielesc0911.lemusync`, which resolves only when the icon is installed into the icon theme (the rpm and deb put it in hicolor 256); dev, `cargo run` and AppImage runs show the host's generic icon. Urgency and title carry the kind of notice (one brand icon, no `image` crate for per-kind raw images). Urgency critical for attention, normal for errors, low for info; a click comes back as the action id (`review`, `wait`, `open`). The view and notice texts are pure functions (`view::tray_view`, `notices::notice`) over local input types that the daemon loop fills from the status model. Code: `crates/daemon/src/view.rs`, `crates/daemon/src/notices.rs`, `crates/daemon/src/platform/` (`icons.rs` holds the pixmaps).

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
