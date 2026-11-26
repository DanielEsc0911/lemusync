# App shell

> **Status:** Decided (design only, nothing built) · **Related:** [rules](../../rules.md) (F1, F2, F3, B2, B3, P5, P6), [architecture §IPC](../architecture.md#ipc), [navigation](navigation.md), [status-and-notifications](status-and-notifications.md), [decisions](../../decisions.md) (D35, D40, D43, D44) · **Code:** `crates/engine/src/library.rs` (built, 3b Task 2: the game-library rule shared with the tray); `crates/bridge/src/link/` (built, 3b Task 3: calls, event watcher, backoff, DTOs; plain Rust); `crates/bridge/src/api/daemon.rs` and `app/lib/app/daemon_link.dart` (built, 3b Task 4: the FRB surface and the Dart `DaemonLink`); planned `app/lib/app/` (controller), `app/lib/screens/` (nothing is built yet; Phase 3b, [plan](../../progress/plans/phase-3b-shell.md))

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

`failed` and `offline: refused` carry the daemon's `message`, which can contain the user's own local paths (`NotGranted`, `Locked`, `Data`, `Io`). Accepted (lead ruling, B2): they are the same user's paths on their own machine, shown as the CLI shows them, not secrets. The screen leads with the localized text chosen by `code` and shows the raw `message` only as secondary detail; `message` is never logged anywhere that leaves the device.

With a cached status or games and a connection that is not `online`, the last data stays under a banner "Showing the last known state". Reconnect backoff is 1 s, 2 s, 4 s … capped at 30 s, reset on success. The planned texts are in the plan's Task 8 table (English and Spanish, P6).

## Data flow
- Built in `crates/bridge/src/link/` (Task 3). Calls (`Status`, `Games`, `Sync`) use one fresh short connection each, every one with a timeout. One long-lived connection holds the event subscription.
- On every (re)connect the bridge subscribes first, then fetches `Status` (emitted as a `Snapshot` without games), then `Games` on its own connection (emitted as a separate `Games` update). An event between the snapshot and the subscription cannot be lost, and a `Games` call waiting behind a sync never stalls events.
- On subscribe the daemon sends a `StatusChanged` first but no `SessionChanged`, so the snapshot's session seeds the play overlay.
- Events used: `StatusChanged`, `SessionChanged`, `TransferProgress`. `ConflictDetected`, `LeaseWarning`, `Error`, `RemoteNewer` and connection retries are ignored in 3b; their facts reach Home through the `Status` counts. `lemusync_ipc::Client::subscribe` skips an event it cannot decode (an unknown name or a payload that no longer fits), so a newer daemon never breaks an older app; a body that is not JSON still ends the stream. A failed first `Games` fetch on a connection is retried once. Events buffered between subscribing and the `Status` snapshot may be older than it and briefly roll Home back; the next event or snapshot corrects it (accepted).
- The game list is fetched on connect, on app resume, when `last_sync_ms` changes and on `SessionChanged(None)`. There is no polling while connected.
- The bridge hands Dart bridge-owned DTOs (no wire types, no serde, no secrets). Paths stay in Rust: Dart gets `launchable: bool` only (B2).
- **Game-flow calls (3c, built in Task 2, not yet reachable from Dart):** `crates/bridge/src/link/flows.rs` adds `history`, `conflicts`, `leases`, `restore`, `resolve` and `lease_release`, one short connection each, DTOs in `link/flow_model.rs` (`HistoryDto`, `VersionDto`, `ConflictDto`, `HeadDto`, `LeaseDto`, `MovedDto`; the lease holder's device id and the version parents stay in Rust). A failure is the same `ConnectionState` as 3b (`Failed { code }` for a daemon error or a timeout). Timeouts: reads 30 s (they queue behind a sync, like `Games`), restore and resolve 120 s (V-FLOW-1); a write that times out still finishes in the daemon. The reports' warnings are not passed on yet (their codes have no string form outside serde).
- **Play session (3c, built in Task 3, not yet reachable from Dart):** `crates/bridge/src/link/play.rs` sends `play` on its own connection and keeps it open: `play_event` frames become `PlayUpdate::Step`, a `prompt` becomes `Question` and waits for the app's answer, which goes back as `answer` on the same connection; the report ends it as `Done`, a daemon error or a hang-up as `Failed`. Only the first answer after a question is sent; answers given while no question waits are dropped. When the app stops listening (the stream or the answerer is dropped) the connection closes at once, even while the emulator runs: the daemon then refuses any later question and still uploads and releases the lease (architecture §Process model, V-FLOW-3). The app sends no extra emulator arguments. Only the connect and handshake have a timeout (10 s); a game runs for hours.

## App state
Built (3b Task 5). One `AppController` (a `ChangeNotifier`, no new package; `app/lib/app/app_controller.dart`) holds an immutable `AppState` (`app_state.dart`): connection, status, session, games, recent, live transfers, device, players, `loaded`. It is built from the link's event stream and tested with `FakeDaemonLink` (`test/support/fake_link.dart`).

- A `Snapshot` replaces connection, status, session, device and players and clears live transfers; it carries no games, so the last list stays until a `Games` arrives. `Status` replaces the status and clears live transfers unless the state is `syncing`. `Progress` adds the `emulator/game` key. `Connection` only sets the connection: when it is not online, the last status and games stay (stale beats blank).
- The controller notifies only when the state changed (value equality), so a repeated `StatusChanged` does not rebuild the tree. A stream error becomes `NotRunning`; nothing throws to the UI. `dispose` cancels the subscription and is safe to call twice.
- `refresh()` (Retry and app resume) fetches the snapshot and the games; a failed games fetch changes nothing. `AppScope` (`app_scope.dart`) exposes the controller through an `InheritedNotifier` and calls `refresh()` on the `resumed` lifecycle state only.

- `lsStatus` maps the seven daemon states one to one onto the design system's `LsStatus`. `playing` is true when a session is active and the status is synced; any other status keeps its own headline and shows the session as a secondary line.
- Per-game card status is derived: `syncing` when progress was seen for that game since the last non-syncing status, else the global status when it is offline, paused or error, else synced. `attention` is not attributed to a card in 3b (the wire has no per-game attention). Decided by the maintainer on 2026-11-01.
- Missed events are invisible to clients, so the refresh triggers above are the recovery.

## Destinations as built (3b)
Shell built (Task 7): `Destination` enum (`home`, `games`), `AppShell` (bar under 600 dp, rail under 1200 dp, sidebar above, from size tokens) and Ctrl/Cmd+digit shortcuts; `LemuSyncApp(link:)` owns the `AppController` and the current destination. The Home body is built (Task 8, below); the Games body is a stub until Task 9. Home and Games are the destinations; Activity, Devices and Settings are not shown at all until Phase 3d builds them: no placeholder screen, no "coming soon" (F2). The visible destinations keep [navigation §Destinations](navigation.md#destinations) order on every platform (F3), and 3d inserts the new ones where that table puts them. Decided by the maintainer on 2026-11-01.

- **Home (built, Task 8):** `HomeScreen` inside `ConnectionGate` (`app/lib/screens/`). `ConnectionGate` is the one switch from `AppState.connection` to full-area states: connecting; offline daemon (`lemusync service start`, Retry); unsupported; refused and unsafe (no Retry, a security or version signal); failed (the localized text for the code first, the daemon's raw `message` only as small secondary detail, `lemusync logs`, Retry). With a cached status and a connection that is connecting or not running, the child stays under a "Showing the last known state" banner. Online, `HomeScreen` shows `StatusHero` fed by `heroFor` (`status_mapper.dart`, pure), the last-sync caption (synced only), the playing line (as the hero's detail when synced; a separate line when another status leads), one attention banner per fact (conflicts, foreign leases) each followed by its read-only CLI line, the error text with `lemusync doctor`, the not-set-up state (`lemusync init`, `lemusync join`) and up to 8 recent games in the engine's order (read-only rows, "no game file" marker). `CommandLine` shows CLI text as selectable mono text on a sunken panel: it is not a button and is never translated. **Sync now** is offered only online and not in error; a tap disables the button and relabels it "Starting sync…" until the call answers. The paused state shows its label only; "until HH:mm" is a 3d item (no string yet).
- **Games:** read-only library grid of the catalog games plus synced games the catalog does not know (those without a game file, marked). There is no Play in 3b: Play needs the lease and conflict prompts, which are Phase 3c flows. The one write is **Sync now**. Built (Task 9): `GamesScreen` (`app/lib/screens/games/`) inside `ConnectionGate`: a lazy `SliverGrid` of `GameCard`s capped at `LsSize.cardMaxWidth` (columns follow the width), no `onPlay` and no `onTap` (cards are not focusable until 3c). Each card shows the name or key, the emulator with the player count, the status from `AppState.cardStatus` (a live transfer of that game wins; otherwise the daemon's offline, paused or error; attention and scanning read as synced because the wire has no per-game attention, V-SHELL-4), the newest stream time, and a folder-icon "no game file" line when the game cannot be launched here. An empty library shows the CLI line `lemusync games`.
- Layout by width: bottom bar, rail or sidebar at 600 dp and 1200 dp (size tokens, not literals).

## Desktop window
As built on Linux (3b). Ctrl/Cmd+1 and 2 select the visible destinations (one digit per destination, 3d extends them), Ctrl/Cmd+R runs Sync now. A second launch raises the first window (single instance: the runner uses `G_APPLICATION_DEFAULT_FLAGS` and presents the active window; under Wayland focus rules it may only flash the taskbar entry). The minimum window is 360 x 480 dp (`gtk_widget_set_size_request`). Ctrl/Cmd+R does nothing unless the daemon is online. Closing the window quits the app only; the daemon, the tray and the sync keep running. The tray "Open LemuSync" item and the Windows and macOS window code are Phase 3d.

## Verify
Owner is the plan task that answers it; full text in the [plan](../../progress/plans/phase-3b-shell.md#verify).
- `V-SHELL-1` (Task 3, answered from FRB's source and confirmed from Dart in Task 4, 2026-11-01: `Handle::try_current()` is ok inside an `async fn`, not inside a sync fn with a `StreamSink`): FRB's `async fn` runs on its own tokio runtime, but a sync fn with a `StreamSink` runs on a thread pool with none, so the watcher owns a 1-worker runtime (`link::runtime()`).
- `V-SHELL-2` (3d, SP11): the Flatpak permission for the daemon socket.
- `V-SHELL-3` (Task 11): lag is invisible to clients; mitigated by refresh triggers.
- `V-SHELL-4` (Task 9): derived per-game syncing status has no end marker.
- `V-SHELL-5` (Task 11): how long `Games` waits behind a sync.
- `V-SHELL-6` (Task 9): whether a profile display name exists; the emulator id is used.
- `V-SHELL-7` (Task 10, Task 11): the single-instance raise on KDE Wayland and X11.
- `V-SHELL-8` (Task 7, Task 11): shortcuts do not clash with the desktop environment.
- `V-SHELL-9` (Task 11): 60 fps floor on Home and Games in profile mode.
- `V-SHELL-10` (Task 4, confirmed): FRB resolves DTOs re-exported through `api::daemon`; they are generated under `app/lib/bridge/link/`.
- `V-SHELL-11` (Task 4 confirmed with a Dart run, Task 11): cancelling the Dart stream calls `WatchHandle.stop()` and the Rust task ends at once.
- `V-SHELL-12` (Task 3, Task 11): the call timeouts are starting values.
- `V-SHELL-13` (Task 8): what `lemusync doctor` prints for each error kind.
- `V-SHELL-14` (Task 3, Task 4): `liblemusync_bridge.so` grew from 2.11 MB to 4.60 MB with the engine reachable (release, measured Task 4). Breakdown 2026-11-01: `crc_fast`, `core`, `lemusync_api` and `rustls` are the largest symbol groups; `lemusync_engine` is 7 KB.

Task 11 results (tests, 2026-11-25): the rebuild budget holds with the fake link (a repeated identical status rebuilds nothing; 200 progress events for one stream rebuild at most one game card on Games and the hero once on Home), Home's semantic order is banner, hero, attention, recent games, and Home syncing runs no animation under reduce motion. V-SHELL-9 (frame times) and the live items stay for the maintainer's review (plan Task 11 Step 5); the performance overlay was not watched.
