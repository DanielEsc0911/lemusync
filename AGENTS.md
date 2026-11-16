# LemuSync — agent guide

## What this is
LemuSync (lemur + emu + sync) keeps emulator saves in sync across every device: Windows, macOS, Linux, Raspberry Pi, Android and iOS. Saves go through storage the user owns (S3, WebDAV/Nextcloud, SFTP, Google Drive…). Two players can play the same game on different machines at the same time. One player can switch machines mid-game without losing progress. Open source (MPL-2.0). Rust engine, Flutter UI. Status: pre-alpha, Phase 0 (see `docs/progress/`).

## Non-negotiable rules
Cite rules by ID. Full text, rationale and how each is enforced: `docs/rules.md`.

**Frontend (unbreakable)**
- **F1 Same look everywhere.** One Flutter design system with bundled fonts. Only the layout adapts to screen size.
- **F2 No jank, nothing broken.** 60 fps minimum, 120 fps where supported. No half-wired feature ships.
- **F3 Same navigation everywhere.** Same destinations, order and flow steps on every platform.

**Backend (unbreakable)**
- **B1 Scalable architecture.** Layered crates, dependencies point down only. One responsibility per file. Formats are versioned.
- **B2 Security first.** E2E encryption. No inbound ports. Profiles and remote data are data, never code. Writes stay inside user-granted folders.
- **B3 Reliable 24/7.** Never lose or corrupt a save. Survive sleep, crashes and bad networks. Stay light. Updates never break a working install or config.

**Product**
- **P1** KISS and YAGNI.
- **P2** The user owns their storage. No LemuSync server, no accounts.
- **P3** Any emulator, any folder layout. Every emulator gets safety and features scale by tier. No per-game save parsing in core.
- **P4** Every platform at its best: tray/menu bar, Live Updates, Live Activities, Dynamic Island, Shortcuts.
- **P5** Simple by default, powerful underneath. Everything in the UI is also in the CLI and the config file.
- **P6** English and Spanish (neutral Latin American) for every user-facing string.
- **P7** Open source, MPL-2.0.

**Workflow**
- **W1** Docs are code. Update the spec in the same commit as the change.
- **W2** Test first. `just check` must be green before every commit.
- **W3** No unverified facts. Unproven claims go under "Verify" with a spike ID.
- **W4** Decisions are logged in `docs/decisions.md`. Don't reopen one without new facts.
- **W5** Conventional Commits. The human is the author and the AI agent adds a `Co-Authored-By:` trailer. Never add session URLs or session references to commits or PRs.

## Where everything is
| Path | What |
|---|---|
| `docs/index.md` | **Start here.** Map of every doc, plus reading paths by task |
| `docs/rules.md` · `docs/decisions.md` · `docs/glossary.md` | Rules in full · why things are the way they are · terms |
| `docs/specs/` | One spec per feature or aspect, kept current with the code |
| `docs/progress/` | Roadmap, active plans, status board |
| `.claude/skills/lemusync-*` | Project skills (plain Markdown, any agent can follow them): docs, feature, ui, emulator-profile, backend, security |
| `crates/` | Rust workspace: core, crypto, store, api, engine, ipc, daemon, cli, bridge |
| `app/` | Flutter app + native platform code (Kotlin/Swift) |
| `profiles/` | Emulator profiles (TOML data) |
| `xtask/` | Repo checks (`cargo xtask …`) |

## Commands
These exist once Phase 0 Tasks 1–4 land (`docs/progress/plans/phase-0-foundations.md`).

| Command | Does |
|---|---|
| `just check` | Everything CI runs. Must pass before every commit |
| `just fmt` | Format Rust and Dart |
| `just test` | Rust and Flutter tests |
| `cargo xtask docs-check` | Doc links, index coverage, spec headers |
| `cargo xtask layers` | Crate dependency direction (B1) |
| `cargo xtask i18n-check` | en/es string parity (P6) |
