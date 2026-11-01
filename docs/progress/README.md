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
| [phase-1b-play](plans/phase-1b-play.md) | 1 | Done (M1b reached 2026-10-21; SC5 network follow-ups done 2026-10-27; V-ACC-4 deferred) | 2026-10-27 |
| [phase-2-daemon](plans/phase-2-daemon.md) | 2 | In progress (wave 1 done: Tasks 1–7; Task 8 Steps 1–2 done, Step 3 is the maintainer's; Task 9 done; Task 10 done (idle CPU and binary size over budget); Linux first, D36) | 2026-10-29 |

Status values: Not started · In progress · Blocked (reason) · Done (awaiting cleanup).
