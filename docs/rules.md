# Rules

> **Status:** Decided · **Related:** [AGENTS.md](../AGENTS.md), [decisions](decisions.md), [security](specs/security.md), [reliability](specs/reliability.md), [design-system](specs/ui/design-system.md) · **Code:** —

## How to use
- Rules win over convenience, deadlines and every other doc. If a spec conflicts with a rule, the spec has a bug.
- Cite rules by ID in reviews, commits and plans ("violates B2").
- A rule changes only with the maintainer's approval, recorded in [decisions.md](decisions.md).
- If a request would break a rule, say so, cite the ID, and propose a compliant alternative.

## Frontend: unbreakable
### F1 Same look on every platform
- **What:** One Flutter design system renders the same look on Windows, macOS, Linux, Android and iOS: same colors, type (bundled fonts), icons, components and motion.
- **May differ:** layout by screen-size class; platform input behaviours (scroll physics, back gesture, text-selection handles, keyboard shortcuts, window chrome, haptics); OS-owned surfaces (tray menus, notifications, Live Updates, Live Activities). OS-owned surfaces follow the platform's rules but use our icons, wording and status model.
- **Enforced by:** golden tests per size class × theme × locale ([testing](specs/testing.md)), review against [design-system](specs/ui/design-system.md).

### F2 Best possible performance; nothing broken
- **What:** 60 fps minimum, 120 fps on high-refresh displays, within the frame budgets in [design-system §Performance](specs/ui/design-system.md#performance). A feature ships only when it works end to end on every platform where it appears: no dead buttons, no "coming soon" in releases.
- **Enforced by:** profile-mode performance tests, the release checklist ([release](specs/release.md)).

### F3 Same navigation on every platform
- **What:** Same destinations, order, names and flow steps everywhere. Screen size changes the container (bottom bar / rail / sidebar, pushed page / side pane), never the flow.
- **Enforced by:** [navigation](specs/ui/navigation.md) is the source of truth; integration tests cover each flow per size class.

## Backend: unbreakable
### B1 Architecture that scales without breaking
- Layered crates whose dependencies only point down ([architecture §Layers](specs/architecture.md#layers), `cargo xtask layers`).
- One responsibility per file. Feature-first folders in Dart. No god modules.
- Every persisted or wire format (repo, manifest, profile schema, IPC, config) carries a version and has tested migrations. New features extend stored data and never break it.
- Breaking a public interface requires a decision entry.

### B2 Security first: no backdoors, no exploits
- End-to-end encryption is mandatory. Keys never leave devices unencrypted. No telemetry. No project-run server in the data path.
- No inbound network listeners. Local IPC is same-user only.
- Profiles, manifests, remote objects, config files and deep links are untrusted data, never code: no shell, no eval, no plugin loading, no remote profile downloads.
- File writes stay inside user-granted roots, and every path is validated.
- `unsafe` is forbidden outside the FFI bridge. Dependencies are minimal and audited. Releases are signed.
- Full model: [security](specs/security.md).

### B3 Reliability and performance, 24/7
- Never lose or corrupt a save: invariants I1–I6 in [reliability](specs/reliability.md#invariants).
- Always on wherever the OS allows: service managers, restart on crash, sleep inhibition during transfers, rescans on resume. Mobile uses every allowed background mechanism and documents its limits honestly.
- Light: the daemon stays within [reliability §Budgets](specs/reliability.md#budgets).
- Updates never break a working install. After an update, every existing config file, state database, stored key and credential, service unit, granted folder and repo works as before, and the user does nothing. Formats change by adding optional fields or by an automatic, tested migration that keeps the old copy until the new one is verified. Renames of the app, paths, keystore entries or crypto context strings come with that migration. If an update meets data it can't read, it stops with a clear message and changes nothing. See [D42](decisions.md#d42-updates-never-break-a-working-install-2026-11-01).

## Product
- **P1 KISS.** Pick the simplest design that satisfies every rule. No speculative features. A new dependency or component needs a written reason.
- **P2 Your storage, your data.** LemuSync works with storage the user owns, runs no server and has no accounts. Only the user's devices can read the data.
- **P3 Any emulator, any layout.** Every emulator gets the safety features, and capabilities scale by tier ([emulator-profiles §Tiers](specs/emulator-profiles.md#tiers)). Folder layouts are user-overridable. Profiles are declarative data. Core never parses game save formats.
- **P4 Every platform, at its best.** Windows, macOS, Linux (x86_64, arm64), Raspberry Pi, Android and iOS, each using its best integration: tray/menu bar, Live Updates (Now Bar, Super Island), Live Activities and Dynamic Island, App Intents/Shortcuts, Quick Settings tiles.
- **P5 Simple by default, powerful underneath.** Defaults work with no configuration. Every setting can be reached in the UI (Advanced), the CLI and the config file. The CLI offers `--json`.
- **P6 English and Spanish.** Every user-facing string (app, CLI, tray, notifications, native extensions, store listings) exists in `en` and neutral Latin American `es`. No hard-coded strings. See [i18n](specs/i18n.md).
- **P7 Open source.** MPL-2.0, auditable and reproducible.

## Workflow
- **W1 Docs are code.** A behaviour change updates its spec in the same commit. A new spec goes into the [index](index.md). Specs link to each other in both directions.
- **W2 Test first.** Follow TDD. `just check` is green before every commit. Never claim work is done without running it.
- **W3 No unverified facts.** Vendor or platform behaviour that this repo hasn't proven goes under **Verify** with the spike that will prove it.
- **W4 Decisions are logged.** Record them in [decisions.md](decisions.md). Don't reopen one without new facts. A change gets a superseding entry.
- **W5 Commits.** Use Conventional Commits. The human maintainer is the author. AI agents add a `Co-Authored-By:` trailer and nothing else: no session URLs or session references in commits or PRs ([git-workflow](specs/git-workflow.md)).
