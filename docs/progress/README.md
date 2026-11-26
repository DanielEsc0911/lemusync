# Progress

> **Status:** Decided · **Related:** [roadmap](roadmap.md), [rules](../rules.md) (W1, W4), [git-workflow](../specs/git-workflow.md), [release](../specs/release.md#release-checklist), [skill: lemusync-docs](../../.claude/skills/lemusync-docs/SKILL.md) · **Code:** —

## How it works
- **[roadmap.md](roadmap.md):** phases, milestones and exit criteria. It changes only when scope changes.
- **`plans/<name>.md`:** one file per active plan (a whole phase, or one feature inside a phase), in the writing-plans format: goal, global constraints, tasks with checkboxes.
- **Progress is the ticked checkboxes**, committed together with the work they describe.
- **Durable knowledge never lives only in a plan.** Spike results, design changes and new decisions go into specs and `decisions.md` in the same commit (W1).
- **Cleanup:** when every task in a plan is done and merged to `main`, add its user-visible changes to `CHANGELOG.md` `[Unreleased]`, delete the plan file, and remove its board row. Git history keeps the plan.
- **Closed plans are history.** Their command shapes and examples stay as written then (for example `play <emulator> <game>`); the current ones are in the specs.
- **Next plan:** written at the end of the previous phase, once that phase's results are known (no speculative plans, P1).

## Status board
| Plan | Phase | Status | Updated |
|---|---|---|---|
| [phase-0-foundations](plans/phase-0-foundations.md) | 0 | In progress (Tasks 1–5 done; SP7 Linux done; SP6 in progress, Google Drive live done; SP1 in progress, Fedora, Android emulator and CachyOS done; SP3 skipped until an Android device is available; next M1 sync Fedora ↔ CachyOS) | 2026-10-18 |
| [phase-1-first-sync](plans/phase-1-first-sync.md) | 1 | In progress (Tasks 1–17, 17b (S3/R2), 17c (retries) and 17d (only new blobs, live progress) done; Task 18 done: **M1a reached 2026-10-20**; Task 19 (SP8) done 2026-10-20: Cemu accepts LemuSync-written accounts, option c designed; Task 8 Step 8, Task 17b Step 6 and Task 17d Step 7 live runs are the maintainer's) | 2026-10-20 |
| [phase-2-daemon](plans/phase-2-daemon.md) | 2 | In progress (Tasks 1–7, 9, 10, 14, 15 and 16 (budgets) done; Task 8 Step 3 and Task 15 Step 3 are the maintainer's; Task 11 deferred until the UI (D40); Task 12 decided: document portal (D41), spike SP11; Task 13 Linux soak running (2026-11-26, Task 16 build: 0.08–0.13 % CPU, peak RSS 21.0 MiB, 0 restarts, longest run 22.2 h; failed syncs only while offline; a 72 h run is still needed); Linux first, D36) | 2026-11-26 |
| [phase-3a-brand](plans/phase-3a-brand.md) | 3 | In progress (Tasks 1–12 done, Task 9 awaiting maintainer review; V-BRAND-2 pending) | 2026-11-01 |
| [phase-3b-shell](plans/phase-3b-shell.md) | 3 | In progress (Tasks 1–11 done, Task 11 awaiting maintainer review) | 2026-11-25 |
| [phase-3c-flows](plans/phase-3c-flows.md) | 3 | In progress (Task 1 done: plan, D46–D49; next Task 2, bridge calls; UI tasks after the 3b review) | 2026-11-26 |

Status values: Not started · In progress · Blocked (reason) · Done (awaiting cleanup).
