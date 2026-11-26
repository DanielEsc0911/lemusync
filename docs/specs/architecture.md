# Architecture

> **Status:** Decided · **Related:** [rules](../rules.md) (B1, B2, B3), [decisions](../decisions.md) (D1–D4, D10, D12, D18), [sync-model](sync-model.md), [security](security.md), [reliability](reliability.md) · **Code:** `crates/` (process model: `crates/engine/src/service.rs`, `crates/daemon/src/run.rs`), `app/`, `crates/bridge`, `app/hook/build.dart`

## Purpose
How the pieces fit together: components, crate layers, process model per platform, the engine API, and the repository tree.

## Components
```
                 ┌────────────── Flutter UI (app/) — same Dart code everywhere ──────────────┐
                 └───────┬───────────────────────────────────────────────┬───────────────────┘
          desktop: IPC   │                                               │  mobile: in-process
                 ┌───────▼────────┐                              ┌───────▼────────────────────┐
                 │ lemusyncd     │  CLI ──IPC──►                │ lemusync-bridge (cdylib)  │
                 │ engine + tray  │                              │  FRB (Dart) + UniFFI       │
                 │ + notifications│                              │  (Kotlin/Swift) → 1 engine │
                 └───────┬────────┘                              └───────┬────────────────────┘
                         └─────────────► lemusync-engine ◄──────────────┘
                                          │      │       │
                               core ◄─────┘   crypto   store ──► user's storage (S3, WebDAV, SFTP…)
```

## Layers
Crates live under `crates/` with package names `lemusync-<dir>`. Dependencies only point down. `cargo xtask layers` enforces this table:

| Crate | Responsibility | May depend on (internal) |
|---|---|---|
| `core` | Domain model and pure logic: streams, versions, heads, conflict detection, path validation, profile schema. **No I/O, no async** | — |
| `crypto` | Key derivation, AEAD, keyed hashing, key wrapping. Pure | — |
| `store` | Backend interface and adapters (OpenDAL and custom); raw object I/O | — |
| `api` | Engine API: request/response/event types and the `EngineApi` trait. Stable and versioned | — |
| `engine` | Orchestration: scanning, watching, snapshots, transfers, leases, journal, state DB, scheduler. Implements `EngineApi` | core, crypto, store, api |
| `ipc` | Local-socket transport: framing, handshake, `EngineApi` client and server | api |
| `daemon` | `lemusyncd` binary: service lifecycle, tray, notifications, sleep inhibition | api, engine, ipc |
| `cli` | `lemusync` binary | api, engine, ipc |
| `bridge` | Mobile/desktop UI bindings: flutter_rust_bridge + UniFFI in one cdylib | api, engine, ipc |

Rules:
- `unsafe_code = "forbid"` in every crate except `bridge` (generated FFI) ([B2](../rules.md#b2-security-first-no-backdoors-no-exploits)).
- `core` and `crypto` are fully unit- and property-testable without mocks.
- Dev-dependencies may cross layers (test helpers). Normal dependencies may not.

## Process model
| Platform | Processes | Engine lives in |
|---|---|---|
| Windows, macOS, Linux desktop | `lemusyncd` (user-level service, owns tray and notifications) + Flutter app (on demand) + `lemusync` CLI | daemon |
| Headless (Pi, server) | `lemusyncd --headless` + CLI | daemon |
| Android | App process. Kotlin service/jobs keep it alive during work | bridge (in-process) |
| iOS | App process (also used by App Intents and background tasks) | bridge (in-process) |

- **One engine per user profile.** The engine takes an exclusive lock on its state directory. A second instance exits with a clear error. On desktop the CLI talks to the daemon. If no daemon is running, the CLI may run the engine itself for one command, holding the same lock. While the daemon runs, it answers `status`, `pause`, `resume` and event subscriptions at once from its own status model and runs other commands one at a time ([D35](../decisions.md#d35-the-cli-is-the-daemons-client-while-it-runs-2026-10-28)). The CLI side is `crates/cli/src/daemon_client.rs`: an `Api` that is either the local `Engine` or a `Daemon` connection, with the same calls and results; the daemon side maps each request to the engine call in `crates/engine/src/remote.rs` and runs `play` with prompts as server→client frames (`crates/engine/src/remote_play.rs`, `crates/daemon/src/run.rs`). A client that drops mid-`play` gets the non-interactive answer for every later question; the session still uploads and releases its lease.
- **Desktop UI ↔ daemon:** the Flutter app → FRB → `bridge` (IPC client) → `ipc` → daemon. The UI uses the same `EngineApi` as mobile, so the Dart code doesn't change between platforms.
- **The daemon's service** (`crates/engine/src/service.rs`, `crates/daemon/src/{main,run}.rs`): `Service` owns the one `Engine` on its own thread (a current-thread tokio runtime; the engine's state DB connection isn't `Sync`) and runs engine work one call at a time. It feeds the `core` machines (quiet period, scheduler and backoff, watch-mode sessions) from the save-folder watcher, the process poller, requests and sleep edges, and sleeps until the next deadline. While an engine call is in flight it still answers `status` (the last report), `pause`, `resume` and `subscribe`, and queues the rest. After a quiet period it runs a full `sync` (idempotent; the rescan of an unchanged folder is cheap, [reliability §Budgets](reliability.md#budgets)). The watched folders are refreshed after each successful sync and kept when the storage can't be reached. Everything else talks to it through a cloneable `ServiceHandle` (`call`, `subscribe`, `status`, `suspend`, `resumed`, `stop`). `lemusyncd` takes the engine lock first (a second one exits with "already running"), then binds the socket, starts the service thread and, unless `--headless`, the tray, notifications and sleep hooks; `SIGTERM`/`SIGINT` or a `shutdown` request stop it cleanly (leases released, socket unlinked, exit 0).
- **Tray on the main thread:** the tray event loop needs the main thread (macOS, GTK). Async work runs on a tokio runtime on other threads.

## Engine API
Defined in `crates/api`, versioned (`api_version`), and identical in-process and over IPC.

- **Commands:** status · sync now · pause/resume · list games · history · restore version · resolve conflict · start/end play session · players CRUD · account mappings · devices · backends CRUD/test · grants CRUD · settings get/set · profiles list/validate · diagnostics.
- **Events (stream):** `StatusChanged` · `SessionChanged` · `ConflictDetected` · `LeaseWarning` · `TransferProgress` · `ConnectionEvent` · `Error`.
- **Wire types** (`crates/api/src/wire.rs`): handshake `Hello {api_version, client}` → `HelloReply {api_version, daemon_version}`. `Request` is `{"cmd": …}` with `status`, `sync {dry_run}`, `pause {until_ms}` (null = until resume), `resume`, `conflicts`, `games`, `history {game, emu, player}`, `lease_status`, `doctor`, `subscribe`, `shutdown`, plus one request per remaining CLI command (`init`, `join`, `backend_add_fs`, `backend_add_s3`, `grant_*`, `players_*`, `scan`, `accounts_*`, `restore`, `resolve`, `lease_release`, `alias_*`, `config`, `play`, `answer`). During `play` the daemon sends `play_event` and `prompt` frames and the client answers each `prompt` with `answer`. `Response` is `{"ok": <request name or paused/resumed/subscribed/shutting_down>, "data": <report>}` or `{"ok": "error", "data": ErrorBody}`. `Event` is `{"event": …, "data": …}`: `status_changed`, `session_changed`, `conflict_detected`, `lease_warning`, `transfer_progress`, `connection`, `error`, `remote_newer` (watch mode: an emulator started while another device has a newer save; nothing is restored while it runs). The status model (`Status`, `StatusState`, `ErrorKind`, `Session`, and `Status::from_facts`) is in `crates/api/src/status.rs`.
- **`TransferProgress`** (`crates/api/src/lib.rs`): player, emulator, game, phase (`checking`, `uploading`, `downloading`) and files and bytes done out of the phase's total. The engine sends it to the sink set with `Engine::set_progress` during `sync`, `restore` and `accounts recover`. Sync and restore reports carry what each stream moved (`files`, `bytes`).
- **`ConnectionEvent`** (`crates/api/src/lib.rs`): `retrying` (operation, attempt, max attempts, delay in ms) while the store retries a transient error, and `recovered` once, on a success when no other request is still retrying. The engine sends it to the sink set with `Engine::set_connection_sink` for every backend call ([reliability §Network](reliability.md#network)).
- **One status model** feeds every surface: tray, notifications, Live Updates, Live Activities, UI ([status-and-notifications](ui/status-and-notifications.md)).

## IPC
- Unix domain socket in the user runtime dir (mode `0600`, dir `0700`). Windows named pipe with a DACL for the current user only.
- Peer check: the same UID/SID as the daemon (`SO_PEERCRED` / `getpeereid` / pipe client token). Mismatch → close.
- Frames: u32 big-endian length + JSON (serde), max 1 MiB. Handshake: `{api_version, client}`. Unknown versions are rejected with an upgrade hint. Within one API version, clients skip events they don't know (`Client::subscribe`); a frame that is not JSON is still fatal.
- Code: `crates/ipc` (`frame`, `socket`, `server`, `client`). Socket path `$XDG_RUNTIME_DIR/lemusync/daemon.sock` (`LEMUSYNC_TEST_ROOT=<dir>` → `<dir>/run/daemon.sock`); no runtime dir → error, never `/tmp`. `Server::bind` refuses a symlinked dir, a dir owned by another uid or a non-socket file at the path, and replaces a socket only when nothing answers on it. The oversized length is refused before the body is read; a bad request gets `bad_request` and the connection closes. A peer has 5 s to send `Hello`; `Server::accept` only checks the peer and returns an `Incoming`, whose `handshake` the daemon runs on the connection's own task, so a silent client never holds up the next one. Transport logs go through `tracing`. Windows returns `Unsupported` until the named pipe lands (Phase 2 Task 11).
- UI client (planned, Phase 3b, [D43](../decisions.md#d43-the-desktop-app-is-a-client-of-the-daemon-over-the-same-ipc-as-the-cli-2026-11-01)): the desktop app reaches the daemon through `lemusync_ipc::Client` in `crates/bridge/src/link/`, one short connection per call plus one subscription connection that reconnects with backoff. It subscribes first, then fetches the snapshot, so no event is lost. See [app-shell](ui/app-shell.md).

## Bridge
`crates/bridge` builds one library (`cdylib` + `staticlib`) with both bindings ([D4](../decisions.md#d4-mobile-one-engine-in-the-app-process-two-thin-bindings-2026-10-07)):

| Part | How |
|---|---|
| Dart | [flutter_rust_bridge](https://cjycode.com/flutter_rust_bridge/) 2.13.0 (Rust `=2.13.0`, Dart `2.13.0`, codegen `2.13.0`, all three must match). Public items in `crates/bridge/src/api/` get bindings in `app/lib/bridge/`. Config: `app/flutter_rust_bridge.yaml`. Regenerate with `flutter_rust_bridge_codegen generate` in `app/`, then `just fmt` |
| Build | FRB's **native-assets** backend: `app/hook/build.dart` (Dart build hook, `flutter_rust_bridge_hooks` → `native_toolchain_rust` 1.0.4) runs Cargo for the target and bundles the library. No per-platform glue (cargokit) in `android/`, `ios/`, `linux/`… The hook requires `crates/bridge/rust-toolchain.toml` with an exact channel and every target it may build. Keep its channel equal to the root one |
| Kotlin, Swift | [UniFFI](https://mozilla.github.io/uniffi-rs/) 0.32.2, proc-macro mode (`setup_scaffolding!("lemusync")`, `#[uniffi::export]`), surface in `crates/bridge/src/native.rs`. Bindings: `cargo run -p lemusync-bridge --features bindgen --bin uniffi-bindgen -- generate --library <lib> --language kotlin --language swift --out-dir <dir>` |
| `unsafe` | The crate sets `unsafe_code = "deny"` (not `forbid`) in its own `[lints]` and allows it only on `mod frb_generated` and `mod native`. Lints are off for the generated module |
| TLS | `rustls` 0.23 with the `ring` provider (no `aws-lc-rs`, no CMake for cross builds) and `rustls-platform-verifier` 0.7.1 ([security §Network](security.md#network)) |
| Android glue | Kotlin `Bridge` (`app/android/.../Bridge.kt`) calls `System.loadLibrary("lemusync_bridge")`, the same `.so` Dart opens, then `Bridge.initTls(context)` (JNI entry in `crates/bridge/src/android.rs`, `jni` 0.22) before any TLS. Gradle adds `net.java.dev.jna:jna:5.19.1@aar` (UniFFI's Kotlin bindings use JNA) and `org.rustls:rustls-platform-verifier:0.2.0`, which is published only to a Maven repo on GitHub; that repo is restricted to the `org.rustls` group. Its version must match `rustls-platform-verifier-android` in `Cargo.lock`. Generated Kotlin bindings: `app/android/app/src/main/kotlin/uniffi/lemusync/` |
| Daemon link | `src/link/` (plain Rust, 3b Task 3) is the app's client of `lemusyncd`: it depends on `lemusync-api`, `lemusync-engine` (the library rule only) and `lemusync-ipc`, plus `tokio` and `futures-util`; never an engine ([app-shell](ui/app-shell.md#connection)). It owns a 1-worker tokio runtime for the event watcher. `liblemusync_bridge.so` was 2.11 MB and is 4.60 MB with it (`cargo build -p lemusync-bridge --release`; 2.98 MB while `link` was unused; V-SHELL-14). Symbol sizes in the unstripped build (2026-11-01): `crc_fast` 0.76 MB, `core` 0.53 MB, `lemusync_api` 0.40 MB, `rustls` 0.36 MB, `ring` 0.11 MB plus about 0.3 MB of C/asm, `tokio` 0.10 MB, `lemusync_engine` 7 KB. Surface for Dart: `api/daemon.rs` (`snapshot`, `games`, `sync_now`, and `WatchHandle` with `new`, `watch` (a `Stream` of `LinkEvent`) and `stop`; 3c adds `history`, `conflicts`, `leases`, `restore`, `resolve`, `lease_release`, which return `Result<_, ConnectionState>` (thrown in Dart, caught by `guard` in `app/lib/app/call_result.dart`), and `PlayHandle` with `new`, `play` (a `Stream` of `PlayUpdate`), `answer` and `stop`); the DTOs are generated from `link/model.rs`, `link/flow_model.rs` and `link/play.rs`. Enums with data need `freezed` in the Dart app |
| R8 (release) | `app/android/app/proguard-rules.pro` keeps `com.sun.jna.**`, `uniffi.**` and `org.rustls.platformverifier.**`. Without it the release app crashes at the first UniFFI call (`UnsatisfiedLinkError: Can't obtain peer field ID for class com.sun.jna.Pointer`) and the verifier fails (`failed to call native verifier`). Debug and profile builds don't shrink, so they hide this |

**SP1 results, Linux (2026-10-17).** Fedora 44 PC, XFCE on X11, AMD Radeon 860M, 1920×1200 at 60 Hz, Flutter 3.47.6 (Impeller, OpenGL ES), Rust 1.99.0. Probe: `app/tool/sp1_probe.dart` (spike only; `flutter run --profile -d linux -t tool/sp1_probe.dart`).

| Measure | Result | Budget |
|---|---|---|
| Dart → Rust sync call | Works. Two calls returned `#1`, `#2` from the shared counter | — |
| Cold start to first rasterized frame (profile, `--trace-startup`, 3 runs, includes `RustLib.init`) | 82–88 ms | ≤ 1 s ([design-system](ui/design-system.md#performance)) |
| Frames during a 10 s rotation animation (profile) | 619 frames; build p50 0.24 ms / p99 0.38 ms; raster p50 0.43 ms / p99 0.71 ms; 4 frames over 8.3 ms total span | ≤ 8 ms per frame |
| Release bundle (`flutter build linux --release`) | 23 MB: Flutter engine 17 MB, app AOT 3.2 MB, `liblemusync_bridge.so` 2.0 MB | — |
| Idle RSS, release app after 6 s | 143 MiB (profile probe: 169 MiB max) | none for the UI (the daemon's is in [reliability](reliability.md)) |
| Rust dependency graph | 112 crates for Linux (FRB, UniFFI, rustls, platform verifier) | — |

The 120 Hz case was not measurable (60 Hz display). It was measured on CachyOS (below).

**SP1 results, CachyOS (2026-10-18).** CachyOS (kernel 7.2.9-2-cachyos), KDE Plasma 6.8.0 on Wayland, Intel Iris Xe (Alder Lake-P) + NVIDIA GeForce RTX 3050 Mobile, 1920×1080 at 144 Hz, Flutter 3.47.6, Rust 1.99.0. Same probe and commands as on Fedora.

| Measure | Result | Budget |
|---|---|---|
| Dart → Rust sync call | Works (`#1`, `#2`). HTTPS through the platform verifier returns a response | — |
| Cold start to first rasterized frame (3 runs) | 127–135 ms | ≤ 1 s |
| Frames during the 10 s animation (profile) | 611 frames (about 61 fps) while `refresh_hz` reports 144. Build p50 0.19 ms / p99 0.32 ms; raster p50 0.33 ms / p99 0.62 ms; 3 frames over 8.3 ms. Same count with `GDK_BACKEND=wayland` (618) and `GDK_BACKEND=x11` (617; raster p99 6.0 ms, 19 over 8.3 ms) | 120 fps on high-refresh displays ([design-system](ui/design-system.md#performance)) |
| Release bundle | 23 MB: engine 17 MB, app AOT 3.2 MB, bridge 2.0 MB (same as Fedora) | — |
| Idle RSS, release app after 6 s | 186 MiB (profile probe: 239 MiB) | none for the UI |

Frame work fits well inside the 120 Hz budget, but the Linux app is paced at about 60 fps on a 144 Hz display. The cause is not confirmed (V-DESK-5 in [desktop](platforms/desktop.md#verify)).

**SP1 results, Android emulator (2026-10-18).** Pixel 8 AVD, Android 14 (API 34, x86_64, Google Play image), emulator 36.5.10 on the Fedora PC, NDK 28.2.13676358, JDK 21 (Android Studio's JBR). Emulator frame and start times use a software GPU path, so they say nothing about real devices.

| Check | Result |
|---|---|
| One instance, two bindings | Dart `core_version` → `#1`, Kotlin `enginePing()` through UniFFI → `pong #2`, Dart again → `#3`. Profile and release builds. This confirms D4 on Android |
| TLS with Android's verifier (release build) | `www.rust-lang.org` accepted. Expired → `invalid peer certificate: Expired`; self-signed and untrusted root → `UnknownIssuer`; wrong host → `certificate not valid for name` |
| Real app, `flutter build apk --release --split-per-abi` | Builds the bridge for all three ABIs. arm64-v8a 17.4 MB (bridge 1.9 MB), armeabi-v7a 13.8 MB (1.2 MB), x86_64 19.0 MB (2.2 MB). Launches with no crash |

Windows, macOS, iOS and a real Android device (Galaxy A25) are still to run ([Phase 0 plan](../progress/plans/phase-0-foundations.md#sp1-flutter--rust-bridge-on-every-target)). Swift bindings generate on Linux (`enginePing()`); the in-app check needs the iOS run.

## Engine internals
| Module | Job |
|---|---|
| `scan` | Resolve profiles + grants → candidate streams |
| `watch` | File watching (notify): each mapped save folder recursively, symlinks never followed and events that resolve outside the folder dropped; a missing folder is watched through its nearest existing parent until it appears; one id per stream per 250 ms burst. Rescans come from the scheduler (`core::schedule`). Code: `crates/engine/src/watch.rs` |
| `session` | Launcher/watch sessions, process detection. Watch mode: the pure session machine in `core` (`crates/core/src/watch_session.rs`) and the process poller (`crates/engine/src/session.rs`, `ProcessProbe::running_set` in `process.rs`); launcher mode: `play.rs`. The daemon drives watch mode in `service.rs`: a change while the emulator runs takes the lease (never a takeover; held elsewhere → `lease_warning`), the upload waits for exit plus the quiet period, the lease is released after it; at emulator start `Engine::remote_newer` lists the version graph only |
| `snapshot` | Quiet-period check, consistent copy, hashing |
| `transfer` | Upload/download with retry and backoff |
| `restore` | Staged, journaled, atomic restore with backup |
| `lease` | Acquire, renew, release, stale handling |
| `heads` | Version graph, divergence → conflicts |
| `repo` | Remote layout, encryption, manifests |
| `state` | SQLite state DB + journal |
| `retention` | Remote retention and blob GC, local history pruning |
| `power` | Sleep inhibition, resume handling (via platform hooks) |
| `events` | Status model and event bus: raw `StatusFacts` (transfers counted from `TransferProgress`) → `Status`, a `StatusChanged` only on a real change, every event on a `broadcast` channel. Code: `crates/engine/src/events.rs`, used by `service.rs` |

Async runtime: tokio. Errors: typed per crate (`thiserror`). Applications map them to user-facing messages through i18n keys.

## Repository tree
```
AGENTS.md  CLAUDE.md  README.md  LICENSE  SECURITY.md  CONTRIBUTING.md  CHANGELOG.md
Cargo.toml  rust-toolchain.toml  justfile  deny.toml  clippy.toml  .cargo/config.toml
crates/{core,crypto,store,api,engine,ipc,daemon,cli,bridge}/
xtask/                    repo checks: docs-check, layers, i18n-check
app/                      Flutter app
  lib/main.dart
  lib/app/                router, theme bootstrap, providers
  lib/design/             tokens + components (design system)
  lib/features/<name>/    screens, widgets, state for one feature
  lib/l10n/               ARB files (+ generated/)
  lib/bridge/             generated FRB bindings
  hook/build.dart         builds crates/bridge for the target (Dart build hook)
  android/ ios/ macos/ windows/ linux/   native code: services, notifications, Live Activities, App Intents
profiles/                 emulator profiles (TOML) + fixtures/
locales/{en,es}/          Fluent files for Rust (tray, notifications, CLI)
packaging/                systemd units, launchd plists, Windows tasks, Flatpak/installer manifests
docs/                     this documentation
.claude/skills/           project skills
.github/workflows/        CI
```
