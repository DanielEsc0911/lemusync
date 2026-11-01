# Config and CLI

> **Status:** Draft · **Related:** [rules](../rules.md) (P5, P6, B2), [architecture](architecture.md), [emulator-profiles](emulator-profiles.md#overrides), [i18n](i18n.md#rust-strings), [platforms/headless](platforms/headless.md) · **Code:** `crates/engine/src/local_config.rs`, `crates/cli/`

## Purpose
Power users and headless devices can do everything without the GUI (P5). The UI, CLI and config file are three views of the same settings.

## Locations
Resolved with the `directories` crate:

| OS | Config | Data (state DB, history, staging) |
|---|---|---|
| Linux | `~/.config/lemusync/` | `~/.local/share/lemusync/` |
| macOS | `~/Library/Application Support/LemuSync/` | same |
| Windows | `%APPDATA%\LemuSync\` | `%LOCALAPPDATA%\LemuSync\` |
| Android / iOS | App-private storage | App-private storage |

## Config file
- `config.toml`, versioned (`config_version = 1`), per device: device id and name, repo id, backend reference (`kind = "gdrive"` with `root` and `client_id`, or `kind = "fs"` with `path`), grants, per-emulator settings `[emulators.<id>]` (`root` override; `exe`, the argv prefix that starts the emulator, e.g. `["/usr/bin/cemu"]` or `["flatpak", "run", "info.cemu.Cemu"]`, never run through a shell; `games."<title id>" = "<absolute path>"`, the file passed as `{game_path}` to the profile's [`launch.args`](emulator-profiles.md#schema-v1), which overrides the detected one; `game_dirs = ["<absolute folder>", …]`, extra folders scanned for installed games with the profile's rule, read only and never created ([Game discovery](emulator-profiles.md#game-discovery))), `[leases] ttl_secs` (lease lifetime, default 300, 120–86400; [sync-model §Leases](sync-model.md#leases)), `conflict_policy = "ask" | "newest"` (default `ask`; [sync-model §Conflicts](sync-model.md#conflicts)), `[aliases]` (`<alias> = "<emulator>/<game key>"`, short names for games on this device; [Game references](#commands)). Later phases add schedule, UI language and notification preferences.
- Shared settings (players, mappings, cross-device game key aliases) live in [repo config](sync-model.md#repo-config), not in this file. The short game aliases typed on the command line (`[aliases]`) are per device (P1).
- The engine validates on load and hot-reloads on change. If the file is invalid, the engine keeps the last good config and reports the error (status `Error`, CLI exit code 3).
- Secrets are **never** stored here ([security §Credentials](security.md#credentials)).
- The UI writes the same file through the engine API, so hand edits and UI edits never fight.
- Every engine write (`config set`, `grant add`, `init`, `backend add`…) merges the new values into the file on disk, so the user's comments and key order survive (`LocalConfig::save`, `toml_edit`).

## Commands
Every command supports `--json` (stable English keys, versioned) and `--help`. Human-readable output is localised.

| Command | Does |
|---|---|
| `lemusync init` | Create a repo on a backend; set passphrase; show recovery key. Once the repo exists the key is always shown: if a later step fails on this device, `init` warns and `lemusync join` finishes it |
| `lemusync join` | Join an existing repo (passphrase or recovery key) |
| `lemusync status` | Engine state, pending transfers, conflicts, leases. When the daemon answers, the report (and `--json`) adds `daemon`: its version, its status model (`Status`) and the play session overlay |
| `lemusync pause [--for <duration>]` / `resume` | Pause sync (`1h`, `30m`, `90s`; none = until `resume`) and end the pause, as the tray does. Needs a running daemon: without one it's the usage error `daemon_not_running`. Code: `crates/cli/src/pause.rs` |
| `lemusync sync [--dry-run]` | Sync now |
| `lemusync play <game> [--emu <id>] [--as <player>] [-- <args>]` | Launcher mode. `<game>` is an alias, key or name ([Game references](#commands)), e.g. `lemusync play botw --as daniel` |
| `lemusync games [--json]` | Installed games on this device: emulator, key, name (the key when none is known), path, where it was found (`config`, `emulator`, `scan`) and its aliases (`aliases`, absent when none). Local and offline, no storage access ([Game discovery](emulator-profiles.md#game-discovery)). Code: `crates/cli/src/games_cmd.rs` |
| `lemusync games alias <game> <alias> [--emu <id>]` / `games unalias <alias>` | Give a game a short name, or remove it ([Game references](#commands)) |
| `lemusync history <game> [--emu <id>] [--as <player>]` / `restore <game> <version> [--emu <id>] [--as <player>] [--yes]` | Browse and restore |
| `lemusync conflicts` / `resolve <game> <version> [--as <player>]` | Conflict handling |
| `lemusync lease status` / `lease release --force <game>` | Lease handling |
| `lemusync players add/list` / `accounts map <player> <emulator> <id> [--keep\|--replace]` | Players and account mappings |
| `lemusync accounts recover <emulator> <id> --player <name>` | Upload an orphaned save folder as a player's first version |
| `lemusync accounts create <player> <emulator> [--name <name>] [--yes]` | Write a new emulator account from profile data, map it to the player, then sync to restore their saves ([players-and-accounts](players-and-accounts.md#creating-a-cemu-account-sp8)) |
| `lemusync scan` | Accounts, save folders, orphaned saves, grants needed |
| `lemusync devices` | Devices in the repo |
| `lemusync backend add/test/show` | Storage setup and probe |
| `lemusync backend add s3 --endpoint <url> --bucket <name> --access-key-id <id> [--region auto]` / `backend add gdrive --client-id <id>` / `backend add fs <path>` / `backend login` | S3-compatible (R2), Google Drive or local-folder setup, with a probe for S3; OAuth login |
| `lemusync grant add/list/revoke` | Folder permissions |
| `lemusync profiles list/show/validate <file>` | Emulator profiles |
| `lemusync service install [--headless]/uninstall/start/stop/status` | Daemon service management. Linux: a systemd user unit at `~/.config/systemd/user/lemusyncd.service` ([reliability §Always on](reliability.md#always-on-247)); the desktop unit is `WantedBy=graphical-session.target` with `After=`/`PartOf=` it; `--headless` uses `default.target` and also runs `loginctl enable-linger`; `install` and `uninstall` remove every `*.wants/lemusyncd.service` link left next to the unit; `uninstall` leaves linger as it is and says how to turn it off; `status --json` gives `installed`, `enabled`, `active`, `restarts`. Other OSes: usage error `service_unsupported` for now. Code: `crates/cli/src/service.rs` |
| `lemusync logs [--follow]` | Local, redacted logs. `lemusyncd` writes `<data>/logs/lemusyncd.log` (`tracing`, level info), every line through `redact`; it rotates at 5 MiB and keeps 3 old files (`.1` newest). `logs` prints the file (control characters other than newline and tab dropped); `--follow` keeps printing new lines and starts over when the file rotates. It reads the file, so it needs no daemon. Code: `crates/daemon/src/logging.rs`, `crates/cli/src/logs_cmd.rs` |
| `lemusync doctor` | Diagnose permissions, paths, keystore, battery/background settings, clock skew. The CLI adds desktop checks (`crates/cli/src/doctor_desktop.rs`), the same on both paths: `daemon` (the socket answers; installed but silent fails), `service` (unit enabled), `linger` (needed only headless), `tray host` (a StatusNotifierWatcher on the session bus, else the GNOME AppIndicator hint) and `notifications` (`GetServerInformation`) |

The Phase 1 first slice ships `init`, `join`, `backend add/login`, `grant add/list`, `players add/list`, `scan`, `accounts map/recover`, `sync [--dry-run] [--yes]`, `status`, `history`, `restore` and `doctor` ([plan](../progress/plans/phase-1-first-sync.md)). M1b adds `play`, `conflicts`, `resolve <game> <version> [--as <player>]`, `lease status`, `lease release --force <game>` and `accounts create` ([plan](../progress/plans/phase-1b-play.md)). Every `<game>` argument goes through the [game references](#commands) resolver, and `--emu <id>` narrows it to one emulator. `--as` defaults to the only player in the repo, for `history` and `restore` too; with several players it's the "which player?" usage error (`player_needed`). The older `play <emulator> <game>` and `history`/`restore <player> <emulator> <game>` shapes were removed before alpha, with no compatibility form; `play <emulator> <game>` fails with a usage error (`play_old_shape`) that shows `lemusync play <game> --emu <emulator>`. `config path|list|get|set|unset` reads and changes this file (below). Phase 2 adds `service` on Linux, `pause`, `resume` and `logs` ([plan](../progress/plans/phase-2-daemon.md)).

**Through the daemon (D35).** Every command but `service`, `logs` and the Google Drive login first connects to the daemon's socket. When it answers, the command runs there and prints the same text and the same JSON as in-process; when nothing listens (or there is no runtime dir, or the OS has no IPC yet) the CLI runs the engine itself as before. A daemon with another `api_version` is an error, never a silent in-process run. Prompts are asked on the CLI side first; `play` streams its steps from the daemon and answers the lease and conflict questions the daemon sends (one pending question per connection; without a terminal or with `--json` the answer is the refusal, so exit 5 and 4 stay as they are). `backend add gdrive` and `backend login` refuse while the daemon runs (`daemon_running_oauth`): the OAuth loopback listener stays in the CLI process ([D28](../decisions.md#d28-oauth-loopback-listener-during-login-only-2026-10-18)). Code: `crates/cli/src/daemon_client.rs`.

**`config`.** Keys are dotted paths. `list` shows every value (defaults included) and `get` one key or a whole table as one object (`backend`, `leases`, `emulators.<id>`); both take `--json` (`{"api_version", "values": {key: value}}` and `{"api_version", "key", "value"}`; an unset key is `null`). `path` prints the file's path. `set`/`unset` change only the settable keys below. The edit keeps the user's comments and key order (`toml_edit`), is parsed back into the config and refused if invalid, and is written atomically under the engine's state lock. A bad value, an unknown key or emulator, or a managed key is a usage error (exit 2) and leaves the file byte-identical. `unset` returns a key to its default and removes tables it leaves empty. Code: `crates/engine/src/config_keys.rs`, `crates/cli/src/config_cmd.rs`.

| Key | Type | Owner |
|---|---|---|
| `conflict_policy` | `ask` \| `newest` | `config set` |
| `leases.ttl_secs` | integer, 120–86400 | `config set` |
| `emulators.<id>.root` | absolute path, no `..` (`scan` still needs it inside a grant, B2) | `config set` |
| `emulators.<id>.exe` | list of words (`set` takes several) | `config set` |
| `emulators.<id>.game_dirs` | list of absolute paths, no `..` (`set` takes several) | `config set` |
| `emulators.<id>.games.<game>` | absolute path, no `..`; the game key is lowercased, and an alias of one of `<id>`'s games stands for its key | `config set` |
| `aliases.<alias>` | `<emulator>/<game key>`: a bundled emulator and a key in the game-key alphabet, installed or not; the alias matches `^[a-z0-9][a-z0-9_-]{0,31}$`, is unique and isn't another game's key | `config set`, `games alias` |
| `repo` | repo id | `init` / `join` |
| `backend.*` | backend reference | `backend add` |
| `grants` | list of folders | `grant add` (B2: grants change only there) |
| `config_version`, `device.*` | fixed | none |

`<id>` must be a loaded profile id.

**`play` in the terminal.** One line on stderr per step (getting the newest save, waiting for another device, the emulator running, uploading), then the emulator's exit code and the upload result on stdout. Lease renewals are silent unless one fails. On Unix the emulator runs in its own process group: Ctrl-C reaches only LemuSync, which prints a hint and keeps waiting, then uploads and releases the lease when the emulator closes. A held lease or a conflict is asked about only with a terminal and without `--json`; otherwise `play` stops before launch with exit 5 or 4.

**Game references.** Any command that takes a game accepts an alias, the game key, or the game's name typed loosely. Tiers, tried in order; the first with any match wins: (1) exact alias, (2) key, case-insensitive, (3) name after normalization. Normalization folds accented Latin letters to ASCII (`é→e`, `ñ→n`, `ü→u`, `ß→ss`, `œ→oe`), lowercases, drops region and revision tags in `(…)` or `[…]` (`(USA)`, `[v1.2]`), and turns every run of other characters into one space. There is no fuzzy or substring tier (P1): `zelda breath` finds nothing. An alias equal to another game's key (only possible by hand-editing the file) is ignored for that query, so a key always means its own game. Letters outside the fold table (Greek, Cyrillic, CJK, emoji) are dropped like punctuation, so a name made only of them, or only of tags, never matches by name: use its key or an alias. More than one match in the winning tier is ambiguous: the error lists every candidate (name, emulator, key), so region variants are visible, and suggests `--emu` or an alias. Candidates are this device's installed games, the games its aliases point at, and the games the repo has synced (no name). Code: `crates/core/src/game_ref.rs`, `crates/engine/src/game_refs.rs`.

Aliases are short names, local to this device: `config set aliases.<alias> <emulator>/<key>` is the source of truth. `lemusync games alias <game> <alias> [--emu <id>]` resolves `<game>` with the tiers above (repo games included) and stores the resolved `<emulator>/<key>`, never the text typed; it refuses an ambiguous game. A game no device has installed or synced yet is given as `<emulator>/<key>` or `--emu <id> <key>`. Both ways accept the same targets: a bundled emulator and a key in the game-key alphabet, and an alias that isn't another known game's key. `games unalias <alias>` removes one. `emulators.<id>.games.<alias>` stores the alias's key, so removing or retargeting the alias later doesn't change that entry. A hand-edited invalid alias makes the config invalid (exit 3) with an error naming `aliases.<alias>`. **Open question:** sharing aliases through the repo config, if people ask for it.

**Game names.** `status`, `history`, `conflicts` and `play` show a game as "Name (key)" when this device's catalog knows its name, else the bare key; `play` starts with "Starting <name>…". Names come from the game's metadata on this device, so a game found only by detection (no `games.<key>` entry) is named too, and a device without the game shows the key ([D34](../decisions.md#d34-game-names-are-found-on-each-device-never-stored-in-the-repo-2026-10-27)). `--json` carries the same: `status` streams, `conflicts` entries and `history` have an optional `name`, absent when unknown (P5); `games --json` lists every game. Code: `Engine::game_names` (`crates/engine/src/games.rs`), test `crates/engine/tests/games.rs`.

**Progress and results.** While `sync`, `restore` and `accounts recover` run, one line on stderr shows the stream being worked on: a spinner, player · game, the phase, a bar, files, MB and MB/s. It's redrawn every 100 ms and cleared before anything else prints. It only appears when stderr is a terminal, `TERM` isn't `dumb` and `--json` is off. `sync` then prints one aligned line per stream (with "N files, X MB" when something moved) and the elapsed time. The action is colored (green done, yellow waiting on the user, red conflict) only when stdout is a terminal and `NO_COLOR` is unset.

**Connection notice.** While a storage request is retried (reliability §Network), the engine sends `ConnectionEvent`s (architecture §Engine API). Any command that talks to the backend shows them on stderr: nothing for one quick retry; from the second retry of an outage on, one line, en "Connection trouble, retrying…" / es "Problemas de conexión, reintentando…"; then "Connection is back." / "La conexión volvió." once a request succeeds. At most those two lines per outage; the live progress line is cleared first. Nothing with `--json`, which doesn't stream events. Code: `crates/cli/src/connection.rs`.

Secrets (passphrase, recovery key, OAuth client secret, S3 secret key) are only typed at no-echo prompts, never taken from arguments or environment variables; without a terminal the command fails with a usage error. The recovery key is printed once by `init`, to stderr, even with `--json`. `LEMUSYNC_TEST_ROOT=<dir>` puts the config, data and emulator search under `<dir>`: it's for tests only (the OS keystore is still used). With it, `service` writes the unit under `<dir>/config/systemd/user/` and only records the `systemctl`/`loginctl` argv in `<dir>/systemctl.log`, and `LEMUSYNC_TEST_DAEMON` names the `lemusyncd` path.

## Exit codes
| Code | Meaning |
|---|---|
| 0 | Success |
| 1 | Error |
| 2 | Usage error |
| 3 | Invalid config |
| 4 | Conflict needs a decision (`sync` found one; `play` without a terminal or with `--json`) |
| 5 | Locked by another device (`play` found a held lease without a terminal or with `--json`; never a silent "play anyway") |
| 6 | Backend unreachable or auth failed |

## Error codes
Every error has a stable `code` (the `--json` `error.code`, and the key of its localized message `error-<code>` in `locales/`). Storage errors map by class ([storage-backends §Interface](storage-backends.md#interface)):

| Class | Code | Exit | Message says |
|---|---|---|---|
| `transient` (after the store's retries) | `backend_offline` | 6 | Can't reach the storage; check the connection and try again; saves on this device are safe |
| `auth` | `backend_auth` | 6 | The login failed; run `lemusync backend login` |
| `quota` | `backend_quota` | 1 | The storage is full; free up space; saves on this device are safe |
| `not_found`, `already_exists`, `permanent` | `backend` | 1 | The storage returned an error; try again, then `lemusync doctor` |

The keystore has two codes ([security §Secret Service activation](security.md#secret-service-activation)): `secrets_unavailable` (exit 1) means it isn't there yet (`no_owner`, `no_bus`, `locked`, `gone`). The daemon retries it with backoff and shows no error. `secrets` (exit 1) is a real failure (`no_default_collection`, `prompt_needed`, `ambiguous`, `bad_reply`, `bad_master_key`, `off_under_test_root`). Each key has its own en/es message.

"Saves on this device are safe" holds because a sync or restore downloads and verifies everything into staging before it touches a save folder, and an upload only reads local files ([reliability §Restore procedure](reliability.md#restore-procedure)). The message never contains provider text. That goes on a second, dim line, `Details: …` / `Detalles: …`, and in `--json` as `error.detail` next to `code` and `message` (the English message for storage errors). It always passes through `crates/store/src/redact.rs` first (B2: `StoreError` redacts its text when it is built, so no path sees it raw), with control characters replaced and at most 500 characters. `redact` removes tokens, keys and signatures; it keeps endpoints and bucket names on purpose, because the line is shown only to this device's own user, who configured them and needs them to fix the problem. Other errors have no `detail`. Code: `crates/engine/src/error.rs`, `crates/cli/src/output.rs`.

## Parity rule
Any feature added to the UI gets its CLI command (desktop/headless) in the same plan, and the other way round. Mobile has no CLI. Its Settings → Advanced exposes the same options, mapped to the `config` keys above.
