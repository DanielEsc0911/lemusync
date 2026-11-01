# Roadmap

> **Status:** Decided · **Related:** [product](../specs/product.md) (scenarios SC1–SC7), [progress](README.md), [architecture](../specs/architecture.md), [rules](../rules.md) · **Code:** —

Each phase ends with a demo of its exit criteria on real devices. Scenario IDs (SC1–SC7) are defined in [product](../specs/product.md#core-scenarios). Spike IDs: SP1–SP7 are defined in the [Phase 0 plan](plans/phase-0-foundations.md), SP8 in [phase-1-first-sync](plans/phase-1-first-sync.md), SP9 in [phase-1b-play](plans/phase-1b-play.md) (results under V-SYNC-1 and V-PLAY-1 in [sync-model](../specs/sync-model.md#verify)), SP10 in [emulator-profiles §Game discovery](../specs/emulator-profiles.md#game-discovery).

## Phase 0: Foundations and spikes
- **Goal:** a CI-checked skeleton, plus answers to every platform risk before any feature code is written.
- **Deliverables:** Rust workspace + xtask checks; Flutter app shell (en/es); CI; release pipeline producing `.apk` `.msi` `.exe` `.dmg` `.deb` `.rpm` `.AppImage` `.tar.gz`; spikes SP1–SP7 recorded in the specs; Phase 1 plan.
- **Exit:** `just check` green on Linux, Windows and macOS CI. Every Verify item targeted by a spike is resolved or re-scoped.

## Phase 1: Sync engine + CLI (desktop)
- **Goal:** the whole sync model working headless.
- **Deliverables:** `core` (model, heads, reconciliation, path validation, profile schema), `crypto`, `store` (fs, S3, WebDAV, SFTP + conformance), `engine` (scan, snapshot, transfer, restore, leases, journal, local history, retention), the five initial profiles, `cli` with `--json`, the simulation test harness.
- **Exit (milestone M1, "BotW handoff"):** the project's first end-to-end test. It runs on the maintainer's two Linux machines, the Fedora PC and the CachyOS laptop, both with Cemu ([testing §Device lab](../specs/testing.md#device-lab)). It's reached in two steps ([D29](../decisions.md#d29-m1-is-split-first-both-saves-on-both-pcs-then-leases-and-play-2026-10-18)): **M1a**, both players' saves on both PCs ([plan](plans/phase-1-first-sync.md)), then **M1b**, the rest below ([plan](plans/phase-1b-play.md)). Two players play BotW on Cemu on those two PCs at the same time (SC1); one player hands off between PCs (SC2); a forced double-open becomes a conflict that's resolved with nothing lost (SC3); `kill -9` and network cuts at every journal step leave no corruption (SC5). Daemon budgets measured. Setup (maintainer, 2026-10-18): each machine has one Cemu account (`80000001` on both) holding a different player's BotW save. The first goal is to have both saves on both machines, which is the [same-ID trap](../specs/players-and-accounts.md#the-same-id-trap) case.
- **M1a reached on 2026-10-20** through Cloudflare R2 ([D31](../decisions.md#d31-cloudflare-r2-replaces-google-drive-for-the-first-real-test-2026-10-20)), CLI only. Both players' BotW saves are on both PCs, and the maintainer handed off back and forth between PCs and accounts. Surprises: R2 returned one `500 InternalError` on the first push (fixed by retries, Task 17c); each sync re-sent the whole save and printed nothing for ~50 s (fixed by Task 17d); Cemu gave the new account a reused id on Fedora (`80000002`) but a fresh one on CachyOS (`80000003`).
- **M1b reached on 2026-10-21** on both PCs with `lemusync play` through R2: SC1, SC2 and SC3 passed, and an SC5 spot check (network cut, `kill -9` mid-restore) left no corruption. Surprises: Cemu opened its last active account until `play` passed `-a <account>`; a network cut printed a raw storage error (now friendly and localized, P6). SC5 at every journal step is covered by the synthetic network suite ([reliability §SC5 coverage](../specs/reliability.md#sc5-coverage)). Still open: daemon budgets (the daemon is still a stub).

## Phase 2: Desktop daemon
- **Goal:** always-on desktop sync with native presence. Plan: [phase-2-daemon](plans/phase-2-daemon.md), Linux first ([D36](../decisions.md#d36-phase-2-builds-the-daemon-on-linux-first-2026-10-28)).
- **Deliverables:** `lemusyncd` + `ipc`; service install on all three OSes; tray/menu bar; actionable notifications (Fluent en/es); launcher and watch modes with process detection; sleep inhibition and resume handling; `doctor`.
- **Exit (M2):** 72-hour soak on Windows, macOS and Linux with sleep/resume cycles. No missed or late sync beyond the budgets. Tray states verified. The Linux soak comes first; Windows and macOS follow Phase 3's first usable UI ([D40](../decisions.md#d40-the-desktop-ui-comes-before-the-windows-and-macos-daemon-2026-11-01)).

## Phase 3: Flutter app (desktop)
- **Goal:** the beautiful, parity-locked UI.
- **Deliverables:** design exploration (brand, fonts, icon set); design system tokens + components; navigation shell; every flow in [navigation](../specs/ui/navigation.md); FRB bridge over IPC; goldens; performance tests; accessibility pass.
- **Exit (M3):** a new user goes from install to first synced save in under 3 minutes unaided. Goldens pass for every size class, theme and locale. Frame budgets met.

## Phase 4: Android
- **Deliverables:** engine in-process (FRB + UniFFI); `SaveFs` with SAF; launcher mode via intents; foreground service and WorkManager; Live Updates (Now Bar, Super Island) and ongoing notifications; QS tile; app shortcuts; Android profiles; Play + F-Droid + APK builds.
- **Exit (M4):** SC1–SC3 between an Android device and a PC with RetroArch and one standalone emulator. Live Update verified on Pixel, Samsung and Xiaomi.

## Phase 5: iOS
- **Deliverables:** Files bookmarks; Live Activity + Dynamic Island; App Intents + guided Shortcuts automation; background tasks; iOS profiles; TestFlight.
- **Exit (M5):** SC2 between an iPhone and a PC with at least one emulator that exposes saves in Files.

## Phase 6: Breadth
More backends (Google Drive, OneDrive, Dropbox, MEGA); more profiles (RPCS3, Yuzu-family forks, Azahar, melonDS, PPSSPP, mGBA, Ryujinx-family index reader, Vita3K, xemu, Flycast…); opt-in save states; headless packages and Linux handhelds; compat groups; storage how-to guides.

## Phase 7: Hardening and 1.0
External security audit (look for funding, e.g. open-source security grants); key rotation and device revocation; extended fuzzing; reproducible builds; signing on every channel; store submissions; docs site.

## Later ideas (not committed)
UnifiedPush / optional push relay for instant lock warnings · QR device pairing · widgets and Control Center controls · chunked dedupe for large save states · Steam Deck Decky plugin · opt-in, sandboxed per-game progress readers.
