# Phase 3c, flows

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Also follow the project skills: `lemusync-feature` (always), `lemusync-docs` (every task), `lemusync-ui` (Tasks 5–13), `lemusync-security` (Tasks 2, 3, 4, 9: IPC client, secrets crossing the bridge, paths from the user).

**Goal:** Every user flow in [navigation §Flows](../../specs/ui/navigation.md#flows) except the power and Settings flows works in the desktop app: game detail with history and **Restore**, **Resolve conflict**, **Play** with the lease and conflict questions, **Lease warning**, **First run (create)** and **Join existing**. After 3c a new user can go from a running daemon to a first synced save without the terminal, which is what M3 measures ([roadmap](../roadmap.md#phase-3-flutter-app-desktop)). Third of the four Phase 3 plans. Linux first, no per-OS visual branch.

**Architecture:** Same shape as 3b ([app-shell](../../specs/ui/app-shell.md)): Flutter → FRB → `crates/bridge/src/link/` → `lemusync_ipc::Client` → `lemusyncd`. The daemon already serves every request these flows need (`history`, `restore`, `conflicts`, `resolve`, `lease_status`, `lease_release`, `play` + `answer`, `init`, `join`, `backend_add_fs`, `backend_add_s3`, `grant_add`, `grant_list`, `scan`, `players_add`, `players_list`, `accounts_map`, `confirm_first_sync`: `crates/api/src/wire.rs`, `crates/engine/src/remote.rs`), so 3c adds **no protocol and no engine code**; if a task finds it needs one, it stops and reports. The bridge gains one call per request (one short connection each, each with a timeout, as `link/calls.rs`) and one **play session**: `play` keeps its connection open, streams `play_event` and `prompt` frames, and takes one `answer` per prompt on the same connection (`crates/cli/src/daemon_client.rs` `play` is the reference client). Dropping the session's connection gives the daemon's non-interactive answer for every later question, and the session still uploads and releases its lease (architecture §Process model), so closing the app mid-game is safe. In Dart, flows are pushed routes over the 3b shell (`Navigator` pages, still no router package: P1), each with its own small controller tested against `FakeDaemonLink`.

**Tech Stack:** as 3b (Rust 1.99.0, FRB 2.13.0, Flutter 3.47.6). One new Dart package, `file_selector` ([D48](../../decisions.md#d48-the-app-picks-folders-with-file_selector-2026-11-26)), added in Task 11 where it is first used; nothing else.

**Spec:** [navigation](../../specs/ui/navigation.md) §Flows, §Adaptive containers; [app-shell](../../specs/ui/app-shell.md); [design-system](../../specs/ui/design-system.md) §Components, §Accessibility; [security](../../specs/security.md) §Local IPC, §Credentials, §File-system confinement; [sync-model](../../specs/sync-model.md) (conflicts, leases, restore); [players-and-accounts](../../specs/players-and-accounts.md) §Discovery; [config-and-cli](../../specs/config-and-cli.md) §Commands (the CLI is the behavioural reference for each flow); [i18n](../../specs/i18n.md) §Terminology. Read only the sections a task names.

## Decisions for the maintainer (Task 1 records the answers)

Answered 2026-11-26: 1 → [D46](../../decisions.md#d46-secrets-typed-in-the-desktop-app-cross-the-bridge-once-dart-to-rust-2026-11-26), 2 → [D47](../../decisions.md#d47-the-apps-first-run-offers-s3-compatible-storage-and-a-local-folder-google-drive-waits-for-its-login-in-the-app-2026-11-26), 3 → [D48](../../decisions.md#d48-the-app-picks-folders-with-file_selector-2026-11-26), 4 → [D49](../../decisions.md#d49-first-run-has-no-game-picking-and-no-device-naming-for-now-2026-11-26), each as proposed. 5 is the proposed default until the maintainer says otherwise. The text below is kept as asked.

1. **Secrets typed in the app cross the bridge (Dart → Rust).** The passphrase, the recovery key and the S3 secret key are typed in a Flutter field, so they exist as a Dart `String` (immutable, garbage-collected, cannot be zeroed) until the bridge turns them into `lemusync_api::Secret`. *Proposed:* accept, with mitigations: the field is obscured, has no autofill or suggestions, is cleared and its controller disposed right after the call, is never put in `AppState`, logs or exceptions, and the Rust side wraps it in `Secret` at the FRB boundary. This extends 3b's "no secrets cross the bridge" (which stays true for everything Rust → Dart) and security §Local IPC. The recovery key from `init` is the one secret that goes Rust → Dart, to be shown once.
2. **Storage choices in the app's first run.** Google Drive's login is CLI-only while the daemon runs (`daemon_running_oauth`, [D28](../../decisions.md)). *Proposed:* the app offers **S3-compatible (R2)** and **Local folder** in 3c; Google Drive is not shown until its login works from the app (3d or later; F2: nothing half-wired). The CLI keeps all three (P5 holds: the app is a subset until then, the CLI is not).
3. **Folder picker.** The local-folder backend and "grant a folder the scan did not suggest" need a path. Flutter has no folder dialog. *Proposed:* add `file_selector` (flutter.dev first-party, `getDirectoryPath`, Linux/macOS/Windows; Linux through the XDG portal or GTK). The scan's suggested grants are confirmed with one tap and need no picker. Alternative: a typed path field only (`PathField`), no package.
4. **"Pick games" and "This device (name)" steps.** The engine has no per-game opt-out (a stream is every save the profile finds) and no device-rename request (`init`/`join` name the device). *Proposed:* 3c drops "Pick games" (the found games are shown as a read-only list before the initial sync) and shows the device name read-only; navigation §Flows is changed to match. Adding either later is engine work with its own plan.
5. **Start before 3b's review.** 3b Task 11 Step 5 (the maintainer's live review) is still open. *Proposed:* 3c Tasks 1–4 (spec, bridge, no UI) start now; UI tasks start after the review, and any 3b change it asks for lands first.

## Global constraints

- Everything in 3b's Global constraints still holds (tokens only, no `Platform.isX` visuals, `LsSize.target`, semantics, reduce motion, 200 % text, goldens on Linux only with `pump()`, every new or changed PNG opened, fake link in every Dart test, the Rust side tested against a real `lemusync_ipc::Server` in a temp dir, never the maintainer's socket, W5 trailer only, toolchain PATH).
- **Every write asks first.** Restore, resolve, take over a lease, release a lease and the first sync each go through one confirmation that says what happens to the current save ("your current save will be backed up", navigation §Restore). Play anyway warns that two devices will write the same save. Destructive confirmations default to Cancel.
- **The app never decides for the user what the CLI would ask.** The play session answers a `prompt` only from a user tap; if the sheet is dismissed or the app closes, it answers `Refuse` (or drops the connection, which means the same).
- **Errors are values.** Every new call returns a result type with the daemon's `code` (config-and-cli §Error codes) and a localized text chosen by code; the raw `message` is secondary detail only (app-shell §Connection). Nothing throws to a widget.
- **Paths:** Dart still never receives save or game paths except where the user must see them (the scan's suggested grant folders and the chosen local-folder backend path, which are the user's own folders). Grants are only added by an explicit user action (B2).
- **P6:** every string in `app_en.arb` (with `@description`) and `app_es.arb`; each task lists its keys in its As built note. Neutral Latin American Spanish, "tú". CLI lines stay untranslated code.
- **No polling.** Lists (history, conflicts, leases) are fetched when their screen opens, after an action, and when `last_sync_ms` changes, as Games is.
- Each task ends with `just check` green (plus `cd app && flutter test`), then its commit, with this plan's boxes ticked and its board row updated in the same commit.

## Order

Tasks 1–4 (spec and bridge) run first and need no UI decision. Tasks 5–8 (games flows) after 3b's review. Tasks 9–12 (setup flows) follow the games flows. Task 13 is the gate: stop and wait for the maintainer.

## File map

| Path | Responsibility | Task |
|---|---|---|
| `docs/specs/ui/app-shell.md` §Flows as built (3c), `docs/specs/ui/navigation.md` §Flows | Spec of the flows as the app runs them | 1, each task |
| `docs/decisions.md` | D46 (secrets typed in the app), and whatever decisions 2–4 need | 1 |
| `crates/bridge/src/link/model.rs` | New DTOs: `HistoryDto`, `VersionDto`, `ConflictDto`, `HeadDto`, `LeaseDto`, `PlayUpdate`, `PlayQuestionDto`, `PlayAnswerDto`, `ScanDto`, `InstallDto`, `SetupResult`, `CallError` | 2, 3, 9 |
| `crates/bridge/src/link/calls.rs` → `link/calls/{read,write,setup}.rs` | One function per request, one short connection each, with a timeout (split when it passes one responsibility, B1) | 2, 9 |
| `crates/bridge/src/link/play.rs` | `PlaySession`: one connection, events and prompts out, answers in, `Refuse` on drop | 3 |
| `crates/bridge/src/api/daemon.rs` | FRB surface for the new calls and `PlayHandle` | 4, 9 |
| `app/lib/app/daemon_link.dart`, `app/test/support/fake_link.dart` | `DaemonLink` gains the calls and `play`; the fake scripts them | 4, 9 |
| `app/lib/design/components/{version_timeline,lease_badge,conflict_compare,confirm_dialog,wizard_step,secret_field}.dart` | New components from design-system §Components (+ gallery entries) | 5, 10 |
| `app/lib/screens/games/game_detail.dart` | Game detail: streams per player, lease, history, Play | 6 |
| `app/lib/screens/flows/{restore,resolve,play_sheet}.dart` | The three game flows and their controllers | 6, 7, 8 |
| `app/lib/screens/setup/{welcome,storage,passphrase,recovery_key,player,emulators,map_accounts,initial_sync}.dart`, `setup_controller.dart` | First run and Join | 11, 12 |
| `app/test/**` | Controller, flow, component and golden tests | every task |

---

### Task 1: spec, decisions and this plan

**Files:** this plan, `docs/progress/README.md` (board row), `docs/specs/ui/navigation.md` §Flows (decision 4), `docs/specs/ui/app-shell.md` (a §Flows as built (3c) section, planned), `docs/specs/security.md` §Local IPC (decision 1), `docs/decisions.md` (D46 and others the answers need), `docs/specs/ui/design-system.md` (component status).

- [x] **Step 1:** write this plan and its board row (2026-11-26).
- [x] **Step 2:** the maintainer answers decisions 1–5 (1–4 on 2026-11-26, as proposed; 5 stands as proposed). Record each in the spec it changes, and as a decision where it is one (W4). Change any task below the answers affect.
- [x] **Step 3:** `cargo xtask docs-check`; **commit** `docs(app): plan the Phase 3c flows`.

### Task 2: bridge read and write calls for the game flows

**Files:** `crates/bridge/src/link/{model,calls}.rs` (split `calls` by responsibility), tests next to them, `docs/specs/ui/app-shell.md` §Data flow, `docs/specs/architecture.md` §Bridge (surface list).

**Interfaces:** `history(game, emu, player) → Result<HistoryDto, CallError>`, `conflicts() → Result<Vec<ConflictDto>, CallError>`, `lease_status() → Result<Vec<LeaseDto>, CallError>`, `players() → Result<Vec<String>, CallError>`, `restore(game, emu, player, version)`, `resolve(game, emu, player, version)`, `lease_release(game)` → `Result<WriteDto, CallError>` (the wire takes the game id only). `CallError { connection: Option<ConnectionState>, code: String, message: String }` (no connection → the 3b state; a `Response::Error` → its code). Timeouts: reads 10 s, `history` 30 s and writes 120 s (restore and resolve move files; V-FLOW-1). DTOs carry ids, names, times, sizes, device names; no paths.

- [ ] **Step 1: failing tests** against a real `lemusync_ipc::Server` (the `link/fake.rs` pattern): each call sends the right `Request`, maps the matching `Response`, maps `Response::Error` to `CallError` with its code, maps no listener to `NotRunning`, times out with paused time; DTO conversion keeps every field the screens need and drops paths (destructuring test, as 3b).
- [ ] **Step 2:** FAIL. **Step 3:** implement. **Step 4:** `cargo test -p lemusync-bridge`, `cargo xtask layers`, `just check`.
- [ ] **Step 5:** docs; tick; board. **Commit** `feat(bridge): add history, restore, conflict and lease calls`.

### Task 3: bridge play session

**Files:** `crates/bridge/src/link/play.rs`, `model.rs` (`PlayUpdate`, `PlayQuestionDto`, `PlayAnswerDto`, `PlayResultDto`), tests, `docs/specs/ui/app-shell.md` §Data flow.

**Interfaces:** `PlaySession::start(path, game, emu, player) → (Stream<PlayUpdate>, Answerer)`. `PlayUpdate` is `Step(Pulling | WaitingForLease | Playing | Uploading)`, `Question(PlayQuestionDto)`, `Done(PlayResultDto)` or `Failed(CallError)`. `Answerer::answer(PlayAnswerDto)` sends `Request::Answer` on the session's connection; a second answer to the same question is ignored; dropping the `Answerer` or the stream ends the session (the daemon then answers `Refuse` itself). No timeout on the session as a whole (a game runs for hours); a 10 s timeout on connect and handshake only.

- [ ] **Step 1: failing tests** with a scripted server: steps arrive in order; a lease question then `Wait`/`PlayAnyway`/`TakeOver` sends the matching `Answer`; a conflict question then `Keep { version }` and `Keep { None }`; dropping the stream mid-question closes the connection and the server sees EOF; `Done` carries the report; a `Response::Error` becomes `Failed`; a daemon that disappears mid-game becomes `Failed` with `not_running`.
- [ ] **Step 2:** FAIL. **Step 3:** implement on the watcher's runtime (`link::runtime()`, V-SHELL-1). **Step 4:** `just check`.
- [ ] **Step 5:** docs; tick; board. **Commit** `feat(bridge): run a play session over the daemon link`.

### Task 4: FRB surface and Dart link

**Files:** `crates/bridge/src/api/daemon.rs`, regenerated `app/lib/bridge/`, `app/lib/app/daemon_link.dart`, `app/test/support/fake_link.dart`, tests.

**Interfaces:** `DaemonLink` gains one method per Task 2 call and `Stream<PlayUpdate> play(...)` plus `answer(PlayAnswer)`; `FrbDaemonLink` is still the only importer of `bridge/`; `FakeDaemonLink` scripts results and play sessions and records calls. Cancelling the play stream calls the handle's `stop` once (as 3b's `WatchHandle`).

- [ ] **Step 1: failing tests** (Dart, fake only): result mapping, play stream cancel → stop once, answer forwarded once. Rust: the new API functions compile through codegen (`flutter_rust_bridge_codegen generate`, no diff after `just fmt`).
- [ ] **Step 2–4:** FAIL, implement, `flutter test`, `just check`.
- [ ] **Step 5:** docs; tick; board. **Commit** `feat(app): reach the game flows through the daemon link`.

### Task 5: components for the game flows

**Files:** `app/lib/design/components/{version_timeline,lease_badge,conflict_compare,confirm_dialog}.dart`, gallery entries, widget tests and goldens, `docs/specs/ui/design-system.md` §Components and §Code paths.

**Interfaces:** `VersionTimeline(versions, onRestore)`: newest first, device, time, size, "current" marker; lazy. `LeaseBadge(device, age, stale)`: "In use on {device} · {age}" with an icon, stale says so (not color alone). `ConflictCompare(heads, onChoose)`: one column per head (device, time, size, files), side by side at ≥ 600 dp, stacked below. `ConfirmDialog(title, body, confirmLabel, destructive)`: Cancel is the default focus.

- [ ] **Step 1: failing tests:** semantics labels, 200 % text at 320 px, keyboard focus order, Cancel focused first, lazy timeline with 1000 versions. Goldens: each component at 390 and 900, light and dark, en (8 PNGs), opened and reviewed.
- [ ] **Step 2–4:** FAIL, implement with tokens only, `cargo xtask brand --check`, `just check`.
- [ ] **Step 5:** docs; tick; board. **Commit** `feat(design): add the timeline, lease, compare and confirm components`.

### Task 6: game detail and Restore

**Files:** `app/lib/screens/games/game_detail.dart`, `app/lib/screens/flows/restore.dart` (+ controller), `games_screen.dart` (cards get `onTap`; cards become focusable: 3b's pinning test changes), tests, `docs/specs/ui/app-shell.md`, navigation §Restore.

**Flow:** Games → card → detail (pushed full screen below 1200 dp; side pane at ≥ 1200 dp, navigation §Adaptive containers) → one section per player stream (`LeaseBadge` when a lease is held elsewhere, last version, History) → version → **Restore** → `ConfirmDialog` ("your current save will be backed up") → progress (`TransferProgress` for that game) → done banner, list refreshed. With several players the detail asks "which player" by showing each stream separately (no hidden default).

- [ ] **Step 1: failing tests** (fake link): open detail from card tap and from keyboard (Enter); side pane at 1200; history loads on open and on `last_sync_ms` change; Restore asks, Cancel does nothing, Confirm calls `restore` once with the right ids; error code → localized text + `lemusync history` line; Back returns focus to the card. Goldens: detail at 390 and 1400, light, en and es (4 PNGs).
- [ ] **Step 2–4:** FAIL, implement, checks.
- [ ] **Step 5:** docs; tick; board. **Commit** `feat(app): show game detail and restore a version`.

### Task 7: Resolve conflict

**Files:** `app/lib/screens/flows/resolve.dart` (+ controller), Home's attention banner (conflicts → opens the flow), game detail (a conflicted stream shows "Resolve"), `AppState.cardStatus` (a conflicted stream's card shows `attention`: closes the 3b gap noted in app-shell §App state), tests, docs.

**Flow:** Home attention item or game detail → `ConflictCompare` → choose → `ConfirmDialog` ("the other versions stay in history") → `resolve` → done. With several conflicts, Home's item opens a list first.

- [ ] **Step 1: failing tests:** conflicts fetched on open and when `status.conflicts` changes; choose → confirm → `resolve` with the chosen head; cancel → nothing; card shows attention only for the conflicted game; error mapping. Goldens: compare at 390 and 900, light, en (2 PNGs).
- [ ] **Step 2–4, Step 5:** as above. **Commit** `feat(app): resolve a conflict`.

### Task 8: Play and lease warning

**Files:** `app/lib/screens/flows/play_sheet.dart` (+ controller), `GameCard.onPlay` wired for launchable games, game detail Play, lease section (Wait / Play anyway / Take over when stale; Release when this device holds a stale lease, `lease_release`), tests, docs (navigation §Play and §Lease warning as built).

**Flow:** Play (default player; "Play as…" when the repo has several) → sheet: pulling → lease question if held elsewhere (Wait / Play anyway / Take over only when stale, Take over and Play anyway confirm) → conflict question (choose a head or play this device's copy) → emulator running (the 3b session overlay takes over Home) → uploading → done. Closing the sheet while the emulator runs keeps the session (the daemon owns it); closing it on a question answers `Refuse`.

- [ ] **Step 1: failing tests:** each step renders; each answer sent once; dismissing on a question sends `Refuse`; dismiss while playing does not stop the session; `Done` shows the upload result; `Failed` shows the code text; Play hidden for a game with no file; "Play as…" only with several players. Goldens: sheet on the lease question at 390 and 900, light, en and es (4 PNGs).
- [ ] **Step 2–4, Step 5:** as above. **Commit** `feat(app): play a game with lease and conflict questions`.

### Task 9: bridge setup calls

**Files:** `link/calls/setup.rs`, `model.rs` (`ScanDto`, `InstallDto`, `AccountDto`, `SetupResult`), `api/daemon.rs`, Dart link and fake, tests, security §Local IPC (decision 1 as built).

**Interfaces:** `init(passphrase) → Result<InitDto { device, recovery_key }, CallError>`, `join_passphrase(p)`, `join_recovery(k)`, `backend_add_fs(path)`, `backend_add_s3(endpoint, bucket, region, access_key_id, secret)`, `grant_add(path)`, `scan() → ScanDto`, `players_add(name)`, `accounts_map(player, emu, id, choice)`, `confirm_first_sync()`. Secret parameters become `Secret` at the FRB function's first line; `InitDto.recovery_key` is the only secret sent to Dart. Timeouts: 120 s for `init`, `join` and `backend_add_s3` (they probe the storage).

- [ ] **Step 1: failing tests:** each request shape; a secret never appears in `Debug` of any DTO or error; `CallError.message` from a server that echoes the request does not contain the secret (redaction test); scan maps installs, needed grants and accounts.
- [ ] **Step 2–4, Step 5:** as above. **Commit** `feat(bridge): add the setup calls`.

### Task 10: setup components

**Files:** `app/lib/design/components/{wizard_step,secret_field}.dart` , gallery, tests, goldens, design-system §Components.

`WizardStep(title, body, primary, secondary, step, of)`: same steps in every container; Back keeps entered values except secrets. `SecretField`: obscured, show/hide toggle, no autofill, no suggestions, clears on dispose; strength hint for a new passphrase (length only, no external list).

- [ ] **Step 1: failing tests** (semantics, 200 %, focus, the field clears its controller on dispose). Goldens: a wizard step at 390 and 900, light and dark, en (4 PNGs).
- [ ] **Step 2–4, Step 5:** as above. **Commit** `feat(design): add the wizard step and secret field`.

### Task 11: First run (create)

**Files:** `app/lib/screens/setup/*`, `setup_controller.dart`, `app/pubspec.yaml` (`file_selector`, pinned; licence in the app's licence list), Home's not-set-up state (its CLI lines stay, plus a "Set up LemuSync" button), tests, docs (navigation §First run as built, app-shell).

**Flow (as decided in Task 1):** Welcome (language) → Create → Storage (S3/R2 or Local folder → details → probe) → Passphrase (twice) → Recovery key (shown once, "I saved it" checkbox before Next; copy button; never stored by the app) → You (player name) → This device (read-only name) → Find emulators (scan → confirm suggested grants) → found games (read-only list) → Initial sync (`confirm_first_sync`, then live progress) → Home.

- [ ] **Step 1: failing tests:** every step and its Back; a failed probe stays on Storage with the code text; secrets are cleared after the call and absent from the controller state; the recovery key screen cannot be skipped; an interrupted setup (app closed after `init`) resumes at the right step from the daemon's state (`status` says set up but no player, etc.; V-FLOW-2). Goldens: Storage and Recovery key at 390, light, en and es (4 PNGs).
- [ ] **Step 2–4, Step 5:** as above. **Commit** `feat(app): set up LemuSync from the app`.

### Task 12: Join existing

**Files:** `app/lib/screens/setup/{join,map_accounts}.dart`, tests, docs (navigation §Join as built).

**Flow:** Welcome → Join → Storage → Passphrase or recovery key → This device → Map accounts (scan's accounts, proposed by name, confirm; keep/replace when both sides have files, `accounts_map` `NeedsChoice`) → Find emulators → Initial sync (pull) → Home.

- [ ] **Step 1: failing tests:** both secret kinds; map proposal by name; `NeedsChoice` asks keep or replace with a confirmation on Replace; pull progress. Goldens: Map accounts at 390 and 900, light, en (2 PNGs).
- [ ] **Step 2–4, Step 5:** as above. **Commit** `feat(app): join an existing repo from the app`.

### Task 13: golden, accessibility and performance pass, then maintainer review (gate)

As 3b Task 11 for every 3c screen: 200 % text and 320 px in all four themes, semantic order, reduce motion, rebuild budget on the play sheet during 200 progress events, profile build.

- [ ] **Step 1–4:** tests, checks, profile build, docs, commit `test(app): add the flow accessibility and rebuild checks`.
- [ ] **Step 5: maintainer review (stop here).** On the PC with the real daemon: restore a version and see the backup; resolve a real conflict (two devices); play a game from the app and from the tray at the same time on two devices to see the lease question; run the first-run flow on a fresh test profile (`LEMUSYNC_TEST_ROOT` daemon, never the real config) to a first synced save and time it against M3's 3 minutes. Say what to change. Tick when approved.

## Verify

- `V-FLOW-1` (Tasks 2, 13): how long `restore` and `resolve` take on the maintainer's real saves through the daemon's one-at-a-time queue; 120 s is a starting timeout.
- `V-FLOW-2` (Task 11): the daemon's `status` tells every partial setup state apart (repo created but no player, no grants, first sync not confirmed), so the wizard can resume. If not, the wizard restarts at the first missing step that `status` can see, and the gap is reported.
- `V-FLOW-3` (Task 3): a play session survives the app closing (the daemon finishes the upload and releases the lease) when started from the app, as it does from the CLI.
- `V-FLOW-4` (Task 9, decision 3): `file_selector`'s Linux folder dialog works under KDE Wayland on the maintainer's PC and inside a future Flatpak (SP11).

## Self-review

- **Coverage:** navigation §Flows: Restore (6), Resolve (7), Play (8), Lease warning (8), First run (11), Join (12). Add emulator and Storage change are Settings flows (3d). Deep links and notifications (3d).
- **No new protocol:** every flow uses a request that exists today; Task 2's DTOs follow the CLI's `--json` shapes.
- **Rules:** B2 (decision 1, secrets cleared, grants only by user action, paths only the user's own); B3 (play survives app exit, timeouts, no polling); F2 (Google Drive hidden until it works, decision 2); F3 (same steps in every container); P5 (every flow has its CLI line on errors); P6; W1; W2; W3 (V-FLOW-1–4); W4 (decisions logged in Task 1).
