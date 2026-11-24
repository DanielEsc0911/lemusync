# LemuSync documentation index

Read this file, then only the docs your task needs. Headings are stable: grep the outline (`grep -n "^#" <file>`) and read just that section. Status values: **Draft** (direction set, details open) · **Decided** (agreed, not built) · **Partial** (partly built) · **Implemented** (matches the code).

## Foundations
| Doc | Covers | Status |
|---|---|---|
| [rules.md](rules.md) | Non-negotiable rules with IDs, rationale, enforcement | Decided |
| [decisions.md](decisions.md) | Decision log. Read it before proposing an alternative | Decided |
| [glossary.md](glossary.md) | Every project term, code name ↔ UI name | Decided |

## Specs
### Product and architecture
| Doc | Covers | Status |
|---|---|---|
| [specs/product.md](specs/product.md) | Vision, users, core scenarios, scope, non-goals, market gap | Decided |
| [specs/architecture.md](specs/architecture.md) | Components, crates and layers, process model, engine API, repo tree | Decided |

### Sync
| Doc | Covers | Status |
|---|---|---|
| [specs/sync-model.md](specs/sync-model.md) | Streams, versions, heads, conflicts, leases, play sessions, remote layout, retention | Draft |
| [specs/players-and-accounts.md](specs/players-and-accounts.md) | Players, devices, emulator account mapping, the same-ID trap, swapping | Draft |
| [specs/emulator-profiles.md](specs/emulator-profiles.md) | Profile schema, layouts, tiers, discovery, overrides, initial set | Draft |
| [specs/storage-backends.md](specs/storage-backends.md) | Backend interface, capabilities, adapters, provider quirks, credentials | Draft |

### Quality
| Doc | Covers | Status |
|---|---|---|
| [specs/security.md](specs/security.md) | Threat model, crypto, keys, IPC, file confinement, supply chain | Draft |
| [specs/reliability.md](specs/reliability.md) | Invariants, atomic writes, journal, local history, 24/7, sleep, budgets | Draft |
| [specs/testing.md](specs/testing.md) | Test layers, simulation, goldens, performance, conformance, device lab | Draft |

### Interfaces
| Doc | Covers | Status |
|---|---|---|
| [specs/config-and-cli.md](specs/config-and-cli.md) | Config files and locations, CLI commands, JSON output, exit codes | Draft |
| [specs/i18n.md](specs/i18n.md) | English/Spanish, string sources per layer, terminology, parity checks | Decided |
| [specs/ui/brand.md](specs/ui/brand.md) | Direction, palette roles, lemur mark, the generator pipeline and its outputs | Decided |
| [specs/ui/design-system.md](specs/ui/design-system.md) | Visual parity, tokens, components, motion, accessibility, frame budgets | Draft |
| [specs/ui/app-shell.md](specs/ui/app-shell.md) | How the desktop app reaches the daemon: connection states, data flow, app state, destinations as built | Decided |
| [specs/ui/navigation.md](specs/ui/navigation.md) | Destinations, adaptive containers, every user flow, deep links | Draft |
| [specs/ui/status-and-notifications.md](specs/ui/status-and-notifications.md) | One status model → tray, Live Updates, Live Activities, toasts | Draft |

### Platforms
| Doc | Covers | Status |
|---|---|---|
| [specs/platforms/desktop.md](specs/platforms/desktop.md) | Windows, macOS, Linux: services, tray, detection, paths, packaging | Draft |
| [specs/platforms/android.md](specs/platforms/android.md) | Storage access, background limits, Live Updates, OEM quirks | Draft |
| [specs/platforms/ios.md](specs/platforms/ios.md) | Files access, background tasks, Live Activities, App Intents | Draft |
| [specs/platforms/headless.md](specs/platforms/headless.md) | Raspberry Pi, servers, Linux handhelds | Draft |

### Delivery
| Doc | Covers | Status |
|---|---|---|
| [specs/release.md](specs/release.md) | Versioning, CI, artifacts (`.apk` `.msi` `.exe` `.dmg` `.deb` `.rpm` `.AppImage` `.tar.gz`), install layout, signing, channels | Draft |
| [specs/git-workflow.md](specs/git-workflow.md) | Branches, commit format, co-authorship, PR checklist | Decided |

## Progress
| Doc | Covers |
|---|---|
| [progress/README.md](progress/README.md) | How plans are tracked and cleaned up; status board |
| [progress/roadmap.md](progress/roadmap.md) | Phases, milestones, exit criteria |

## Reading paths
| Task | Read, in order |
|---|---|
| Sync logic, versions, conflicts, locks | sync-model → reliability → security §Cryptography |
| Add or fix an emulator | emulator-profiles → players-and-accounts → the platform spec |
| Add a storage provider | storage-backends → security §Credentials |
| UI screen or component | ui/design-system → ui/brand (tokens, mark) → ui/navigation → i18n |
| Tray, notifications, live surfaces | ui/status-and-notifications → the platform spec |
| Background or service behaviour | reliability → the platform spec |
| CLI or config | config-and-cli → i18n §Rust strings |
| CI, packaging, release | release → security §Supply chain → git-workflow |
