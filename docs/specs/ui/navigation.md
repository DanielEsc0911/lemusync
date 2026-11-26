# Navigation

> **Status:** Draft · **Related:** [rules](../../rules.md) (F3, P5, B2), [design-system](design-system.md), [status-and-notifications](status-and-notifications.md), [players-and-accounts](../players-and-accounts.md), [sync-model](../sync-model.md), [security](../security.md) · **Code:** `app/lib/app/destination.dart`, `app/lib/app/app_shell.dart`, `app/lib/app/shortcuts.dart`

## Purpose
The single source of truth for where everything is and how every flow goes, the same on every platform (F3).

## Destinations
Same order, names and icons everywhere:

| # | Destination | Holds |
|---|---|---|
| 1 | **Home** | Status hero, active sessions, needs-attention list (conflicts, leases, errors), recent activity |
| 2 | **Games** | Library by emulator/system, from the engine's game catalog (`Engine::games`, the same list as `lemusync games`: name, key, path, where it was found); game detail → Play, history, restore, lease, save states |
| 3 | **Activity** | Timeline of syncs, conflicts, restores, errors; filters |
| 4 | **Devices** | Devices in the repo, last seen, sessions; this device's settings |
| 5 | **Settings** | Players · Storage · Emulators · Notifications · Language · Advanced · About |

Destinations appear only once they work end to end (F2); until Phase 3d the desktop app shows Home and Games ([D44](../../decisions.md#d44-destinations-appear-only-once-they-work-end-to-end-2026-11-01)). As of 3b (planned): Home and Games are built. The order of the five stays this table's; a destination is shown from the phase that builds it. This is a staging order, the same on every platform at any build (F3 holds). See [app-shell](app-shell.md#destinations-as-built-3b).

## Adaptive containers
| Size class (width) | Container | Detail views |
|---|---|---|
| Compact (< 600 dp) | Bottom navigation bar | Pushed full screen |
| Medium (600–1199 dp) | Navigation rail | Pushed, or a side pane in landscape |
| Expanded (≥ 1200 dp) | Sidebar with labels | List-detail side pane |

The flow steps are identical in every container. Only the presentation changes.

## Flows
### First run (create)
Welcome (language auto-detected, changeable) → Create or Join → **Storage** (pick provider → details → probe) → **Passphrase** → **Recovery key** (show, confirm saved) → **You** (player name) → **This device** (name, read-only) → **Find emulators** (scan → grant folders) → **Games found** (read-only list) → Initial sync → Home.

The device name is the one `init`/`join` gives; there is no game picking: every save a profile finds is synced ([D49](../../decisions.md#d49-first-run-has-no-game-picking-and-no-device-naming-for-now-2026-11-26)). The desktop app's storage step offers S3-compatible and a local folder; Google Drive appears once its login works in the app ([D47](../../decisions.md#d47-the-apps-first-run-offers-s3-compatible-storage-and-a-local-folder-google-drive-waits-for-its-login-in-the-app-2026-11-26)). Planned in [phase-3c](../../progress/plans/phase-3c-flows.md).

### Join existing
Welcome → Join → Storage → Passphrase *or* recovery key → This device (read-only) → **Map accounts** (proposed by name, confirm) → Find emulators → Initial sync (pull) → Home.

### Play (launcher mode)
Games → game → **Play** (as default player; "Play as…" to choose) → pulling (live status) → lease check (if held elsewhere: Wait / Play anyway / Take over if stale) → emulator runs (session status) → on return: uploading → done.

### Resolve conflict
Notification or Home attention item → **Compare** (heads side by side) → Choose → confirmation → done. The other versions stay in history.

### Lease warning
Notification → game detail with lease info → Wait / Play anyway / Take over (stale only, with confirmation).

### Restore a version
Games → game → History → version → **Restore** (shows "your current save will be backed up") → confirm → done.

### Add emulator or custom path (power)
Settings → Emulators → emulator → Advanced → path override / extra roots / custom profile → validate (shows resolved paths and grants) → save.

### Storage change
Settings → Storage → edit or test → probe → save. Changing provider means migrating the repo (guided copy, verified, then switch).

## Deep links
`lemusync://` opens Home, a game, a conflict, or a lease. Links **only navigate**. Every action needs an explicit confirmation ([security](../security.md#profiles-deep-links-launching)). Notifications use these links.

## Desktop extras
- Keyboard shortcuts (⌘/Ctrl+1–5 for destinations, ⌘/Ctrl+, for Settings, ⌘/Ctrl+R to sync now).
- As built in 3b (Linux): Ctrl/Cmd+1 and 2 (the visible destinations), Ctrl/Cmd+R (sync now, only when the daemon is online, otherwise nothing happens), one window per user (a second launch raises the first) and a 360 x 480 dp minimum window. Ctrl/Cmd+, waits for Settings (3d).
- "Open LemuSync" in the tray is deferred to 3d: it needs the app's installed path or `.desktop` id and a spawn, which belong with packaging.
- Closing the window quits the app only. The tray belongs to the daemon ([D35](../../decisions.md)), so the sync and the tray keep running; there is no close-to-tray in the app.
- The tray menu mirrors Home's quick actions ([status-and-notifications](status-and-notifications.md#desktop-tray-and-menu-bar)).
