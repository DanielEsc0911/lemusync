# Decisions

> **Status:** Decided · **Related:** [rules](rules.md), [architecture](specs/architecture.md), [product](specs/product.md) · **Code:** —

Append-only. To change a decision, add a new entry that says `Supersedes Dn`, and mark the old one `Superseded by Dm`. Format: decision · why · alternatives rejected.

### D1 Rust for the engine, daemon and CLI (2026-10-07)
Memory safety (B2), native performance and small binaries (B3, P7), and one codebase that compiles for every target, including Android, iOS and ARM Linux. Rejected: Go (weaker mobile story), C++ (memory safety), Kotlin Multiplatform (weak headless/Pi story).

### D2 Flutter for every UI (2026-10-07)
Flutter draws every pixel itself, so the look is identical everywhere (F1). Impeller gives smooth animation on mobile (F2). One Dart codebase means one navigation implementation (F3). Rust is reached through flutter_rust_bridge. This replaces the early chat suggestion of Tauri or native UIs. Rejected: Tauri (the webview differs per OS; WebKitGTK performance on Linux), Compose Multiplatform (JVM on desktop is heavy), Slint (iOS support immature), three native UIs (they would drift apart, breaking F1 and F3).

### D3 Desktop: headless daemon owns sync and tray; UI is a client (2026-10-07)
Sync must run 24/7 without the UI open (B3) and stay light (P7). `lemusyncd` runs as a user-level service and owns the tray, notifications and engine. The Flutter app connects over same-user IPC. Headless installs run the same daemon without a tray.

### D4 Mobile: one engine in the app process, two thin bindings (2026-10-07)
On Android and iOS, background work, notifications, Live Updates and Live Activities are native (Kotlin/Swift) and must work with no Flutter UI running (B3). The single `lemusync-bridge` cdylib exports flutter_rust_bridge for Dart **and** UniFFI for Kotlin/Swift, so the process has exactly one engine instance. Rejected: headless Flutter engines for background work (heavier, more fragile). Verify in spike SP1 that both bindings work from one library.

### D5 Append-only, content-addressed repo with derived heads (2026-10-07)
Objects get unique names and are never overwritten, so correctness doesn't depend on atomic compare-and-swap. That's required to support Google Drive, WebDAV and similar backends. Divergence is detected from the parent links. See [sync-model](specs/sync-model.md).

### D6 Leases are advisory; divergence detection is the safety net (2026-10-07)
Not every backend can provide a perfect distributed lock. Leases give the UX ("in use on Desktop"); heads and parents guarantee nothing is lost.

### D7 No game save-format parsing in core (2026-10-07)
Conflicts are compared by metadata (device, time, session length, changed files, size). This works for every game and avoids parsing untrusted binary data (B2, P3). Narrow emulator *metadata* readers (e.g., an index that maps opaque folders to title IDs) are allowed only as small, fuzzed parsers in core, each with its own decision entry.

### D8 Emulator profiles are declarative TOML, shipped inside releases (2026-10-07)
No scripts, plugins or remote downloads (B2). Users may add local custom profiles. Community profiles arrive through reviewed PRs and signed releases.

### D9 Capability tiers instead of an allow-list (2026-10-07)
Every emulator gets the safety features. The engine derives Full, Standard or Basic from the profile's layout ([emulator-profiles §Tiers](specs/emulator-profiles.md#tiers)).

### D10 Storage through Apache OpenDAL, plus custom adapters where needed (2026-10-07)
OpenDAL covers S3, WebDAV, SFTP, Google Drive, OneDrive, Dropbox, local filesystem and more, with conditional-write options. MEGA isn't in OpenDAL; options are in [storage-backends](specs/storage-backends.md#mega).

### D11 End-to-end encryption is mandatory (2026-10-07)
Having one mode is simpler (P1) and safer (B2). Users can't browse saves on the storage directly; they restore through LemuSync.

### D12 SQLite for local state and the journal (2026-10-07)
Battle-tested crash safety (WAL), available on every target, bundled in the binary.

### D13 No server, telemetry or accounts in v1; push only optional and later (2026-10-07)
P2, B2. Lock warnings come from devices' own checks. UnifiedPush (self-hosted ntfy) on Android and an optional relay are Phase 6+ ideas.

### D14 Launcher mode is primary; watch mode is the fallback (2026-10-07)
Launching the emulator is the only way to reliably know when a session starts and ends on every OS, especially mobile.

### D15 Save states are opt-in (2026-10-07)
They're large and tied to the emulator version. They're stored as a separate stream kind.

### D16 License MPL-2.0; hosting GitHub (2026-10-07)
MPL-2.0 is file-level copyleft and compatible with the App Store and Play Store. GitHub provides free macOS, Windows, Linux and ARM runners, which iOS and macOS builds need.

### D17 Spanish is neutral Latin American (`es`) (2026-10-07)
Uses `tú`, no `vosotros`, and neutral vocabulary. One Spanish locale.

### D18 Flutter app conventions: Riverpod (no codegen) and go_router (2026-10-07)
Riverpod handles the async streams that come from Rust. go_router's `StatefulShellRoute` gives one navigation tree with adaptive containers (F3). No code generation (P1).

### D19 i18n sources: ARB for Dart, Fluent for Rust, native files for extensions (2026-10-07)
Each layer uses its platform-standard format, and `cargo xtask i18n-check` enforces en/es parity across all of them. See [i18n](specs/i18n.md).

### D20 "Player" is the term for a person (2026-10-07)
Used in code, docs and UI (`es`: "jugador"). One term everywhere avoids translation drift.

### D21 Encryption from RustCrypto/BLAKE3 primitives, not the `age` file format (2026-10-07)
The early chat suggested `age`. The repo needs AEAD with associated data bound to each object's kind and id, plus keyed content ids. `age` provides neither. The construction in [security §Cryptography](specs/security.md#cryptography) uses audited primitives only, with no custom algorithms.

### D22 App and bundle id `io.github.danielesc0911.lemusync` (2026-10-07)
The project has no domain. A reverse-DNS id under the maintainer's GitHub namespace is unique to them and valid on Android, iOS, macOS, Windows and Linux (lowercase). It's only permanent after the first store publication, so it can still change before then if a domain is bought.

### D23 Distribution before store accounts (2026-10-07)
There's no Apple Developer or Google Play account yet. GitHub Releases ship `.apk` (self-generated release key), `.msi`, `.exe`, `.dmg` (ad-hoc signed), `.deb`, `.rpm`, `.AppImage` and `.tar.gz`, each with SHA-256 checksums and GitHub build-provenance attestations. iOS is limited to developer testing on the maintainer's own device until a membership exists. Refines D16. Details: [release](specs/release.md).

### D24 Standard per-format packaging tools (2026-10-07)
WiX (`.msi`), Inno Setup (`.exe`), `hdiutil` (`.dmg`), nfpm (`.deb`/`.rpm`), appimagetool (`.AppImage`), `tar`. Each is the de-facto tool for its format, and the release path keeps few third-party layers (B2). Rejected for now: all-in-one packagers (e.g. fastforge, cargo-packager). None was confirmed to cover all eight requested formats, and each adds a layer to the supply chain. Revisit if V-PKG-5 finds one that does.

### D25 SFTP only on Linux and macOS for now (2026-10-08)
SFTP is offered only where OpenDAL's `sftp` service works: Linux and macOS desktop. It drives the system `ssh` binary through the `openssh` crate, which is unix-only (`compile_error!` on other targets), can't spawn processes on iOS and has no `ssh` on Android ([finding](specs/storage-backends.md#sftp-on-mobile-and-windows)). Windows, Android and iOS leave SFTP out for now. It needs the system OpenSSH client at runtime: macOS ships it; on Linux the packages should recommend it (follow-up in [storage-backends](specs/storage-backends.md#open-questions)). Why: KISS and YAGNI (P1), and no custom SSH code in the security-critical path yet (B2). Rejected for now: writing our own SFTP adapter. Revisit with a pure-Rust adapter (`russh` + `russh-sftp`, or `ssh2`) when there's demand (V-STORE-4).

### D26 Google Drive is the backend for the first real test (2026-10-18)
The first end-to-end test (M1a, [roadmap](progress/roadmap.md#phase-1-sync-engine--cli-desktop)) syncs through Google Drive, pulled forward from Phase 6. Both machines reach it from anywhere, it was live-tested in SP6, and the maintainer has an account. `fs` is built at the same time, for tests. S3, WebDAV and SFTP stay in Phase 1 and come after this test. The test uses the maintainer's own OAuth client (type "Desktop app", consent screen in Testing), entered at setup. Its secret is kept in the OS keystore, never in config. A public client for official builds stays open (V-STORE-2). Why: the maintainer's choice, and the least setup for two machines on different networks. Rejected for the first test: SFTP to the Fedora PC (needs `sshd` and LAN reachability), WebDAV, S3. **Superseded for the first test by [D31](#d31-cloudflare-r2-replaces-google-drive-for-the-first-real-test-2026-10-20)**; the Drive adapter stays.

### D27 `store` refreshes Google tokens itself; OpenDAL gets only an access token (2026-10-18)
Answers V-STORE-7. `crates/store` runs the OAuth refresh itself (form body, its own HTTP client) and builds the OpenDAL `gdrive` operator with `access_token` only. OpenDAL then never refreshes: `opendal-service-gdrive` 0.59.5 `backend.rs` sets a given access token's expiry to `Timestamp::MAX`. The store rebuilds the operator 60 s before the token expires. So the refresh token and client secret never enter a URL. Every store error is still redacted (V-SEC-4). Rejected: patching OpenDAL first (slower; an upstream issue is still worth filing).

### D28 OAuth loopback listener during login only (2026-10-18)
The desktop OAuth flow needs a redirect target. Only during `lemusync backend add gdrive` (and `backend login`), the CLI binds `127.0.0.1` on a random port, accepts exactly one request whose `state` matches, and closes after it or after 5 minutes. This is the one exception to "no inbound listeners" ([security §Network](specs/security.md#network)). It never runs in the daemon. Rejected: pasting the code by hand (worse UX; Google removed the out-of-band flow).

### D29 M1 is split: first "both saves on both PCs", then leases and play (2026-10-18)
M1a: encrypted repo, players, account mappings, scan, `sync` with versions, heads, fast-forward and backup-first restore. Conflicts are detected and kept, not resolved. M1b: the rest of the M1 exit (leases, launcher mode, SC1–SC3, SC5). Why: the maintainer wants the first real test soon, and M1a's formats are final, so M1b builds on the same data (P1).

### D30 Repo-config heads merge automatically (2026-10-18)
Several heads of the repo-config stream are merged by union instead of being shown as a conflict. Players are kept by id. Each device writes only its own account mappings. The merge version lists all heads as parents. Why: config edits are additive and per device, so a union never loses data, and asking the player would be noise. If two devices add the same player name at once, both players are kept, the later id gets a ` (2)` suffix, and the CLI says so.

### D31 Cloudflare R2 replaces Google Drive for the first real test (2026-10-20)
The first real test uses an S3-compatible Cloudflare R2 bucket instead of Google Drive (D26). It starts on the Fedora PC alone: set up, sync, play BotW, sync again, and check the new version and a restore. The CachyOS laptop joins afterwards. `crates/store` gets an `s3` adapter (OpenDAL `s3`, static keys, `~/.aws` and `AWS_*` loading disabled, our rustls client). The access key id is in `config.toml`; the secret key is typed at a no-echo prompt and kept in the OS keystore. `backend add s3` probes with a write, read, list and delete under `lemusync/.probe/` before saving anything. Why: the maintainer's choice. R2 documents atomic `If-None-Match` on `PutObject` ([R2 S3 API](https://developers.cloudflare.com/r2/api/s3/api/), read 2026-10-20), so `create_only` is native, not emulated as on Drive (V-STORE-8 doesn't apply), and there's no OAuth client or consent screen to manage. The live race test is `s3_conformance` (V-STORE-1). Rejected: keeping Drive for this test.
### D32 New Cemu accounts are generated like Cemu does, never copied (2026-10-20)
When LemuSync creates a Cemu account (choice c in [players-and-accounts](specs/players-and-accounts.md#creating-a-cemu-account-sp8)), it writes `account.dat` the way Cemu's own "Create account" does: a new id, a random `Uuid`, `TransferableIdBase` from it, the default `MiiData` for the name, and every online field empty. The file is described as data in the profile (P3). Why: spike SP8 showed Cemu needs nothing from another account, and Cemu accepted a file built this way (listed it, BotW booted and saved under it). A copied template would carry the source account's online fields (`AccountId`, `PrincipalId`, password cache) into a second account. Rejected: copying an existing account as a template (the earlier V-ACC-3 wording); editing `settings.xml` to make the new account active (another file to write, and Cemu already falls back to the first account).

### D33 The project is named LemuSync (2026-10-27)
The project was called LeemuSync and is now LemuSync: lemur + emu + sync, where "emu" is shared by lemur and emulator. Why: there's no "leemur", so the old spelling didn't match its own etymology. Crates, binaries (`lemusync`, `lemusyncd`), env vars (`LEMUSYNC_*`), app id (`io.github.danielesc0911.lemusync`, D22 otherwise unchanged), local folders and the remote layout prefix (`lemusync/`) all change with it. No migration of old paths: the project is pre-alpha (P1). Rejected: keeping LeemuSync; migrating old folders and the old remote prefix.

### D34 Game names are found on each device, never stored in the repo (2026-10-27)
LemuSync finds installed games and their names at runtime on each device ([emulator-profiles §Game discovery](specs/emulator-profiles.md#game-discovery)): from `config.toml` `games.<key>`, the game folders the emulator lists in its own granted config, and the user's `game_dirs`, reading each game's metadata file. Names aren't written to the repo, so a device without the game shows the bare key. Sync stays driven by save folders and doesn't need the catalog. Game folders are read for metadata only (one bounded listing per listed folder and one bounded XML file per game), never written or created, and no symlink below a listed folder is followed; a folder counts as approved because the emulator's granted config or the user's `game_dirs` lists it ([security §File-system confinement](specs/security.md#file-system-confinement)). Why: no new remote format (P1, formats stay v1), and names come from data the player already has. Rejected: storing names in repo config (a new synced field for a cosmetic value); asking for a grant per game folder (game folders aren't written, and one more prompt per folder hurts setup).

### D35 The CLI is the daemon's client while it runs (2026-10-28)
When `lemusyncd` runs, it owns the one engine (it holds the state lock) and every CLI command goes to it over IPC ([architecture §IPC](specs/architecture.md#ipc)). When no daemon answers, the CLI runs the engine itself for that command, as in Phase 1. The daemon runs engine work one command at a time and answers `status`, `pause`, `resume` and event subscriptions from its own status model at once, even mid-sync. Prompts the CLI needs (passphrase, lease and conflict choices in `play`) are asked on the CLI side and travel as IPC frames over the same-user socket. Why: one engine per user profile is how B3 avoids two writers, and the desktop UI will use the same IPC (Phase 3), so the CLI proves it first (P5). Rejected: the daemon lending its lock to the CLI for a command (the tray and notifications would go blind during `play`); a second engine with finer-grained locks (two writers on one state DB).

### D36 Phase 2 builds the daemon on Linux first (2026-10-28)
The [Phase 2 plan](progress/plans/phase-2-daemon.md) builds and tests `lemusyncd` on Linux (the M1 machines) with every platform piece behind a small trait. Windows and macOS get their implementations after SP2 runs on them, which the maintainer deferred on 2026-10-08 (V-DESK-1, V-DESK-3, V-REL-2); until then they compile with no-op implementations, so CI stays green. Why: the spikes that exist are Linux ones, and no facts are invented for the other OSes (W3). Rejected: writing Windows and macOS code now from documentation only.

### D37 Dev builds keep line tables only (2026-10-28)
`[profile.dev]` sets `debug = "line-tables-only"`. Panics, backtraces and test failures still show file and line; a debugger can step by line but cannot inspect local variables. Why: full debug info made `target/` grow past 100 GB on the maintainer's machine. Anyone who needs full debug info can build with `CARGO_PROFILE_DEV_DEBUG=true`. Rejected: full debug info (disk use); no debug info (loses file:line in backtraces).

### D38 The Linux keystore client never starts, unlocks or creates a store (2026-10-29)
On Linux LemuSync talks to the Secret Service with its own small client (`crates/engine/src/secret_service.rs`) instead of `keyring`. It checks `NameHasOwner` first and then sends every call to the owner's unique name with `NO_AUTO_START`. It never calls `Unlock`, `CreateCollection` or `SetAlias`, never runs a prompt, and touches only items with `service = lemusync`. No owner, no bus or a locked store is `secrets_unavailable`, a temporary state the scheduler retries with backoff. Why: on 2026-10-29 the daemon, started at boot by linger, made `keyring` D-Bus-activate GNOME Keyring before KDE's KWallet was up, which took the user's secret store away from KWallet ([desktop §Secret Service activation race under linger](specs/platforms/desktop.md#secret-service-activation-race-under-linger)). `keyring` 3.6.3 hard-codes the well-known name with auto-start allowed and unlocks collections itself, so a check before calling it still leaves a race and the unlock prompts (B2, P2). macOS and Windows keep `keyring`. Rejected: `keyring` plus a `NameHasOwner` check (the owner can leave between the check and the call, and the unlocks stay); zbus (its blocking API inside the engine's tokio runtime is a risk, and zbus 4 is unmaintained); a plain, unencrypted session (secrets would cross the bus in clear).

### D39 The desktop unit starts with the graphical session (2026-10-29)
`lemusync service install` writes `After=graphical-session.target`, `PartOf=graphical-session.target` and `WantedBy=graphical-session.target`, so the daemon starts with the desktop session and stops at logout, and linger never starts it before login. `--headless` keeps `WantedBy=default.target` and linger, because running without a login is its purpose. `install` and `uninstall` remove every `*.wants/lemusyncd.service` link next to the unit; `uninstall` never changes linger. Supersedes the SP2 choice of `default.target` ([desktop §Linux results](specs/platforms/desktop.md#linux-results-sp2-xfce)), made on XFCE without linger. Why: a lingering `default.target` unit runs before the desktop, which on 2026-10-29 let the daemon take the session's secret store (D38). Cost: XFCE never reaches `graphical-session.target`, so the unit doesn't start by itself there yet (V-DESK-6). Rejected: keeping `default.target` and relying on D38 alone (the daemon would still run, unseen, before login on every lingering desktop); an XDG autostart entry for every desktop (a second start path to keep in step, P1).
