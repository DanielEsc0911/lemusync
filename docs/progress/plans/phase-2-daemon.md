# Phase 2, desktop daemon (Linux first)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Also follow the project skills: `lemusync-feature` (always), `lemusync-security` (Tasks 2, 5, 6, 7, 8, 9), `lemusync-docs` (every task).

**Goal:** `lemusyncd` keeps saves in sync all day on Linux with no terminal open: it watches save folders, syncs after the quiet period, follows emulators in watch mode, shows a tray icon and actionable notifications, survives sleep, crashes and bad networks, and answers the CLI over local IPC. This is the Linux half of [Phase 2](../roadmap.md#phase-2-desktop-daemon); Windows and macOS follow once SP2 runs there ([D36](../../decisions.md#d36-phase-2-builds-the-daemon-on-linux-first-2026-10-28)).

**Architecture:** Pure state machines in `core` (quiet period, scheduler and backoff, watch-mode sessions; time and randomness are inputs). `engine` gets the drivers: a save-folder watcher (`notify`), a process poller, and `Service`, the loop that owns the one `Engine` and runs commands one at a time ([architecture §Process model](../../specs/architecture.md#process-model)). `api` gets the wire types (requests, responses, events, the status model). `ipc` is the transport. `daemon` is thin: service lifecycle, IPC server, tray, notifications and sleep hooks behind small traits with Linux implementations. The CLI is an IPC client whenever the daemon runs ([D35](../../decisions.md#d35-the-cli-is-the-daemons-client-while-it-runs-2026-10-28)).

**Tech Stack:** as Phase 1 (Rust 1.99.0, tokio, OpenDAL 0.59.5, SQLite, clap, Fluent). New dependencies, each resolved to the current release and `cargo deny`-checked in the task that adds it: `notify` (Task 4), `ksni`, `notify-rust`, `zbus` (Task 6; SP2 used 0.3.6, 4.18.2, 5.19.0), `tracing` + `tracing-subscriber` (Task 8). Prefer crates already in `Cargo.lock` (`zbus` is) and no default features.

**Spec:** [architecture](../../specs/architecture.md) §Process model, §Engine API, §IPC, §Engine internals; [sync-model](../../specs/sync-model.md) §Play sessions (Watch mode), §Snapshot rules; [reliability](../../specs/reliability.md) §Always on, §Sleep and resume, §Network, §Budgets; [status-and-notifications](../../specs/ui/status-and-notifications.md); [desktop](../../specs/platforms/desktop.md) §Linux results; [security](../../specs/security.md) §Local IPC; [config-and-cli](../../specs/config-and-cli.md) §Commands. Read only the sections a task names.

## Global constraints

- Everything in the first-sync plan's [Global constraints](phase-1-first-sync.md#global-constraints) still holds (toolchain, clippy pedantic as errors, layers, `core` has no I/O/clock/randomness, I1–I6, secrets, no real saves in tests, P6 en + es, W1, W2).
- `unsafe_code = "forbid"` stays in every crate but `bridge`. Peer credentials, uids and D-Bus come from safe APIs (tokio `UnixStream::peer_cred`, `rustix` or `std` metadata, `zbus`).
- Formats: new wire types are versioned with `API_VERSION`; existing report types gain only `#[serde(default)]` fields. Remote formats stay v1.
- I3 is never weakened by watch mode: no snapshot and no restore while the emulator runs or before the quiet period ends.
- The daemon opens no inbound network port (B2). The IPC socket is same-user only ([security §Local IPC](../../specs/security.md#local-ipc)).
- Idle means idle (B3, [reliability §Budgets](../../specs/reliability.md#budgets)): no busy loops; timers sleep until their deadline; the process poll is the only periodic wake (Task 5 sets its interval).
- Linux code is `#[cfg(target_os = "linux")]`; other targets compile with a no-op or "unsupported" implementation, so CI stays green on Windows and macOS.
- Every new user-facing string goes into `locales/{en,es}/lemusync.ftl`; `cargo xtask i18n-check` passes.
- W5: Conventional Commits; the maintainer is the author; the only trailer is `Co-Authored-By: Claude <noreply@anthropic.com>`. No session links.
- Each task ends with `just check` green, then its commit.

## Waves

Tasks in the same wave touch different files and can run in parallel. Wave 2 starts once wave 1 is merged.

| Wave | Tasks |
|---|---|
| 1 | A: Tasks 1–2 (`api`, `ipc`) · B: Tasks 3–5 (`core` machines, `engine` watcher and poller) · C: Tasks 6–7 (Linux tray, notifications, power, `service`) |
| 2 | Task 8 (`engine::Service`, `lemusyncd`), then Task 9 (CLI over IPC) |
| 3 | Task 10 (daemon budgets); Tasks 11–13 are the maintainer's |

---

### Task 1: `api` wire types and the status model

**Why:** the daemon, the CLI and later the UI speak one versioned `EngineApi` ([architecture §Engine API](../../specs/architecture.md#engine-api)); today `api` holds only report types.

**Files:** `crates/api/src/lib.rs` (keep report types), `crates/api/src/wire.rs` (new: `Hello`, `Request`, `Response`, `Event`), `crates/api/src/status.rs` (new), spec `architecture.md` §Engine API, `status-and-notifications.md` §Status model (Code field).

**Interfaces (produces):**
- `pub struct Hello { pub api_version: u32, pub client: String }` and `HelloReply { api_version, daemon_version: String }`.
- `pub enum Request` (serde `tag = "cmd"`, snake_case): `Status`, `Sync { dry_run: bool }`, `Pause { until_ms: Option<u64> }` (None = until resume), `Resume`, `Conflicts`, `Games`, `History { game: String, emu: Option<String>, player: Option<String> }`, `LeaseStatus`, `Doctor`, `Subscribe` (the connection then receives `Event`s), `Shutdown`. Task 9 adds the rest of the CLI commands.
- `pub enum Response` (`{"ok": <name>, "data": …}`, adjacently tagged): one variant per request carrying the existing report type (`Conflicts` and `LeaseStatus` carry `Vec<ConflictInfo>` / `Vec<LeaseInfo>`; `Paused`, `Resumed`, `Subscribed`, `ShuttingDown` carry nothing), plus `Error(ErrorBody)` reusing `ErrorBody`. `Event` is `{"event": <name>, "data": …}`.
- `pub enum Event`: `StatusChanged(Status)`, `SessionChanged(Option<Session>)`, `ConflictDetected(ConflictInfo)`, `LeaseWarning(LeaseInfo)`, `TransferProgress(TransferProgress)`, `Connection(ConnectionEvent)`, `Error(ErrorBody)`, and (added in Task 8) `RemoteNewer(RemoteNewer { player, emulator, game, name, device })` for the watch-mode "newer save elsewhere" check.
- `pub struct Status { pub state: StatusState, pub last_sync_ms: Option<u64>, pub uploads: u32, pub downloads: u32, pub bytes: u64, pub paused_until_ms: Option<u64>, pub queued: u32, pub conflicts: u32, pub foreign_leases: u32, pub error: Option<ErrorKind> }`, `pub enum StatusState { Synced, Scanning, Syncing, Paused, Offline, Attention, Error }`, `pub enum ErrorKind { Auth, Quota, Permission, Config }`, `pub struct Session { game, name: Option<String>, emulator, player, device, started_ms }`.
- `pub struct StatusFacts { … }` (the raw booleans and counts) and `pub fn Status::from_facts(&StatusFacts) -> Status` applying the spec's priority `error > attention > syncing > offline > paused > scanning > synced`.

- [x] **Step 1: failing tests.** Golden JSON for one `Request`, `Response` and `Event` of each kind (stable keys, like `sync_report_json_is_stable`); round trip serde for every variant; a table test of `from_facts` covering each priority pair (e.g. conflicts + syncing → `Attention`; error + anything → `Error`; paused + offline → `Offline`).
- [x] **Step 2:** implement; tests pass; `cargo clippy -p lemusync-api --all-targets -- -D warnings`.
- [x] **Step 3:** specs (W1): §Engine API lists the request and event names and points to `crates/api/src/{wire,status}.rs`; `just check`; commit `feat(api): add engine wire types and the status model`.

### Task 2: `ipc` over a Unix socket

**Files:** `crates/ipc/Cargo.toml` (deps: `lemusync-api`, `tokio` with `net`, `io-util`, `rt`, `sync`, `time`; `serde_json`; `thiserror`; a safe uid source), `crates/ipc/src/{lib,frame,socket,client,server}.rs`, `crates/ipc/tests/`, specs `architecture.md` §IPC, `security.md` §Local IPC (Code field).

**Interfaces (produces):**
- `frame`: `async fn read_frame<T: DeserializeOwned>(r) -> Result<Option<T>, IpcError>` (None on clean EOF) and `write_frame`. u32 big-endian length + JSON; a length above 1 MiB (`MAX_FRAME = 1 << 20`) is rejected **before** allocating; invalid JSON is an error, never a panic.
- `socket::default_path() -> Result<PathBuf, IpcError>`: `$XDG_RUNTIME_DIR/lemusync/daemon.sock`; no `XDG_RUNTIME_DIR` → `IpcError::NoRuntimeDir` (never fall back to `/tmp`). `LEMUSYNC_TEST_ROOT=<dir>` → `<dir>/run/daemon.sock`.
- `Server::bind(path) -> Result<Server, IpcError>`: creates the dir `0700`, refuses a dir owned by another uid or a symlink, removes a stale socket only when nothing answers on it (a live one → `IpcError::AlreadyRunning`), binds, sets the socket `0600`. `Server::accept() -> Incoming` checks `peer_cred().uid() == own uid` and closes otherwise (logged, no reply); `Incoming::handshake() -> Result<Option<Connection>, IpcError>` then reads `Hello` within 5 s (`IpcError::HandshakeTimeout`), on the connection's own task (Task 8 moved it off the accept loop); a different `api_version` gets an error reply with an upgrade hint and `None`.
- `Connection { async fn recv() -> Option<Request>, async fn send(&Response), async fn event(&Event) }`.
- `Client::connect(path, client_name) -> Result<Client, IpcError>` (does the handshake; `IpcError::NotRunning` when nothing listens), `async fn call(&mut self, Request) -> Result<Response, IpcError>`, `async fn subscribe(self) -> Result<impl Stream<Item = Result<Event, IpcError>>, IpcError>` (fails when the daemon doesn't answer `subscribed`).
- Windows and macOS: macOS uses the same Unix transport (`getpeereid` through tokio's `peer_cred`); Windows returns `IpcError::Unsupported` for now (named pipe with a current-user DACL is Task 11).

- [x] **Step 1: failing tests.** Frame: round trip; an announced length of 1 MiB + 1 is refused without reading the body; truncated body → error; garbage JSON → error. Socket (Linux, tempdir): bind sets dir `0700` and socket `0600`; a second `bind` while the first runs → `AlreadyRunning`; a stale socket file is replaced; a handshake with `api_version + 1` gets the upgrade error; `call(Status)` gets the server's reply; `subscribe` gets two events in order; `connect` with nothing listening → `NotRunning`. Peer check: unit-test the uid comparison function (a real second uid isn't available in CI).
- [x] **Step 2:** implement; pass; clippy.
- [x] **Step 3:** specs (W1), `just check`, commit `feat(ipc): add the same-user Unix socket transport`.

### Task 3: `core` quiet period, scheduler and backoff

**Why:** when to sync is pure logic; keep it testable without clocks ([reliability §Always on](../../specs/reliability.md#always-on-247), §Network).

**Files:** `crates/core/src/schedule.rs` (new), `crates/core/src/lib.rs`, spec `reliability.md` (Code field and a short "Scheduler" paragraph under §Always on).

**Interfaces (produces):**
- `pub struct Quiet { … }`: `fn new(quiet_ms: u64)`, `fn touch(&mut self, stream: StreamId, now_ms: u64)`, `fn due(&self, now_ms: u64) -> Vec<StreamId>` (streams quiet for `quiet_ms` since their last touch), `fn take_due(&mut self, now_ms) -> Vec<StreamId>`, `fn next_deadline(&self) -> Option<u64>`.
- `pub enum Trigger { Change(StreamId), Timer, Resume, NetworkBack, EmulatorExit(String), Manual }`.
- `pub struct Scheduler { … }`: `fn new(cfg: ScheduleConfig)` (`rescan_ms` default 15 min, `backoff_min_ms` 1 s, `backoff_max_ms` 5 min), `fn on_trigger(&mut self, t: Trigger, now_ms)`, `fn on_sync_done(&mut self, outcome: SyncOutcome, now_ms, jitter: f64)` (`Ok`, `Transient`, `Auth`, `Other`; jitter in `[0, 1)` is an input), `fn on_sync_started(&mut self)` (the caller is running the `SyncNow` it got), `fn pause(&mut self, until_ms: Option<u64>)`, `fn resume(&mut self)`, `fn next_action(&self, now_ms) -> Action` where `Action` is `SyncNow`, `WaitUntil(u64)` or `Idle` (paused until resume, or stopped on `Auth`).
- Rules: one sync at a time (a trigger during a sync sets "again after"); `Transient` → exponential backoff with full jitter, doubling from 1 s up to 5 min, reset on `Ok`; `Auth` → stop until `Manual` or `resume` (the daemon notifies); paused → no `SyncNow` except `Manual`; the rescan timer fires every `rescan_ms` after the last finished sync; `Resume` and `NetworkBack` sync now and reset the backoff.

- [x] **Step 1: failing tests** (table and property tests with `proptest` if `core` already uses it, plain loops otherwise): quiet period with interleaved touches; backoff sequence 1, 2, 4 … capped at 300 s with jitter 0.999 and 0; reset after `Ok`; `Auth` stops until `Manual`; pause until a time then automatic resume; a trigger during a sync gives exactly one follow-up sync; `next_action` never returns a deadline in the past.
- [x] **Step 2:** implement; pass; clippy; spec; `just check`; commit `feat(core): add the quiet-period tracker and sync scheduler`.

### Task 4: `engine` save-folder watcher

**Files:** `crates/engine/Cargo.toml` (`notify`, no default features beyond the OS backend), `crates/engine/src/watch.rs` (new), `crates/engine/tests/watch.rs`, spec `architecture.md` §Engine internals (`watch` row: Code path).

**Interfaces (produces):** `pub struct SaveWatcher`: `fn start(folders: Vec<(StreamId, PathBuf)>, tx: tokio::sync::mpsc::Sender<StreamId>) -> Result<SaveWatcher, EngineError>` watches each save folder recursively and sends the stream id on any create/modify/remove/rename below it (deduplicated per 250 ms burst so the channel can't flood); `fn set_folders(&mut self, …)` replaces the set (grants or mappings changed). A folder that doesn't exist yet is watched through its nearest existing granted parent and promoted when it appears. Never follows symlinks out of the grant (B2): events whose canonical path leaves the folder are dropped. Also `pub async fn Engine::watch_targets(&mut self) -> Result<Vec<(StreamId, PathBuf)>, EngineError>` (the mapped save folders this device syncs, from the same data `sync` uses; no saves are read, but stream ids need the repo keys and the account mapping lives in the repo config, so it opens the repo and lists config versions, cached ones come from the state DB). Task 8 adds `Engine::watch_plan() -> Vec<WatchTarget { stream, player, player_name, emulator, game, folder }>` (what `watch_targets` is built on), `Engine::process_names_by_emulator()` (local, for the poller) and `Engine::process_probe()`.

- [x] **Step 1: failing tests** (tempdir, real `notify`): writing a file under a watched folder sends its stream id within 2 s; 100 writes in a burst send at most a few ids, not 100; a folder created after `start` is picked up; a symlink pointing outside the folder produces nothing for writes outside; `set_folders` stops events from a removed folder; `watch_targets` on a test repo lists the mapped Cemu save folder.
- [x] **Step 2:** implement; pass; clippy; `cargo deny check`; spec; `just check`; commit `feat(engine): watch save folders for changes`.

### Task 5: watch mode: `core` session machine, `engine` process poller

**Why:** most players open the emulator themselves ([sync-model §Watch mode](../../specs/sync-model.md#watch-mode-fallback)). Spec fix in this task: the "Pull happens at emulator start" line can't restore while the emulator runs (I3); at emulator start LemuSync only *checks* the remote and warns when another device has a newer save.

**Files:** `crates/core/src/watch_session.rs` (new), `crates/engine/src/session.rs` (new), `crates/engine/src/process.rs` (`ProcessProbe::running_set`), spec `sync-model.md` §Watch mode, `architecture.md` §Engine internals (`session` row).

**Interfaces (produces):**
- `core`: `pub struct WatchSessions` keyed by emulator id: `fn on_running(&mut self, emulator, running: bool, now_ms) -> Vec<SessionAction>` and `fn on_change(&mut self, stream: StreamId, emulator, now_ms) -> Vec<SessionAction>`, with `SessionAction` = `CheckRemote(emulator)` (emulator started: fetch heads only, warn if behind), `TakeLease(StreamId)` (first change of a stream while its emulator runs), `UploadAfterQuiet(StreamId)` (emulator exited, or a change while it isn't running), `ReleaseLease(StreamId)` (after that upload). A stream whose lease take fails stays tracked; the upload still happens (leases never weaken safety, D6).
- `engine`: `ProcessProbe::running_set(&self, names_by_emulator: &[(String, Vec<String>)]) -> BTreeSet<String>` refreshing the process list once per call (one `/proc` walk for all emulators); `pub struct ProcessPoller` (`fn start(probe: Arc<dyn ProcessProbe>, names_by_emulator: Vec<(String, Vec<String>)>, poll: Duration, tx: Sender<(String, bool)>) -> ProcessPoller`, stops on drop) that calls it every `poll` (default 3 s; measure idle CPU in Task 10 and adjust) and sends `(emulator, running)` only on a change. `FakeProbe` gains the same method for tests.

- [x] **Step 1: failing tests.** `core`: start → `CheckRemote`; change while running → one `TakeLease` (a second change → nothing); exit → `UploadAfterQuiet` for every changed stream, then `ReleaseLease`; change while not running → `UploadAfterQuiet` and no lease; two emulators tracked independently. `engine`: the poller with `FakeProbe` sends start and stop once each.
- [x] **Step 2:** implement; pass; clippy; specs (W1, including the I3 fix above); `just check`; commit `feat(engine): add watch-mode sessions and the process poller`.

### Task 6: `daemon` Linux tray, notifications and power

**Why:** desktop presence (P4). SP2 proved each piece on XFCE ([desktop §Linux results](../../specs/platforms/desktop.md#linux-results-sp2-xfce), [reliability §Sleep and resume](../../specs/reliability.md#sleep-and-resume)).

**Files:** `crates/daemon/Cargo.toml`, `crates/daemon/src/platform/mod.rs` (traits + `Fake*` for tests), `crates/daemon/src/platform/linux/{tray,notify,power}.rs`, `crates/daemon/src/platform/other.rs` (no-op for other targets), `crates/daemon/src/i18n.rs` (Fluent loading like `crates/cli/src/i18n.rs`; share the code through a small module both crates include, or copy with a note; don't add a crate), `locales/{en,es}/lemusync.ftl`, `assets/tray/` icons if needed, specs `status-and-notifications.md` §Desktop tray, `desktop.md` (Code field).

**Interfaces (produces):**
- `pub struct TrayView { pub icon: TrayIcon, pub tooltip: String, pub status_line: String, pub paused: bool, pub recent: Vec<(String /*game ref*/, String /*label*/)>, pub menu: TrayMenuText /*localised menu labels*/ }`, `enum TrayIcon { Synced, Syncing, Paused, Offline, Attention, Error }`, `enum TrayAction { SyncNow, Pause1h, PauseUntilResume, Resume, Play(String), Open, Quit }`.
- `trait Tray { fn show(&self, view: &TrayView); }` built by `fn start_tray(tx: mpsc::Sender<TrayAction>) -> Box<dyn Tray>`; Linux: `ksni`, created lazily and retried when an `org.kde.StatusNotifierWatcher` appears on the bus (`NameOwnerChanged`), no panic and no error loop without one. Icons: freedesktop symbolic names (`emblem-synchronizing-symbolic` etc.) or bundled pixmaps; the syncing animation at ≤ 4 fps only while syncing.
- `trait Notifier { fn notify(&self, n: Notice) -> NoticeId; }` with `Notice { kind: NoticeKind (Attention | Error | Info), title, body, actions: Vec<(String /*id*/, String /*label*/)> }` and action callbacks sent on an `mpsc` channel as `(NoticeId, action id)`; Linux: `notify-rust` over D-Bus, works with `ActionInvoked`.
- `trait Power { fn block_idle(&self, reason: &str) -> IdleGuard; fn sleep_events(&self) -> mpsc::Receiver<SleepEdge> }` with `SleepEdge { Suspending, Resumed }`; Linux: logind `Inhibit("idle", …, "block")` per guard (dropped = released; also released after 10 min), plus `org.freedesktop.PowerManagement.Inhibit` when that service exists (XFCE), and the delay lock on `sleep` held while running, closed on `PrepareForSleep(true)` after the daemon acknowledges (`fn ack_suspend(&self)`), retaken on `PrepareForSleep(false)`.
- Strings (en + es): tray menu items, the status texts from [status-and-notifications §Status model](../../specs/ui/status-and-notifications.md#status-model), notification titles for conflict, foreign lease, auth error, quota error. `fn tray_view(status: &StatusInput, session: Option<&SessionInput>, t: &I18n) -> TrayView` in `crates/daemon/src/view.rs` (pure, tested); `StatusInput` (state `Synced | Scanning | Syncing | Paused | Offline | Attention | Error(ErrorKind)` plus counts) and `SessionInput { game, player }` are local types that Task 8 fills from `lemusync_api::Status`/`Session`. Notice texts: `fn notice(&NoticeEvent, &I18n) -> Notice` in `crates/daemon/src/notices.rs`. The syncing icon is static (no frames; within the ≤ 4 fps ceiling).

- [x] **Step 1: failing tests:** `tray_view` for each `StatusState` in en and es (text from the spec table); `Fake*` record calls; `IdleGuard` drop releases (fake). Linux D-Bus pieces are tested manually in Step 3, not in CI (no session bus there).
- [ ] **Step 2:** implement; pass; clippy; `cargo deny check`; `cargo xtask i18n-check`.
- [x] **Step 3: manual check on the Fedora PC (XFCE)** with a throwaway `--example` binary under `crates/daemon/examples/`: tray icon shows and swaps state; a notification with 2 actions reports the clicked action; `systemd-inhibit --list` shows the idle block while a guard lives and the sleep delay lock while running. Record date and versions in desktop §Linux results only if something differs from SP2. Don't suspend the machine.
- [x] **Step 4:** specs; `just check`; commit `feat(daemon): add the Linux tray, notifications and power hooks`.

### Task 7: `lemusync service` on Linux (systemd user unit)

**Files:** `crates/cli/src/service.rs` (new; unit text, argv runner trait, `install/uninstall/start/stop/status`), `crates/cli/src/main.rs` (subcommand), `locales/{en,es}/lemusync.ftl`, `crates/cli/tests/service.rs`, specs `config-and-cli.md` §Commands, `desktop.md`, `reliability.md` §Always on.

**Behaviour:**
- `service install [--headless]` writes `~/.config/systemd/user/lemusyncd.service` atomically: `ExecStart=<absolute path of lemusyncd next to the running lemusync, else from PATH>` (refuse a relative or non-existent path), `Restart=on-failure`, `RestartSec=2`, `StartLimitIntervalSec=600`, `StartLimitBurst=5` (the crash-loop rule), `After=`/`PartOf=`/`WantedBy=graphical-session.target`, `--headless` `WantedBy=default.target` (D39 replaced SP2's `default.target`; install and uninstall remove every `*.wants/lemusyncd.service` link). Then `systemctl --user daemon-reload` and `enable --now lemusyncd.service`. `--headless` also runs `loginctl enable-linger` and `ExecStart=… --headless`. All argv, no shell.
- `uninstall` disables, stops and removes the file (linger is left as it is and the output says how to turn it off). `start`/`stop` map to `systemctl --user`. `status` prints installed/enabled/active from `systemctl --user show -p LoadState,UnitFileState,ActiveState,NRestarts lemusyncd.service` and `--json`.
- Not Linux → usage error `service_unsupported` (Windows/macOS are Task 11).
- With `LEMUSYNC_TEST_ROOT` the unit goes under `<root>/config/systemd/user/` and the runner is always a fake that records argv to `<root>/systemctl.log` (`show` answers from `<root>/systemctl-show.txt`; `LEMUSYNC_TEST_DAEMON` names the daemon path), so tests never touch the real user manager.

- [x] **Step 1: failing tests:** golden unit text (normal and headless); install records `daemon-reload` then `enable --now`; uninstall order; `status --json` parses a recorded `show` output; en and es output; relative `lemusyncd` path refused.
- [x] **Step 2:** implement; pass; clippy; i18n-check; specs (the commands table and the "first slice" sentence); `just check`; commit `feat(cli): add service install/uninstall/start/stop/status on Linux`.

### Task 8: `engine::Service` and `lemusyncd`

**Depends on:** Tasks 1–7.

**Files:** `crates/engine/src/service.rs` (new), `crates/engine/src/events.rs` (new: builds `StatusFacts`, fans out `Event`s on a `tokio::sync::broadcast`), `crates/daemon/src/{main,run,logging,view}.rs`, `crates/engine/tests/service.rs`, `crates/daemon/tests/`, specs `architecture.md` §Process model, `reliability.md` §Always on and §Sleep and resume, `status-and-notifications.md` §Notification policy, `config-and-cli.md` (`logs`), `desktop.md`.

**Behaviour:**
- `Service::new(engine, ServiceConfig) -> (Service, ServiceHandle)`; `Service::run(self)` owns the `Engine` and loops over: commands from `ServiceHandle` (an `mpsc` of `(Request, oneshot reply)`), watcher stream ids → `Quiet`, poller edges → `WatchSessions`, scheduler deadlines, sleep edges. Sync work runs one at a time on the engine; `Status`, `Subscribe` and `Pause`/`Resume` are answered from the service's own state at once, even mid-sync ([D35](../../decisions.md#d35-the-cli-is-the-daemons-client-while-it-runs-2026-10-28)).
- After a quiet period the service syncs only the changed streams if `sync` can be narrowed cheaply; otherwise a full `sync` (it's idempotent and the rescan is fast, [reliability §Budgets](../../specs/reliability.md#budgets)). Snapshot rules and I3 are the engine's as today.
- Watch-mode actions call `lease_acquire` / `lease_release` with a non-interactive policy (never "take over" a live lease; a held lease → `LeaseWarning` event and notification, then upload anyway at exit, which may become a conflict, D6). `CheckRemote` → a fetch-only check; if the remote head is ahead → `Attention` notification "Newer save from <device>: close <emulator> and LemuSync updates it".
- `Suspending` → finish or stop at the next atomic step (the journal covers the rest), release held leases if the network allows, `ack_suspend`, all within 4 s. `Resumed` → `Trigger::Resume`. Transfers hold a `Power::block_idle` guard.
- `lemusyncd [--headless]`: takes the engine lock (`Locked` → exit with a clear message, B3 single instance), binds IPC, starts tray/notifier/power (skipped with `--headless`), maps `Status` → `TrayView` and events → `Notice`s with the 5-minute coalescing per stream and event type, tray actions → requests. `SIGTERM`/`SIGINT` → graceful stop: finish the atomic step, release leases, unlink the socket, exit 0.
- Logging: `tracing` to `<data>/logs/lemusyncd.log`, size-capped (rotate at 5 MiB, keep 3), every message through `redact`. `lemusync logs [--follow]` prints it (Task 9 wires the command if it needs IPC; reading the file needs none).

- [x] **Step 1: failing tests** (`Store::fs`, `FakeProbe`, fake platform): a write in a mapped save folder → upload after the quiet period (shortened in tests) → `StatusChanged` `Syncing` then `Synced`; `Status` answered while a sync is paused at a failpoint; emulator start + change + exit → lease taken, upload after exit, lease released; transient store faults → backoff, then success; `Pause` stops automatic syncs, `Manual` still syncs; a `Suspending` edge mid-transfer acks within the budget and leaves I1–I6 intact; a second `lemusyncd` → clear error; `SIGTERM` leaves no socket.
- [x] **Step 2:** implement; pass; clippy; i18n-check; specs; `just check`; commit `feat(daemon): run the engine as an always-on service`.
**As built:**
- `engine`: `Service::new(engine, ServiceConfig) -> (Service, ServiceHandle)`; `Service::run(self) -> Engine` (not `Send`: the state DB connection isn't `Sync`), so `Service::spawn(self) -> JoinHandle<Engine>` runs it on its own thread with a current-thread runtime. `ServiceConfig { quiet, schedule, poll, renew, finish, release }` (defaults 10 s, `ScheduleConfig::default()`, 3 s, 60 s, 3 s, 1 s). `ServiceHandle` (Clone): `call(Request) -> Response`, `subscribe() -> broadcast::Receiver<Event>`, `status() -> Status`, `suspend()` (returns once checkpointed), `resumed()`, `stop()`. Commands go through an internal `mpsc` of commands with `oneshot` replies; every engine call runs inside `drive`, which keeps answering `Status` (cached report), `Pause`, `Resume`, `Subscribe` and queues the rest. `events.rs` holds the status model (`StatusFacts` → `Status`, transfers counted from `TransferProgress`) and the `broadcast` bus (`EVENT_BUFFER` = 256). After a quiet period: a full `sync` (not narrowed). `Engine::remote_newer(emulator)` is the fetch-only check at emulator start. A pause failpoint `upload.before_manifest` was added for the tests.
- `daemon`: `run::run(handle, server, Option<Desktop>, I18n, shutdown)`; `Desktop::start()` bundles tray, notifier, power and their action channels; `status_input`/`session_input` map the api types onto `view`'s local inputs; `notice_for` + `notices::Coalescer` (5 min per stream and kind); `tray_request` maps clicks; `logging::init` (own `RotatingFile`, `RedactedLog` writer). `lemusyncd --version` prints the version (it used to print it and exit with no arguments).
- Release `lemusyncd` is 17.5 MB, over the 15 MB budget: Task 10.

- [ ] **Step 3: manual (Fedora PC):** `lemusync service install`, play BotW for a minute in Cemu opened by hand, close it: the tray shows syncing then synced, `lemusync status` (through IPC) shows the new version. Record surprises in the specs.

### Task 9: CLI over IPC, `pause`/`resume`, `logs`, `doctor` checks

**Depends on:** Task 8.

**Files:** `crates/cli/src/{main,daemon_client,play}.rs`, `crates/api/src/wire.rs` (remaining requests), `crates/engine/src/service.rs`, `crates/engine/src/query.rs` (`doctor`), `locales/`, `crates/cli/tests/`, specs `config-and-cli.md` §Commands, `architecture.md` §Process model, `security.md` §Local IPC.

**Behaviour:**
- Every CLI command first tries `Client::connect`; `NotRunning` → run the engine in-process as today. Commands with prompts collect their input on the CLI side first (passphrase, recovery key and secret keys over the same-user socket, kept in `Zeroizing` buffers; never logged). `play` through the daemon streams `PlayEvent`s and sends the lease/conflict prompt as a server→client frame the CLI answers (one pending prompt per connection; a dropped connection = the non-interactive answer, exit 4/5 rules unchanged).
- New commands (P5 parity with the tray): `lemusync pause [--for <duration>]` (`1h`, `30m`; none = until resume), `lemusync resume`, `lemusync logs [--follow]`. `status` shows the daemon's `StatusState` and the session overlay when the daemon runs.
- `doctor` adds: daemon installed/running (socket answers), service enabled, linger (headless), a StatusNotifierWatcher on the session bus (else the GNOME AppIndicator hint), a notification server (`GetServerInformation`).

- [x] **Step 1: failing tests:** with a test daemon on a tempdir socket, `status --json`, `sync`, `pause --for 1h`, `resume`, `conflicts` go through IPC (a marker in the reply proves it) and give the same JSON as in-process; with no daemon they still work in-process; `play` with a held lease through the daemon and no TTY → exit 5; en and es for the new strings; `doctor --json` has the new checks.
- [x] **Step 2:** implement; pass; clippy; i18n-check; specs; `just check`; commit `feat(cli): talk to the daemon over IPC`.

**Deviation (as built):** the IPC marker is not in the command's reply, so that replies stay byte-identical to in-process: `lemusyncd` logs `ipc client <name> connected` (info) per connection and `crates/cli/tests/daemon.rs` counts those lines; `status --json` also carries the `daemon` object only from the daemon. Pause and resume need a running daemon (in-process they are the usage error `daemon_not_running`). The held-lease `play` → exit 5 test runs against a fake daemon on a tempdir socket (`crates/cli/src/daemon_client.rs` tests), because a binary test can't set up a repo without the OS keystore. `backend add gdrive`/`backend login` refuse while the daemon runs (`daemon_running_oauth`, D28 + D35). The daemon-side request mapping lives in `crates/engine/src/{remote,remote_play}.rs`, and `logs` in `crates/cli/src/logs_cmd.rs`.

### Task 10: daemon budgets

**Files:** `crates/daemon/benches/` or `xtask` (`cargo xtask budgets`), `justfile` (`just bench`), CI job (non-blocking report first), spec `reliability.md` §Budgets and V-REL-3.

- [x] **Step 1:** measure on the Fedora PC, release build: idle CPU over 10 min (`/proc/<pid>/stat` utime+stime) with 3 emulators' process names polled; daemon RSS; cold start to "IPC ready"; change detected → upload started; `lemusyncd` binary size. If the binary is over 15 MB, record by how much and what adds the most (`cargo bloat` if available); don't change the budget.
- [x] **Step 2:** a CI check fails on a >20 % regression of binary size and rescan time against values in the spec (idle CPU and latency stay manual).
- [x] **Step 3:** record (date, machine, numbers) in §Budgets; `just check`; commit `perf(daemon): measure and guard the daemon budgets`.

**Deviation (as built):** `cargo xtask budgets [idle-secs]` runs the release daemon isolated with no repo (the binary only has the OS keystore) and one fake Cemu install granted, so the poll looks for Cemu's 3 process names; it is the only bundled profile with a loader, and the poll does one `/proc` walk whatever the number of emulators. Change → upload is measured in-process through `engine::Service` in the engine bench (1 s quiet period, reported as quiet + overhead). Binary-size contributors come from `nm` on an unstripped build, because `cargo bloat` isn't installed. Idle CPU is 0.71 %, over the ≈ 0 % budget (V-DAEMON-2 refuted); fixing the poll is left for a follow-up. The CI job `budgets` is non-blocking (`continue-on-error`) and its references are the PC's numbers.

### Task 11: Windows and macOS (blocked: SP2 on those OSes)

Blocked until the maintainer runs SP2 there (V-DESK-1, V-DESK-3, V-REL-2, deferred 2026-10-08). Then: named pipe with a current-user DACL and client-token check; Task Scheduler logon task; `SMAppService` LaunchAgent from the app bundle; tray (`tray-icon` or native) and toasts/`UNUserNotificationCenter`; `SetThreadExecutionState` / IOPM assertions. Plan the steps from the spike results.

### Task 12: Flatpak packaging strategy (maintainer decision)

V-DESK-2: static `--filesystem=~/.var/app/<id>` per emulator, the document portal, or a native daemon package next to a Flatpak UI. Needs the maintainer's call and a decision entry.

### Task 13: M2 soak (maintainer)

72 hours on Linux (then Windows and macOS after Task 11) with sleep/resume cycles: no missed or late sync beyond the budgets, tray states verified ([roadmap](../roadmap.md#phase-2-desktop-daemon)). Record in the roadmap.

## Verify

- **V-DAEMON-1** (new, Task 8) Narrowing a daemon-triggered sync to the changed streams is cheap enough to bother; otherwise a full sync each time.
- **V-DAEMON-2** (new, Task 5, measured in Task 10) A 3 s process poll costs ≈ 0 % idle CPU on the Fedora PC. **Refuted 2026-10-29:** 0.71 % (426 ticks over 600 s), from the `sysinfo` walk of every process; a cheaper poll (read only `/proc/<pid>/comm`, or a longer interval) is open.
- Existing: V-REL-1, V-REL-4, V-REL-5 (sleep on GNOME/KDE, lingering, desktop idle suspend), V-DESK-1/2/3, V-REL-2, V-REL-3.

## Self-review

- **Coverage:** roadmap Phase 2 deliverables: `lemusyncd` + `ipc` (2, 8), service install (7; Windows/macOS 11), tray (6, 8), actionable notifications in Fluent en/es (6, 8), launcher mode over the daemon (9) and watch mode with process detection (3, 5, 8), sleep inhibition and resume (6, 8), `doctor` (9), daemon budgets carried over from M1 (10).
- **Not planned (P1):** retention/GC, local history pruning, hot-reload of `config.toml` (a restart picks changes up; add when the UI writes config in Phase 3), UnifiedPush.
- **Open for the maintainer:** Flatpak (12), the watch-mode "lease the last played game at start" question ([sync-model §Open questions](../../specs/sync-model.md#open-questions)) stays open until there's usage data.
- **Names:** `Request`, `Response`, `Event`, `Status`, `StatusFacts`, `Quiet`, `Scheduler`, `Trigger`, `WatchSessions`, `SessionAction`, `SaveWatcher`, `ProcessPoller`, `Service`, `ServiceHandle`, `TrayView`, `Notice`, `Power` are used consistently across Tasks 1–9.
