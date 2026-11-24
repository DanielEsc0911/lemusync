# Players and accounts

> **Status:** Partial · **Related:** [rules](../rules.md) (P3, P5, B2, B3), [sync-model](sync-model.md), [emulator-profiles](emulator-profiles.md), [glossary](../glossary.md), [navigation](ui/navigation.md) · **Code:** `crates/core/src/config.rs`, `crates/engine/src/accounts.rs`, `profiles/cemu.toml`

## Purpose
Map people (players) to emulator user folders on every device, so each player's saves stay separate and follow them from machine to machine.

## Concepts
- **Player:** a person in the repo. A solo user has exactly one player and only ever sees their own name (progressive disclosure, P5).
- **Device:** one install. Random device id, user-chosen name, **default player**.
- **Account mapping:** (device, emulator, player) → emulator account id. Stored in [repo config](sync-model.md#repo-config).

## Multi-user emulators (Full tier)
| Emulator | Account id | Location (Cemu verified on Linux in SP7; others Verify) |
|---|---|---|
| Cemu (Wii U) | persistent id `8xxxxxxx` (first account: `80000001`) | accounts `mlc01/usr/save/system/act/<id>/`; saves `mlc01/usr/save/00050000/<titleid>/user/<id>/` |
| RPCS3 (PS3) | user `0000000N` | `dev_hdd0/home/<user>/savedata/<serial>/` |
| Yuzu-family forks (Switch: Eden, Citron…) | 128-bit profile UUID | `nand/user/save/0000000000000000/<uuid>/<titleid>/` |

### Cemu accounts (SP7)
Verified on Cemu 2.6 (Linux Flatpak) against the [v2.6 source][cemu-acc], 2026-10-08:
- One folder per account: `mlc01/usr/save/system/act/<persistent id, 8 hex digits>/account.dat`. Next to them, `act/persisid.dat` stores the id counter.
- `account.dat` is a line-based text file. The first line is `AccountInstance_20120705`, then `Key=value` lines (`PersistentId`, `Uuid`, `MiiData`, `MiiName`, `AccountId`, `Country`, …).
- **Display name:** key `MiiName`. The value is exactly 44 hex characters: 11 UTF-16 code units, each written as 4 hex digits (most significant first), padded with `0000`. Decode until the first `0000`. Cemu rejects any other length. The maintainer's real file matches this format (value not recorded).
- **BotW and `user/common`:** BotW (US, `00050000/101c9400`) creates `user/common/`. On the maintainer's long-played install (Fedora 44, Cemu 2.6 Flatpak) it exists but is empty. All game data (`0/`–`5/` slots, `option.sav`, `album/`, `pict_book/`, `tracker/`) is under `user/80000001/`. So BotW is fully per-player. The profile still lists `user/common` under `saves.shared` for games that use it.
- **Second Linux install (CachyOS, 2026-10-18):** native package `cemu-bin` 2.6 (`/usr/bin/cemu`), data root `~/.local/share/Cemu`, config `~/.config/Cemu/settings.xml` with an empty `mlc_path`. Two accounts, `80000001` and `80000002`. Both `account.dat` files have the same first line, the same key set and a 44-character `MiiName`. BotW US has full game data under each `user/<id>/`, and `user/common/` is empty. Everything matches the Fedora install (names and formats only, no values recorded). The maintainer then deleted `80000002` on both machines for M1 ([roadmap](../progress/roadmap.md#phase-1-sync-engine--cli-desktop)): on the CachyOS laptop on 2026-10-18, on the Fedora PC on 2026-10-19 after its panic backup (there BotW's `user/80000002/` holds 20 files and `80000001` holds 317). Deleting an account in Cemu removes only `system/act/80000002/`. BotW's `user/80000002/` save folder stays behind, so discovery can find save folders with no matching account.
- **New accounts on the M1a test day (2026-10-20):** each PC created one account in Cemu for the other PC's player. Fedora got `80000002`, the id of the account deleted on 2026-10-19, so the new account inherited the orphaned `user/80000002/` folder. CachyOS got `80000003`. A new account can therefore land on a folder that already holds files: `accounts map` asked keep or replace, and **replace** moved the orphan's files to local history before restoring the player's save. Neither orphan was recovered as its own player; the panic backups hold both.
- **BotW title ids** ([WiiUBrew title database][wiiubrew]): JP `00050000-101C9300` (`WUP-P-ALZJ`), US `00050000-101C9400` (`WUP-P-ALZE`), EU `00050000-101C9500` (`WUP-P-ALZP`). Updates use high part `0005000E`, DLC `0005000C`. Cemu writes folder names in lowercase.

[cemu-acc]: https://github.com/cemu-project/Cemu/blob/v2.6/src/Cafe/Account/Account.cpp
[cemu-gs]: https://github.com/cemu-project/Cemu/blob/v2.6/src/gui/GeneralSettings2.cpp
[wiiubrew]: https://wiiubrew.org/wiki/Title_database

### Creating a Cemu account (SP8)
Cemu builds a new account from two inputs, a persistent id and a Mii name ([Account.cpp][cemu-acc] `Account(persistent_id, mii_name)` and `Save()`, called from [GeneralSettings2.cpp][cemu-gs] `OnAccountCreate`). Everything else is generated or left empty, so LemuSync can create an account the same way, with no template account:

| Key (file order) | Value Cemu writes |
|---|---|
| `PersistentId` | the id, 8 hex digits. Next free id = max(`persisid.dat` `PersistentIdHead` + 1, highest account id + 1), at least `80000001`. At most 12 accounts |
| `TransferableIdBase` | `2000004` followed by the last 8 hex digits of `Uuid` |
| `Uuid` | 16 random bytes, 32 hex digits |
| `MiiData` | 96 bytes, 192 hex digits: a fixed default Mii, the name as 10 UTF-16LE units at byte `0x1A`, and a big-endian counter at bytes 4–7 raised from 0 until the CRC-16 (polynomial `0x1021`, Cemu's `FFLCalculateCRC16`) of the 96 bytes is 0. Same name → same bytes |
| `MiiName` | the name, as in [Cemu accounts](#cemu-accounts-sp7). Up to 10 UTF-16 units |
| `AccountId` | empty |
| `BirthYear`, `BirthMonth`, `BirthDay`, `Gender` | `0` |
| `EmailAddress` | empty |
| `Country`, `SimpleAddressId`, `PrincipalId`, `IsPasswordCacheEnabled` | `0` |
| `AccountPasswordCache` | 64 zeros |

The file is the `AccountInstance_20120705` line, then the keys in this order, one per line, each line ending in `\n` including the last ([Account.cpp][cemu-acc] `Account::Save()`).

**Evidence (2026-10-20, Fedora 44, Cemu 2.6 Flatpak, scratch data tree, live data untouched):** a throwaway script outside the repo wrote `SP8 Test` as `80000002`. Cemu listed it (log: `IOSU_ACT: using account SP8 Test in first slot`), BotW US booted with it, and an in-game save created `00050000/101c9400/user/80000002/` (`option.sav`, `0/`, `1/`, `tracker/`). Cemu then created `sp8cemu` itself as `80000003`. Its `MiiData` matched the script's output for the same name byte for byte, and it had the same keys in the same order, with the same values except the per-account ones (id, `Uuid`, `TransferableIdBase`, name and `MiiData`). Its `TransferableIdBase` ended in its `Uuid`'s last 8 hex digits.

**Quirks:**
- Cemu's `settings.xml` keeps the active account (`<Account><PersistentId>`, decimal). If that account doesn't exist, Cemu uses the first one and still writes `system/pdm/<old id>/PlayDiary.dat`. LemuSync doesn't edit `settings.xml`. `play --as` starts Cemu with `-a <id>`, which selects the player's account for that session only and leaves `settings.xml` alone ([LaunchSettings.cpp][cemu-launch], only ids ≥ `80000001`). Before `-a` (Task 9, 2026-10-21) Cemu opened its last active account, so the game saved under the wrong player. Outside `play`, the player selects the account in Cemu.
- Copying an existing account as a template would also copy its online fields (`AccountId`, `PrincipalId`, password cache). So LemuSync never copies one ([D32](../decisions.md#d32-new-cemu-accounts-are-generated-like-cemu-does-never-copied-2026-10-20)).
- Flatpak overrides `XDG_DATA_HOME` and `XDG_CONFIG_HOME` given with `flatpak run --env`. A scratch Cemu needs `flatpak run --filesystem=<scratch> --command=sh info.cemu.Cemu -c "export XDG_DATA_HOME=… XDG_CONFIG_HOME=…; exec Cemu-wrapper"`. Flatpak's `/tmp` is private, so the scratch tree can't be there.

**Design (option c, built in phase-1b-play Task 6):**
- **Profile data, not code (P3).** A new `accounts.create` profile section describes the file: id rule and counter file, header line, max accounts, name length, and the key list in order. Each value is a template built from generic primitives in core: the id, random bytes, hex slices of another value, the name in UTF-16 (as hex, or as bytes at an offset), a fixed byte block, and a CRC-16 fix-up (polynomial, which bytes to vary, target 0). Cemu's table above becomes data in `profiles/cemu.toml`.
- **Fields (`[accounts.create]`, `crates/core/src/account_template.rs`).** `id = { format = "hex8", min, counter_file, counter_key }` (the counter file is read, never written), `max_accounts`, `name_max_utf16`, `header`, `file` (written as `[accounts].root/<id>/<file>`), `[accounts.create.vars]` and `[[accounts.create.keys]]`. Each key has a text `value`, or a hex `bytes` block with optional `put_utf16le = { at, units, from }` and `crc16 = { poly, vary = [start, end], target }`. `vars` are computed before any key is written, so a key can use a value that a later key prints (`TransferableIdBase` uses `uuid`); they may only use `{id}`, `{name}` and `{random:N}`. Templates use a closed set: `{id}` (8 hex digits), `{name}`, `{<var>}`, `{random:N}` (N random bytes as hex, vars only), `{slice:<var>:A:B}` (hex characters A..B), `{utf16hex:<var>:UNITS}` (UTF-16 units as 4 hex digits each, NUL-padded). Anything else fails profile validation (B2). The name is cut to `name_max_utf16` units at a character boundary, so a non-BMP character (a surrogate pair) is dropped whole rather than split. Golden test: `profiles/fixtures/cemu/account-create/`, hand-built in `Save()` order around the `sp8cemu` `MiiData` that Cemu wrote in SP8 ([V-ACC-4](#verify)).
- **Safety (B2, B3).** Only while the emulator isn't running. The new account folder must not exist. The file is written to a temp name, synced, then renamed. Nothing else is written: not `persisid.dat` (Cemu rewrites it), not `settings.xml`, not other accounts.
- **Flow (P5).** `lemusync accounts create <player> cemu [--name <mii name>]` and the same action in the UI. The name defaults to the player's name, cut to 10 UTF-16 units. The CLI and UI then map the new account to the player and restore the player's latest version through the normal restore.
- **Confirmation.** Before writing: "Create cemu account “Ana” for Ana on this device? cemu must be closed." (`--yes` skips it; the id is picked at write time and shown after). After writing: "Created cemu account “Ana” (80000003) for Ana. Open cemu and select “Ana” in Account settings." Spanish strings go through [i18n](i18n.md#terminology).

### The same-ID trap
Every Cemu install creates its first account as `80000001`. Two machines with differently *named* accounts usually still share that id, so syncing raw folders would mix two players' saves. LemuSync never syncs by account id. Each device maps a player to its own local id:

```toml
# repo config (edited through UI/CLI; TOML shown for clarity)
[players.daniel.accounts.cemu]
desktop = "80000001"
laptop  = "80000002"

[players.ana.accounts.cemu]
laptop  = "80000001"
desktop = "80000002"
```

### Discovery
A scan lists the accounts it finds (id, plus the display name if the profile says how to read it). LemuSync proposes mappings by matching names and the player confirms. Unmatched accounts → ask. Two players can't map to the same id on one device. Scan: `crates/engine/src/scan.rs`.

**Orphaned saves.** A scan also lists save folders whose account no longer exists (for Cemu, `user/<id>/` with no `system/act/<id>/`). Deleting an account in Cemu leaves them behind ([above](#cemu-accounts-sp7)). The scan never ignores or deletes them. It offers to recover one: the player picks who it belongs to, and LemuSync syncs it into that player's stream. The folder is only read; LemuSync writes it to a live account only through a normal restore. Seen on the maintainer's CachyOS laptop (2026-10-18).

**A player with no account here.** When a player's stream exists in the repo but the player has no account mapped on this device, `sync` reports it and does nothing for that stream. The player chooses:
- **a. Use an existing account:** `lemusync accounts map <player> cemu <id>`. If that account already has files and the player has a version in the repo, the CLI asks whether to keep them (they're uploaded and become a second head, a conflict) or replace them (`--replace`): the current files go to local history first (I1), then the player's latest version is restored.
- **b. Create the account in the emulator first,** then `lemusync scan` and `accounts map`.
- **c. Let LemuSync create it:** proven in SP8 ([Creating a Cemu account](#creating-a-cemu-account-sp8)). Not built yet.

**Recovering an orphaned save:** `lemusync accounts recover cemu <id> --player <name>` uploads the orphaned folder as the first version of that player's stream. It's refused if the player already has a version of that game, so a recovered save can never replace one. The folder itself is only read.

### Shared folders
Some games write to a folder shared by all accounts (Wii U `user/common`). Per-player separation is impossible for those files. That part of the game drops to Basic (one lease for all players), and the UI says so. Profiles list such paths under `saves.shared`.

## Single-user emulators (Standard tier)
The emulator has one save location per game. Per-player saves work by **swapping**:
- **Launcher mode:** before launch, the chosen player's latest version is restored into the emulator's save location. The current content is backed up first, and if it's dirty it's uploaded to its own stream first. After the emulator exits, the snapshot goes to the chosen player's stream.
- **Watch mode:** changes belong to the device's default player. "Play as Ana" is an explicit action that performs the swap.
- **Never swap while the emulator runs.**

## Basic tier
Swapping works the same way, but at container granularity: the whole memory card is swapped, and the lease covers the container.

## Edge cases
| Case | Behaviour |
|---|---|
| Player removed | Mappings removed; streams and history kept |
| Device renamed | Mappings follow the device id |
| Account deleted in the emulator | Mapping flagged on next scan; asks the player. Its save folders stay and show up as orphaned saves ([Discovery](#discovery)) |
| New device, existing players | Join flow asks for this device's mappings ([navigation](ui/navigation.md)) |

## Verify
- **V-ACC-1** Same Cemu layout and `account.dat` format on Windows and macOS. Deferred by the maintainer on 2026-10-08 (spike SP7 device pass). The second Linux install (CachyOS) is done ([above](#cemu-accounts-sp7)).
- **V-ACC-4** Capture a Cemu-written `account.dat` from a scratch tree on the M1b test day (phase-1b-play Task 9 B1) and replace the hand-built golden fixtures in `profiles/fixtures/cemu/account-create/` (the SP8 scratch tree was deleted). Deferred by the maintainer on 2026-10-27; the hand-built fixtures stay until then.
- **V-ACC-2** RPCS3 and Yuzu-family layouts (Phase 6 profile work).
[cemu-launch]: https://github.com/cemu-project/Cemu/blob/main/src/config/LaunchSettings.cpp
