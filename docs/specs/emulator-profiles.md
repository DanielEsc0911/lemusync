# Emulator profiles

> **Status:** Draft · **Related:** [rules](../rules.md) (P3, B2), [decisions](../decisions.md) (D7, D8, D9), [players-and-accounts](players-and-accounts.md), [sync-model](sync-model.md), [security](security.md#file-system-confinement), [platforms](platforms/desktop.md) · **Code:** `profiles/`, `crates/core/src/profile.rs`

## Purpose
Describe, as data, where each emulator keeps saves and how to recognise games, so the engine can sync any emulator safely. Profiles are TOML, ship inside releases, and never contain code ([D8](../decisions.md#d8-emulator-profiles-are-declarative-toml-shipped-inside-releases-2026-10-07)). Only save folders are synced, never an emulator's whole data folder (e.g. not Cemu's `mlc01` with its updates and DLC).

## Layouts
| Layout | Meaning | Examples (Verify) |
|---|---|---|
| `per_game_user_dir` | A folder per game per emulator account | Cemu, RPCS3, Yuzu-family forks |
| `per_game_dir` | A folder per game, no accounts | Dolphin Wii NAND, PPSSPP `SAVEDATA`, Azahar |
| `per_game_file` | A file per game | RetroArch `.srm`, melonDS `.sav`, mGBA, DuckStation per-game cards |
| `container` | One file holds many games | PS1/PS2 shared memory cards, Dolphin GameCube `.raw` cards |

## Tiers
The engine derives the tier. Profiles don't declare it ([D9](../decisions.md#d9-capability-tiers-instead-of-an-allow-list-2026-10-07)).

| Tier | When | Gets |
|---|---|---|
| **Full** | `per_game_user_dir` | Per-game + per-player streams via account mapping, per-game leases, launcher handoff |
| **Standard** | `per_game_dir` / `per_game_file` | Per-game streams and leases. Per-player saves through swapping ([players-and-accounts](players-and-accounts.md#single-user-emulators-standard-tier)) |
| **Basic** | `container`, or game folders with opaque names | One stream and lease for the whole container. Suggests a setting that unlocks Standard |

**Every tier** gets version history, conflict detection, leases, atomic restore with backup, and E2E encryption.

## Schema (v1)
| Field | Type | Meaning |
|---|---|---|
| `schema` | int | Profile schema version (`1`) |
| `id` | string `[a-z0-9-]+` | Stable id. Never renamed |
| `name` | string | Display name (proper noun, not translated) |
| `systems` | [string] | Console ids, e.g. `wiiu`, `ps2` |
| `layout` | enum | See [Layouts](#layouts) |
| `requires_closed` | bool, default `true` | Never snapshot or restore while the emulator runs |
| `platforms.<os>.roots` | [template] | Candidate base folders, in priority order (`os`: windows, macos, linux, android, ios) |
| `platforms.<os>.config_roots` | [string] | Config folder for each entry of `roots`, same order (Cemu keeps `settings.xml` outside its data folder). Optional; defaults to the matching root |
| `platforms.<os>.process` | [string] | Process names for detection (desktop) |
| `platforms.android.package` | string | Package name for launching and detection |
| `config_keys` | [{file, format, key, var, default?}] | Read a user-customised path from the emulator's config (`format`: xml, ini, json) into variable `var`; optional `default` is the value used when the key is missing |
| `saves.root` | template | Save root under the base folder |
| `saves.pattern` | template | Path per stream, with captures `{title_id}`, `{account}`, `{game}` |
| `saves.shared` | [template] | Folders shared by all accounts |
| `saves.include` / `saves.exclude` | [glob] | Files that belong to a stream |
| `saves.max_file_mb` | int, default `64` | Per-file size limit for snapshots (disk-fill guard) |
| `accounts` | {root, id_regex, name_file?, name_key?} | Account discovery (multi-user emulators) |
| `accounts.create` | {id, max_accounts, name_max_utf16, header, file, vars?, keys[]} | How to write a new account as data, from a closed set of template functions ([players-and-accounts](players-and-accounts.md#creating-a-cemu-account-sp8)). Implemented (`crates/core/src/account_template.rs`) |
| `game_key` | {from, normalize[]} | Which capture identifies the game, and how to normalise it |
| `compat` | [string] | Compat groups whose files are interchangeable |
| `states` | {pattern, include} | Save states (opt-in, [D15](../decisions.md#d15-save-states-are-opt-in-2026-10-07)) |
| `recommend` | [{setting, value, unlocks, how}] | Emulator settings that raise the tier, shown in the UI |
| `games.dirs` | {file, key} | Optional. Game folders the emulator lists: `file` in the config root (like `config_keys`), `key` the element path of a repeated element whose text is one absolute folder. Read only when granted. Cemu: `{ file = "settings.xml", key = "content/GamePaths/Entry" }` ([Game discovery](#game-discovery)) |
| `games.scan` | {marker, key, key_last?, name} | Optional. A folder holding the XML file `marker` (relative path) is one game, and that folder is its `{game_path}`; `key` and `name` are element paths in the marker, `key_last` keeps the key's last N characters. Each game folder and its direct subfolders are checked. Implemented in `crates/engine/src/games.rs` |
| `launch.args` | [string] | Arguments to start a game. Each one is a literal or exactly `{game_path}` or `{account}`, the only variables here; each is passed as one argument, never split or joined. `{account}` is the chosen player's account on this device (`play --as`). At least one must be `{game_path}`. `[launch]` has no other field: the executable is always chosen by the user (`[emulators.<id>] exe` in [config](config-and-cli.md#config-file)) and started with argv, never a shell. Cemu: `["-g", "{game_path}", "-a", "{account}"]` (`-a` takes the hex persistent id for that session only, [LaunchSettings.cpp][cemu-launch]; Verify V-PLAY-1) |

### Templates
Allowed variables: `{base}` `{home}` `{xdg_data}` `{xdg_config}` `{appdata}` `{localappdata}` `{documents}` `{mac_app_support}` `{flatpak:<app-id>}` `{android_storage}` `{exe_dir}`, plus `config_keys` variables and pattern captures. Anything else is a validation error.

The M1a parser (`crates/core/src/profile.rs`) supports `{home}` `{xdg_data}` `{xdg_config}` `{exe_dir}` `{flatpak:<app-id>}` plus `{base}` `{mlc}` and captures; the other variables arrive with the platforms that need them.

### Example (Cemu; Linux part verified in SP7, Windows is still Verify SP7)
```toml
schema = 1
id = "cemu"
name = "Cemu"
systems = ["wiiu"]
layout = "per_game_user_dir"
requires_closed = true

[platforms.linux]
roots = ["{exe_dir}/portable", "{flatpak:info.cemu.Cemu}/data/Cemu", "{xdg_data}/Cemu"]
config_roots = ["{exe_dir}/portable", "{flatpak:info.cemu.Cemu}/config/Cemu", "{xdg_config}/Cemu"]
process = ["Cemu_relwithdebinfo", "Cemu", "cemu"]

[platforms.windows]
roots = ["{exe_dir}", "{appdata}/Cemu"]
process = ["Cemu.exe"]

[[config_keys]]
file = "settings.xml"              # relative to the matching config root
format = "xml"
key = "content/mlc_path"
var = "mlc"
default = "{base}/mlc01"           # used when the key is missing or empty

[saves]
root = "{mlc}/usr/save"
max_file_mb = 64
pattern = "00050000/{title_id}/user/{account}"
shared = ["00050000/{title_id}/user/common"]

[accounts]
root = "{mlc}/usr/save/system/act"
id_regex = "^8[0-9a-f]{7}$"
name_file = "account.dat"
name_key = "MiiName"

[game_key]
from = "title_id"

[launch]
args = ["-g", "{game_path}", "-a", "{account}"]   # Verify V-PLAY-1
```

## Validation
Rejected at load time, with a clear error. Enforced in `core` and fuzzed:
- Unknown fields (`deny_unknown_fields`), wrong types, unsupported `schema`.
- Template variables not on the allow list. Absolute paths or `..` in patterns.
- Regexes over the size limit. Regexes use the linear-time `regex` crate, which can't backtrack catastrophically.
- Any field that would name an executable. The emulator executable is always picked by the user.

## Discovery
1. For each profile on the current OS: expand `roots` in order. The first that exists is the base.
2. Apply `config_keys` (the user's custom paths inside the emulator's own config).
3. Apply user overrides ([Overrides](#overrides)).
4. Propose grants for the resolved folders. Nothing is read until the user grants it ([security](security.md#file-system-confinement)).
5. Enumerate streams from `saves.pattern` and accounts.

Implemented for `per_game_user_dir` in `crates/engine/src/{profiles,discover}.rs`; fixture test `crates/engine/tests/discovery.rs`. Account ids must match `id_regex` against the whole folder name. `saves.pattern` supports the captures `title_id` and `account` only; any other capture is a config error. A path read from the emulator's own config (`mlc_path`) is not trusted as granted: it goes through the grants and the symlink check like any other. A symlink anywhere below the saves root drops that folder with an `unreadable_folder` warning. If the config file exists but isn't granted, the install is provisional (defaults, possibly stale): scan reports it, sync refuses it.

## Game discovery
Which games are installed, where, and what they are called, so `play` needs no hand-written game paths and every output shows names (spike SP10, 2026-10-27). Read-only: LemuSync reads metadata and never copies, moves or creates game files or folders.

### Cemu 2.6 (SP10)
Source: `cemu-project/Cemu` at tag `v2.6`. Confirmed on Linux against the maintainer's Flatpak (`info.cemu.Cemu`), structure only (element names, file names, folder depth), 2026-10-27.

| Fact | Evidence (v2.6) | Linux Flatpak |
|---|---|---|
| Game folders the user added are `settings.xml` → `content/GamePaths/Entry`, one absolute path per `Entry` text, no attributes | `CemuConfig.cpp` L124–139 (load), L421–426 (save); root `content`, L22 | Confirmed: `content/GamePaths/Entry`, no attributes |
| Cemu scans each game folder recursively. A folder with `content/`, `code/` and `meta/` is an unpacked title; files ending `.wud`, `.wux`, `.iso`, `.wua`, `.wuhb` or named `title.tmd` are titles too. It also scans `mlc01/usr/title/*` | `TitleList.cpp` L268–289, L328–347, L350–400 | Unpacked titles sit one level below each game folder, next to `.wua`/`.wux` files |
| A title list cache `title_list_cache.xml` in the user data folder: `title_list/title` with attributes `titleId` (16 hex), `version`, `sdk_version`, `group_id`, `app_type` and children `region`, `name`, `format`, `path`, `sub_path`. `format`: 1 folder, 2 WUD/WUX, 3 WUA, 4 NUS, 5 WUHB | `main.cpp` L139; `TitleList.cpp` L49–144; `TitleInfo.h` L124–133 | **Absent** in `data/Cemu` although two game folders are set. Not used (see V-GAME-2) |
| The display name is `meta/meta.xml` → `menu/longname_<lang>`, in the console language from `settings.xml` `console_language`, else `longname_en`, else "Unknown Title". Newlines become spaces. A custom name from `content/GameCache/Entry/custom_name` wins in the game list | `TitleInfo.cpp` L711–725; `ParsedMetaXml.h` L90–96; `wxGameList.cpp` L320–333; `CemuConfig.cpp` L143–160 | `meta.xml`: root `menu`, children `title_id` (`type="hexBinary" length="8"`, 16 hex) and `longname_en`, `longname_es`… |
| `-g <path>` accepts a title folder holding `content/`, `meta/` and `code/`, or a `.rpx` inside `code/` next to `content/` and `meta/` (both resolve to the same folder title), or a `.wud`/`.wux`/`.iso`/`.wua`/`.wuhb` file or `title.tmd` | `LaunchSettings.cpp` L124–132 → `MainWindow::FileLoad` L471–474 → `TitleInfo::DetectFormat`, `TitleInfo.cpp` L180–284 (folder L274–282, `.rpx` L186–201) | Not launched (the maintainer's games stay untouched); the M1b test day (2026-10-21) launched a `.wux` set in `games.<key>`, which still wins |
| `.wua` is a ZArchive holding one `<titleId>_v<version>` folder per title; `.wud`/`.wux` are disc images. Reading their `meta.xml` needs the archive or image reader | `TitleInfo.cpp` L57–111, L142, L217–262 | `.wua` and `.wux` present |

**What LemuSync does with it** (the `[games]` profile section, [Schema](#schema-v1)): read the game folders from `content/GamePaths/Entry` (plus the user's own `game_dirs`), and in each folder and its direct subfolders take `meta/meta.xml` → `menu/title_id`, last 8 hex digits lowercased (the save key, as in `00050000/<key>`), and `menu/longname_en`. The game's `{game_path}` is that folder: Cemu accepts it as well as the `.rpx`, and passing the folder needs no listing of `code/` (P1). A `games.<key>` entry in `config.toml` always wins, so a path that already worked is never replaced. One level deep, not recursive like Cemu: that is the layout found, and it bounds the scan. Archives and disc images are not opened (P1): those games appear only through a `games.<key>` entry in `config.toml`.

**Limits** (B2): `settings.xml` and each `meta.xml` are read only if regular files under 1 MiB, never through a symlink; a `DOCTYPE` is rejected (`roxmltree` refuses DTDs by default, so no entity expansion); at most 5,000 entries per folder; game folders must be absolute and without `..` (a `..` entry, even one the user's own Cemu wrote, is dropped rather than resolved: resolving it would read a folder nobody listed; the user can list the real folder in `game_dirs`); the launch path is that folder, passed as one argv element, never through a shell, and it is absolute, so it can't be read as an option; names are stripped of control characters, trimmed and capped at 200 characters; a key must be 8 hex digits.

## Overrides
Per device: base path, extra roots, include/exclude tweaks, disable a profile. **Local custom profiles** (same schema) live in `<config dir>/profiles/` and are trusted because the user created them. Both are editable in UI → Settings → Emulators → Advanced, in the CLI, and in the config file ([config-and-cli](config-and-cli.md)).

## Initial set (Phase 1)
| Emulator | Layout → tier | Linux default (SP7) | Other platforms (Verify SP3/SP5/SP7) |
|---|---|---|---|
| Cemu | `per_game_user_dir` → Full | Full: `user/<account>/` per title, plus `user/common` | Windows, macOS (Android port: verify) |
| RetroArch | `per_game_file` → Standard (honours "sort saves by core") | Standard: `saves/<core>/<rom>.srm` | All, including Android and iOS |
| Dolphin | Wii: `per_game_dir` → Standard. GC: `container` (`.raw`) → Basic, or GCI folders → Standard | GC: GCI folder is the default → Standard | Windows, macOS, Android |
| DuckStation | Shared card → Basic. Per-game cards → Standard (recommended setting) | Per-game card (by title) is the default → Standard | Windows, macOS, Android |
| PCSX2 | `.ps2` file → Basic. Folder memory cards → Standard (recommended setting) | `Mcd001.ps2` file → Basic | Windows, macOS |

These five cover every layout and both ways of handling players. Draft profiles and fixture trees: `profiles/<id>.toml`, `profiles/fixtures/<id>/` (the Cemu profile is loaded and tested by `crates/core`).

### Linux paths (SP7)
Verified 2026-10-08 on Fedora 44 x86_64. Flatpak trees were read from real installs (names only). Native and portable rules come from the upstream source at the tested version. Process names are what `ps -o comm` shows; Linux truncates `comm` to 15 characters, so discovery must match the full executable name (`/proc/<pid>/exe` or `cmdline`), not `comm`. The engine's Linux probe lists `/proc` each poll and reads `exe` (file name, ` (deleted)` stripped) and `comm` only for pids it hasn't seen, re-reading `comm` of known pids and re-reading `exe` when `comm` changed (an `exec`). When `exe` is unreadable (another user's process) it falls back to `comm`, where a full 15-byte `comm` also matches a longer name it starts, so a running emulator is never missed (I3).

| Emulator (version) | Base roots, in the emulator's own order | Config file → save-path key | Process | Default save layout |
|---|---|---|---|---|
| Cemu 2.6 (Flatpak `info.cemu.Cemu`) | Portable: folder `portable/` next to the executable (or next to the AppImage, via `$APPIMAGE`). Else data `$XDG_DATA_HOME/Cemu` (`~/.local/share/Cemu`), config `$XDG_CONFIG_HOME/Cemu`. Flatpak: `~/.var/app/info.cemu.Cemu/data/Cemu` + `…/config/Cemu` ([CemuApp.cpp][cemu-app]) | `settings.xml` → `content/mlc_path` (empty = `<data>/mlc01`, [ActiveSettings.cpp][cemu-mlc]) | `Cemu_relwithdebinfo` (Flatpak binary started by `Cemu-wrapper`; `comm` shows `Cemu_relwithdeb`). Arch package `cemu-bin`: `/usr/bin/cemu`, lowercase (CachyOS, file name only, not launched) | `mlc01/usr/save/00050000/<title_id>/user/<account>/` + `user/common/`, accounts in `mlc01/usr/save/system/act/<id>/account.dat` ([players-and-accounts](players-and-accounts.md#cemu-accounts-sp7)) |
| RetroArch 1.22.2 (Flatpak `org.libretro.RetroArch`) | `$XDG_CONFIG_HOME/retroarch`, else `~/.config/retroarch` ([platform_unix.c][ra-unix]). Flatpak: `~/.var/app/org.libretro.RetroArch/config/retroarch`. Portable/AppImage: Verify SP7 | `retroarch.cfg` → `savefile_directory` (Flatpak writes it with a leading `~`), `savefiles_in_content_dir`, `sort_savefiles_enable` (default `true`), `sort_savefiles_by_content_enable` (default `false`) ([configuration.c][ra-cfg]) | `retroarch` | `saves/[<content dir>/][<core library name>/]<rom name>.srm` ([runloop.c][ra-run]) |
| Dolphin 2606a (Flatpak `org.DolphinEmu.dolphin-emu`) | `portable.txt` next to the exe → `<exe dir>/user`; else `$DOLPHIN_EMU_USERPATH`; else `~/.dolphin-emu` if it exists (never in Flatpak); else data `$XDG_DATA_HOME/dolphin-emu`, config `$XDG_CONFIG_HOME/dolphin-emu` ([UICommon.cpp][dol-ui]). Flatpak: `~/.var/app/org.DolphinEmu.dolphin-emu/data/dolphin-emu` + `…/config/dolphin-emu` | `Dolphin.ini` `[Core]` → `SlotA`/`SlotB` (device type, default `8` = GCI folder), `GCIFolderAPath`/`GCIFolderBPath`, `MemcardAPath`/`MemcardBPath` (`.raw`) ([MainSettings.cpp][dol-main]) | `dolphin-emu` (Flatpak binary started by `dolphin-emu-wrapper`) | GC: `GC/<USA\|EUR\|JAP>/Card A/<maker>-<game id>-<name>.gci`. Wii: `Wii/title/<high>/<low>/data/` ([NandPaths.cpp][dol-nand]) |
| DuckStation 0.1-9482-g0a53bc47c (Flatpak `org.duckstation.DuckStation`) | `portable.txt` or `settings.ini` next to the exe/AppImage → that folder; else `$XDG_CONFIG_HOME/duckstation` when the variable is set (always in Flatpak); else `~/.local/share/duckstation` ([host.cpp][ds-host]). Flatpak: `~/.var/app/org.duckstation.DuckStation/config/duckstation` | `settings.ini` `[MemoryCards]` → `Directory` (default `memcards`), `Card1Type` (default `PerGameTitle`), `Card2Type` (default `None`), `Card1Path`, `UsePlaylistTitle` ([settings.cpp][ds-set]) | `duckstation-qt` | `memcards/<title>_1.mcd` (`PerGame`: `<serial>_1.mcd`; `Shared`: `shared_card_1.mcd`) |
| PCSX2 v2.8.2 (Flatpak `net.pcsx2.PCSX2`) | `portable.ini` or `portable.txt` next to the exe (the `.txt` may hold a custom path) or `-portable`; else `$XDG_CONFIG_HOME/PCSX2`; else `~/.config/PCSX2` ([Pcsx2Config.cpp][p2-cfg]). Flatpak: `~/.var/app/net.pcsx2.PCSX2/config/PCSX2` | `inis/PCSX2.ini` → `[Folders] MemoryCards` (default `memcards`), `[MemoryCards] Slot1_Filename` (default `Mcd001.ps2`), `Slot1_Enable`, `Multitap*_Slot*_*` | `pcsx2-qt` | `memcards/Mcd001.ps2` file card (container). A folder card is a directory with the same name holding `_pcsx2_superblock`, `_pcsx2_index` and one folder per save ([MemoryCardFolder.cpp][p2-folder]) |

Evidence commands: `flatpak list --app --columns=application,version`; `find ~/.var/app/<id> -maxdepth N` (names only); `grep` of the generated config files for key names; RetroArch, DuckStation and PCSX2 were launched once for 15 s to create their default trees, with `ps -eo comm,args` sampled while running. Cemu and Dolphin process names come from the Flatpak's `command` metadata and wrapper script (not launched, to protect the maintainer's data). Seen live on the M1a test day (2026-10-20): `lemusync doctor` reported `cemu running` while Cemu was open, for the Flatpak on Fedora and for `cemu-bin` on CachyOS.

[cemu-app]: https://github.com/cemu-project/Cemu/blob/v2.6/src/gui/wxgui/CemuApp.cpp
[cemu-mlc]: https://github.com/cemu-project/Cemu/blob/v2.6/src/config/ActiveSettings.cpp
[ra-unix]: https://github.com/libretro/RetroArch/blob/v1.22.2/frontend/drivers/platform_unix.c
[ra-cfg]: https://github.com/libretro/RetroArch/blob/v1.22.2/configuration.c
[ra-run]: https://github.com/libretro/RetroArch/blob/v1.22.2/runloop.c
[dol-ui]: https://github.com/dolphin-emu/dolphin/blob/2606a/Source/Core/UICommon/UICommon.cpp
[dol-main]: https://github.com/dolphin-emu/dolphin/blob/2606a/Source/Core/Core/Config/MainSettings.cpp
[dol-nand]: https://github.com/dolphin-emu/dolphin/blob/2606a/Source/Core/Common/NandPaths.cpp
[ds-host]: https://github.com/stenzek/duckstation/blob/0a53bc47c3fa2b073c1cd856583118c25cd01739/src/core/host.cpp
[ds-set]: https://github.com/stenzek/duckstation/blob/0a53bc47c3fa2b073c1cd856583118c25cd01739/src/core/settings.cpp
[p2-cfg]: https://github.com/PCSX2/pcsx2/blob/v2.8.2/pcsx2/Pcsx2Config.cpp
[p2-folder]: https://github.com/PCSX2/pcsx2/blob/v2.8.2/pcsx2/SIO/Memcard/MemoryCardFolder.cpp

## Known special cases
- **Ryujinx and forks (Switch):** save folders have opaque names, and only an internal index maps them to title IDs → Basic until an index reader exists (its own decision entry per [D7](../decisions.md#d7-no-game-save-format-parsing-in-core-2026-10-07)).
- **Portable installs:** `{exe_dir}` root. The executable path comes from the user's launcher setup.
- **Flatpak / Snap / AppImage / EmuDeck / RetroDECK:** listed as extra roots.
- **Android `Android/data`:** not readable by other apps on modern Android ([platforms/android](platforms/android.md#storage-access)). The profile's `recommend` tells users how to move the emulator's folder.

## Authoring and tests
- Fixture trees: `profiles/fixtures/<id>/<case>/`, with `expected.toml` listing the streams the scan must find.
- Each profile file starts with a comment recording the verification evidence: emulator version, OS, date.
- Workflow: the `lemusync-emulator-profile` skill.

## Verify
- **V-PROF-1** Windows and macOS paths, config keys and process names for the initial set; Linux portable/AppImage process names and RetroArch's Linux portable mode; PCSX2 folder-card save-folder naming on a real card (spike SP7). Linux Flatpak and native rules: [Linux paths (SP7)](#linux-paths-sp7). The Windows, macOS and CachyOS passes are deferred by the maintainer (2026-10-08).
- **V-PROF-2** Android save locations and accessibility for each emulator (SP3).
- **V-PROF-3** Which iOS emulators expose saves in the Files app (SP5).
- **V-GAME-1** Cemu `GamePaths` and `meta.xml` on Windows, macOS and native Linux (`cemu-bin`), and the game folders' usual depth there (SP10). Source says the same code path runs everywhere; only Flatpak was seen.
- **V-GAME-2** When Cemu 2.6 writes `title_list_cache.xml` (source: on a finished scan that changed the list, `TitleList.cpp` L320–323, in the user data folder) and why the maintainer's Flatpak has none. Until proven, `[games]` has no `cache` part, so `.wua`/`.wux` names aren't known (SP10).
- **V-GAME-3** Whether names should follow Cemu's `console_language` instead of always `longname_en` (SP10).

## Open questions
Schema gaps found by SP7 (config outside the base is solved by `platforms.<os>.config_roots`; the drafts in `profiles/` work around them with comments; resolve in Phase 1 before adopting):
- **Extra captures.** RetroArch's core folder and Dolphin's region folder have no capture (`{core}`, `{region}`); the drafts match them with a glob.
- **Two layouts in one emulator.** Dolphin has Wii NAND (`per_game_dir`) and GameCube cards (`per_game_file` or `container`). One profile declares one `layout`.
- **Config value forms.** RetroArch paths may start with `~` or be `default`; DuckStation and PCSX2 folders are relative to the data root.
[cemu-launch]: https://github.com/cemu-project/Cemu/blob/main/src/config/LaunchSettings.cpp
