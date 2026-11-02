# App shell

> **Status:** Decided (design only, nothing built) · **Related:** [rules](../../rules.md) (F1, F2, F3, B2, B3, P5, P6), [architecture §IPC](../architecture.md#ipc), [navigation](navigation.md), [status-and-notifications](status-and-notifications.md), [decisions](../../decisions.md) (D35, D40, D43, D44) · **Code:** `crates/engine/src/library.rs` (built, 3b Task 2: the game-library rule shared with the tray); `crates/bridge/src/link/` (built, 3b Task 3: calls, event watcher, backoff, DTOs; plain Rust, not yet exposed to Dart); planned `crates/bridge/src/api/daemon.rs`, `app/lib/app/`, `app/lib/screens/` (nothing is built yet; Phase 3b, [plan](../../progress/plans/phase-3b-shell.md))

## Purpose
How the desktop app reaches the daemon, what state it holds and which destinations it shows. Everything below is planned for Phase 3b and is marked as built only when the task that builds it lands.

## Connection
The app is a client of `lemusyncd` ([D43](../../decisions.md#d43-the-desktop-app-is-a-client-of-the-daemon-over-the-same-ipc-as-the-cli-2026-11-01)). It never runs an engine on desktop and never starts or stops the daemon (that is `lemusync service start`, P5). "Daemon not running" is a value, never an exception.

| State | Meaning | Shown |
|---|---|---|
| `connecting` | First attempt, no result yet | "Connecting…", unless a cached status exists |
| `online` | Handshake done, daemon version known | The data |
| `offline: not_running` | Nothing listens, or there is no runtime dir | "LemuSync isn't running", Retry, the line `lemusync service start` |
| `offline: unsupported` | This OS has no IPC transport yet (Windows today, `IpcError::Unsupported`) | "Not available on this system yet", no Retry |
| `offline: refused` | The daemon rejected the handshake (another `api_version`) | "This app and the service don't match", update LemuSync, no Retry |
| `offline: unsafe` | `IpcError::UnsafePath`: the socket or its folder is a link or belongs to another user | A security message with its own text, no Retry; never shown as "not running" |
| `failed` | The connection works but a call got `Response::Error` or timed out | "The service returned an error" with the code, Retry, the line `lemusync logs` |

With a cached status or games and a connection that is not `online`, the last data stays under a banner "Showing the last known state". Reconnect backoff is 1 s, 2 s, 4 s … capped at 30 s, reset on success. The planned texts are in the plan's Task 8 table (English and Spanish, P6).

## Data flow
- Built in `crates/bridge/src/link/` (Task 3). Calls (`Status`, `Games`, `Sync`) use one fresh short connection each, every one with a timeout. One long-lived connection holds the event subscription.
- On every (re)connect the bridge subscribes first, then fetches `Status` (emitted as a `Snapshot` without games), then `Games` on its own connection (emitted as a separate `Games` update). An event between the snapshot and the subscription cannot be lost, and a `Games` call waiting behind a sync never stalls events.
- On subscribe the daemon sends a `StatusChanged` first but no `SessionChanged`, so the snapshot's session seeds the play overlay.
- Events used: `StatusChanged`, `SessionChanged`, `TransferProgress`. `ConflictDetected`, `LeaseWarning`, `Error`, `RemoteNewer` and connection retries are ignored in 3b; their facts reach Home through the `Status` counts. `lemusync_ipc::Client::subscribe` skips an event it cannot decode (an unknown name or a payload that no longer fits), so a newer daemon never breaks an older app; a body that is not JSON still ends the stream. A failed first `Games` fetch on a connection is retried once. Events buffered between subscribing and the `Status` snapshot may be older than it and briefly roll Home back; the next event or snapshot corrects it (accepted).
- The game list is fetched on connect, on app resume, when `last_sync_ms` changes and on `SessionChanged(None)`. There is no polling while connected.
- The bridge hands Dart bridge-owned DTOs (no wire types, no serde, no secrets). Paths stay in Rust: Dart gets `launchable: bool` only (B2).

## App state
Planned. One `AppController` (a `ChangeNotifier`, no new package) holds an immutable `AppState`: connection, status, session, games, recent, live transfers, device, players. It is built from the event stream and tested with a fake link.

- `lsStatus` maps the seven daemon states one to one onto the design system's `LsStatus`. `playing` is true when a session is active and the status is synced; any other status keeps its own headline and shows the session as a secondary line.
- Per-game card status is derived: `syncing` when progress was seen for that game since the last non-syncing status, else the global status when it is offline, paused or error, else synced. `attention` is not attributed to a card in 3b (the wire has no per-game attention). Decided by the maintainer on 2026-11-01.
- Missed events are invisible to clients, so the refresh triggers above are the recovery.

## Destinations as built (3b)
Planned. Home and Games are real; Activity, Devices and Settings are not shown at all until Phase 3d builds them: no placeholder screen, no "coming soon" (F2). The visible destinations keep [navigation §Destinations](navigation.md#destinations) order on every platform (F3), and 3d inserts the new ones where that table puts them. Decided by the maintainer on 2026-11-01.

- **Home:** status hero, play overlay, last-sync line, attention banners with read-only CLI lines, recent games.
- **Games:** read-only library grid of the catalog games plus synced games the catalog does not know (those without a game file, marked). There is no Play in 3b: Play needs the lease and conflict prompts, which are Phase 3c flows. The one write is **Sync now**.
- Layout by width: bottom bar, rail or sidebar at 600 dp and 1200 dp (size tokens, not literals).

## Desktop window
Planned for Linux first. Ctrl/Cmd+1 and 2 select the visible destinations (one digit per destination, 3d extends them), Ctrl/Cmd+R runs Sync now. A second launch raises the first window (single instance). Closing the window quits the app only; the daemon, the tray and the sync keep running. The tray "Open LemuSync" item and the Windows and macOS window code are Phase 3d.

## Verify
Owner is the plan task that answers it; full text in the [plan](../../progress/plans/phase-3b-shell.md#verify).
- `V-SHELL-1` (Task 3, answered from FRB's source, Dart check in Task 4): FRB's `async fn` runs on its own tokio runtime, but a sync fn with a `StreamSink` runs on a thread pool with none, so the watcher owns a 1-worker runtime (`link::runtime()`).
- `V-SHELL-2` (3d, SP11): the Flatpak permission for the daemon socket.
- `V-SHELL-3` (Task 11): lag is invisible to clients; mitigated by refresh triggers.
- `V-SHELL-4` (Task 9): derived per-game syncing status has no end marker.
- `V-SHELL-5` (Task 11): how long `Games` waits behind a sync.
- `V-SHELL-6` (Task 9): whether a profile display name exists; the emulator id is used.
- `V-SHELL-7` (Task 10, Task 11): the single-instance raise on KDE Wayland and X11.
- `V-SHELL-8` (Task 7, Task 11): shortcuts do not clash with the desktop environment.
- `V-SHELL-9` (Task 11): 60 fps floor on Home and Games in profile mode.
- `V-SHELL-10` (Task 4): FRB resolves DTOs re-exported through `api::daemon`.
- `V-SHELL-11` (Task 4, Task 11): cancelling the Dart stream stops the Rust task promptly.
- `V-SHELL-12` (Task 3, Task 11): the call timeouts are starting values.
- `V-SHELL-13` (Task 8): what `lemusync doctor` prints for each error kind.
- `V-SHELL-14` (Task 3): growth of `liblemusync_bridge.so` from the engine dependency.
