# Changelog

All notable changes to this project. Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Versions follow [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Fixed
- On Linux, LemuSync could start GNOME Keyring and take the desktop's secret store from KWallet when linger started the daemon before login. The keystore client now never starts, unlocks or creates a secret store: it waits, with backoff, until the session's own store (whatever provides it) is there ([D38](docs/decisions.md#d38-the-linux-keystore-client-never-starts-unlocks-or-creates-a-store-2026-10-29)).
- `lemusync service install` binds the daemon to the graphical session (`graphical-session.target`), so it no longer starts at boot under linger; `install` and `uninstall` also remove links an older unit left behind ([D39](docs/decisions.md#d39-the-desktop-unit-starts-with-the-graphical-session-2026-10-29)).
- Tests no longer touch the real keystore: `lemusync doctor` under `LEMUSYNC_TEST_ROOT` wrote a probe item to it.

### Changed
- **Breaking:** every command that takes a game accepts an alias, the game key or its name typed loosely (case, punctuation, accents and region tags like `(USA)` ignored): `play <game> [--emu <id>] [--as <player>]`, `history <game> [--emu] [--as]`, `restore <game> <what> [--emu] [--as] [--yes]`, `resolve <game> <version>`, `lease release --force <game>`. `history` and `restore` default to the only player. The old `play <emulator> <game>` and `<player> <emulator> <game>` forms are gone.
- The project is renamed from LeemuSync to LemuSync (lemur + emu + sync). Binaries, app id, env vars, local folders and the remote storage prefix change with no migration ([D33](docs/decisions.md#d33-the-project-is-named-lemusync-2026-10-27)).

### Added
- Game aliases: `lemusync games alias <game> <alias>` / `games unalias <alias>` or `config set aliases.<alias> <emulator>/<key>`, so `lemusync play botw` works; `games` shows them.
- `lemusync games` lists the games installed on this device with their names; `play` finds a game's path by itself (Cemu's own game folders, or `emulators.<id>.game_dirs`), and `status`, `history`, `conflicts` and `play` show game names next to their ids.
- `lemusync config path|list|get|set|unset` reads and changes `config.toml` from the CLI, keeping comments; managed keys (`repo`, `backend`, `grants`) point to the command that owns them.
- While storage requests are retried, the CLI says "Connection trouble, retrying…" and then "Connection is back." (en/es); the engine sends the same `ConnectionEvent` for the daemon and tray.
- Storage errors are friendly and localized (`backend_offline`, `backend_quota`, `backend_auth`, `backend`), with the redacted provider text on a `Details:` line and in `--json` `error.detail`.
- Synthetic network test suite for SC5: short and long outages and a slow link on upload and download, on top of the cut and lost-response matrices.
- First sync between two PCs through Cloudflare R2 (CLI): both players' Cemu saves on both PCs, with live progress while syncing.
- Project rules (`AGENTS.md`, `docs/rules.md`), documentation set, roadmap and Phase 0 plan.
