# LemuSync Rust strings (English source). Keys describe meaning (i18n §Rules).
# Every key here must exist in es/lemusync.ftl (cargo xtask i18n-check).

## Help
help-about = Keeps your emulator saves in sync across your devices, through storage you own.
help-about-json = Print machine-readable JSON (keys are always in English).
help-init = Create a new encrypted repo on the storage you set up.
help-join = Join an existing repo from this device.
help-join-recovery = Use the recovery key instead of the passphrase.
help-backend = Set up where the repo is stored.
help-backend-add = Choose the storage.
help-backend-add-gdrive = Use Google Drive (LemuSync only sees the files it created).
help-backend-add-gdrive-client_id = Your OAuth client id (type "Desktop app").
help-backend-add-s3 = Use an S3-compatible bucket (Cloudflare R2, AWS S3, Backblaze B2…).
help-backend-add-s3-endpoint = The endpoint, for example https://<account id>.r2.cloudflarestorage.com
help-backend-add-s3-bucket = The bucket name.
help-backend-add-s3-access_key_id = The access key id. The secret key is asked for next and kept in the system keyring.
help-backend-add-s3-region = The region (R2 uses auto).
help-backend-add-fs = Use a local or mounted folder.
help-backend-add-fs-path = The folder.
help-backend-login = Log in to Google Drive again.
help-grant = Choose which folders LemuSync may read and write.
help-grant-add = Allow a folder (an emulator's own folder).
help-grant-add-path = The folder.
help-grant-list = List the allowed folders.
help-players = Manage players.
help-players-add = Add a player.
help-players-add-name = The player's name.
help-players-list = List the players.
help-scan = Find emulators, accounts and saves on this device.
help-accounts = Connect players to emulator accounts on this device.
help-accounts-map = Use an emulator account for a player on this device.
help-accounts-map-player = The player's name.
help-accounts-map-emulator = The emulator id, for example cemu.
help-accounts-map-account = The account id, for example 80000002.
help-accounts-map-keep = Keep the account's current saves (they're uploaded, then you resolve a conflict).
help-accounts-map-replace = Replace the account's saves with the player's latest (the current ones go to local history first).
help-accounts-recover = Keep an orphaned save as a player's first version.
help-accounts-recover-emulator = The emulator id, for example cemu.
help-accounts-recover-account = The orphaned folder's account id.
help-accounts-recover-player = The player who gets this save.
help-accounts-create = Create a new emulator account for a player on this device, then restore their saves into it.
help-accounts-create-player = The player's name.
help-accounts-create-emulator = The emulator id, for example cemu.
help-accounts-create-name = The account's display name (default: the player's name, cut to fit).
help-accounts-create-yes = Don't ask for confirmation.
help-sync = Sync now.
help-sync-dry_run = Show what would happen without changing anything.
help-sync-yes = Confirm the first sync on this device.
help-status = Show this device's setup and saves.
help-history = Show a player's version history for a game.
help-history-player = The player (default: the only player in the repo).
help-history-emu = Only look at this emulator's games, for example cemu.
help-history-game = The game: an alias, its key or its name, for example botw or 101c9400.
help-restore = Put back a version or a local backup (the current files are backed up first).
help-restore-player = The player (default: the only player in the repo).
help-restore-emu = Only look at this emulator's games, for example cemu.
help-restore-game = The game: an alias, its key or its name, for example botw or 101c9400.
help-restore-what = A version id or a backup name from `lemusync history`.
help-restore-yes = Don't ask for confirmation.
help-doctor = Check the keystore, the storage and the emulator folders.

## Errors ($detail is never translated: paths, ids, provider text)
error-usage = { $detail }
error-config = The config file is invalid: { $detail }
error-backend = The storage returned an error. Try again; if it keeps happening, run `lemusync doctor`.
error-backend_auth = The storage login failed. Run `lemusync backend login`.
error-backend_offline = Can't reach your storage. Check your internet connection and try again. Your saves on this device are safe.
error-backend_quota = Your storage is full. Free up space or get more, then try again. Your saves on this device are safe.
error-details = Details: { $detail }
error-io = File problem: { $detail }
error-crypto = Encryption problem: { $detail }
error-unlock = Wrong passphrase or recovery key.
error-locked = Another LemuSync process is running ({ $detail }).
error-emulator_running = { $detail } is running. Close it and try again.
error-not_granted = LemuSync isn't allowed to use { $detail }. Run `lemusync grant add <folder>`.
error-not_set_up = Not set up yet: { $detail }
error-secrets = The system keyring failed: { $detail }
error-data = The repo has invalid data: { $detail }
error-needs_confirmation = This is the first sync on this device. Back up this device's saves first, then run `lemusync sync --yes`.
error-busy = { $detail } changed while it was being read. Try again when the game is idle.
error-injected = Test fault at { $detail }.
error-needs-choice = Use --keep or --replace (the account already has saves for: { $games }).
error-needs-tty = This needs a terminal to ask you: { $what }.

## Keyed errors: error-<code>-<key>. After `key:` the engine passes data ($detail: a path, id or name).
error-not_set_up-backend = No storage set up yet. Run `lemusync backend add fs <folder>`, `lemusync backend add s3 --endpoint <url> --bucket <name> --access-key-id <id>` or `lemusync backend add gdrive --client-id <id>`.
error-not_set_up-s3_key = The S3 secret key isn't in this device's system keyring. Run `lemusync backend add s3` again.
error-not_set_up-gdrive_login = Google Drive isn't logged in on this device. Run `lemusync backend login`.
error-not_set_up-gdrive_backend = Google Drive isn't the storage on this device. Run `lemusync backend add gdrive --client-id <id>`.
error-not_set_up-repo = This device has no repo yet. Run `lemusync init` or `lemusync join`.
error-not_set_up-repo_key = This device's repo key is missing from the system keyring. Run `lemusync join` again.
error-usage-repo_exists = The storage already has a repo. Run `lemusync join` instead.
error-usage-repo_race = Another device created a repo on the storage at the same time. Run `lemusync join` instead.
error-usage-already_joined = This device already uses another repo.
error-usage-passphrase_too_short = The passphrase needs at least 8 characters.
error-usage-player_name_invalid = A player name needs 1 to 32 visible characters.
error-usage-player_name_taken = That player name is taken.
error-usage-file_too_large = { $detail } is too large to sync.
error-usage-file_name_invalid = { $detail } has a file name that can't be synced.
error-usage-grant_path_missing = That folder doesn't exist.
error-usage-grant_path_not_allowed = That folder can't be allowed: choose the emulator's own folder, not / or your home folder.
error-usage-invalid_game_or_account = That game id or account id isn't valid.
error-usage-restore_reason_invalid = Invalid backup name: { $detail }
error-usage-unknown_player = There's no player { $detail }. Run `lemusync players add`.
error-usage-no_install = { $detail } wasn't found on this device. Run `lemusync scan`.
error-usage-no_account = This device has no emulator account { $detail }. Create it in the emulator, then run `lemusync scan`.
error-usage-account_taken = That account is already used by { $detail }.
error-usage-not_orphaned = { $detail } isn't an orphaned save on this device.
error-usage-already_has_save = The player already has a save for { $detail }. Recover it to a new player instead.
error-usage-folder_empty = That save folder is empty.
error-usage-not_mapped = The player has no { $detail } account on this device. Run `lemusync accounts map`.
error-usage-game_not_found = That game wasn't found on this device. Run `lemusync games` to see the games found, or set its path with `lemusync config set { $detail } <path>`.
error-usage-unknown_restore_point = { $detail } is neither a version nor a backup. See `lemusync history`.
error-usage-cannot_create = LemuSync can't create { $detail } accounts. Create it in the emulator, then run `lemusync accounts map`.
error-usage-too_many_accounts = The emulator already has its maximum of { $detail } accounts.
error-usage-account_exists = Something named { $detail } is already in the accounts folder. Nothing was changed.
error-usage-bad_account_name = That account name is empty or has characters the emulator can't store.
error-usage-no_free_account_id = There's no free account id left.
error-config-repo_mismatch = The repo on the storage isn't the one this device joined.
error-config-repo_config = The repo's player list is invalid.
error-config-no_home_dir = Your home folder couldn't be found.
error-config-invalid_flatpak_app_id = An emulator profile has an invalid Flatpak app id.
error-config-invalid_regex = An emulator profile has an invalid pattern.
error-config-unsupported_capture = An emulator profile uses an unsupported pattern.
error-secrets-bad_master_key = The repo key in the system keyring is damaged. Run `lemusync join` again.
error-secrets-no_default_collection = The system keyring has no default collection. LemuSync never creates one; set one up in your keyring app.
error-secrets-prompt_needed = The system keyring asked for a confirmation dialog. LemuSync never opens one; check your keyring app.
error-secrets-ambiguous = The system keyring has more than one LemuSync entry with the same name. Remove the extra one in your keyring app.
error-secrets-off_under_test_root = The system keyring is off while LEMUSYNC_TEST_ROOT is set.
error-secrets-bad_reply = The system keyring sent a reply LemuSync doesn't understand.
error-secrets_unavailable = The system keyring isn't available yet ({ $detail }). LemuSync will try again.
error-secrets_unavailable-no_owner = The system keyring isn't running yet. LemuSync never starts one; it will try again once your desktop session opens it.
error-secrets_unavailable-no_bus = There's no desktop session bus yet, so the system keyring can't be reached. LemuSync will try again.
error-secrets_unavailable-locked = The system keyring is locked. Unlock it (usually by logging in); LemuSync never unlocks it and will try again.
error-secrets_unavailable-gone = The system keyring closed while LemuSync was using it. LemuSync will try again.
error-data-repo_format_newer = The repo was made by a newer LemuSync. Update LemuSync on this device.
error-data-history_changed = A local history copy changed while it was being uploaded: { $detail }
error-data-blob_corrupt = A file in the repo is damaged ({ $detail }).
error-data-manifest_invalid = A version in the repo is invalid.
error-data-wrong_size = A file in the repo has the wrong size: { $detail }
error-data-file_too_large = A file in the repo is too large: { $detail }

## Prompts
prompt-passphrase = Passphrase:
prompt-passphrase-again = Type the passphrase again:
prompt-passphrases-differ = The passphrases don't match.
prompt-recovery = Recovery key:
prompt-client-secret = OAuth client secret (it isn't shown):
prompt-s3-secret = Secret access key (it isn't shown):
prompt-first-sync = This is the first sync on this device. Did you back up this device's saves?
prompt-keep-or-replace = Account { $account } already has saves for { $games }. Keep them (they're uploaded and you resolve a conflict) or replace them with { $player }'s latest save (the current files go to local history first)?
prompt-keep-or-replace-keys = [k]eep / [r]eplace:
prompt-restore = Replace { $player }'s current files with { $what }? The current files go to local history first.
prompt-create-account = Create { $emulator } account “{ $name }” for { $player } on this device? { $emulator } must be closed.
prompt-yes-no = [y/N]:
answer-yes = y
answer-keep = k
answer-replace = r

## Results
login-open-url = Open this address in your browser to let LemuSync use its own Google Drive folder:
login-waiting = Waiting for the browser (up to 5 minutes)…
login-done = Google Drive is connected.
backend-fs-done = The repo will be stored in { $path }.
backend-s3-done = The repo will be stored in bucket { $bucket } at { $endpoint }.
init-done = Repo { $repo } created on { $device }.
init-recovery = Your recovery key. Write it down now: it's shown only once, and without it or the passphrase your saves can't be recovered.
join-done = { $device } joined repo { $repo }.
grant-done = Allowed: { $path }
grant-none = No folders allowed yet.
players-added = Player { $name } added.
players-none = No players yet.
map-done = { $player } now uses { $emulator } account { $account } on this device.
recover-done = The orphaned save is now { $player }'s first { $game } version.
create-done = Created { $emulator } account “{ $name }” ({ $account }) for { $player }. Open { $emulator } and select “{ $name }” in Account settings.
restore-done = Restored { $what } for { $player }. Run `lemusync sync` to upload it.
sync-line = { $player } · { $game } · { $action }
sync-line-moved = { $player } · { $game } · { $action } · { $files ->
    [one] 1 file
   *[other] { $files } files
}, { $size } MB
sync-elapsed = Done in { $secs } s.
progress-checking = { $player } · { $game } · checking…
progress-uploading = { $player } · { $game } · uploading { $bar } { $done }/{ $total } · { $size_done }/{ $size_total } MB · { $rate } MB/s
progress-downloading = { $player } · { $game } · downloading { $bar } { $done }/{ $total } · { $size_done }/{ $size_total } MB · { $rate } MB/s
connection-trouble = Connection trouble, retrying…
connection-back = Connection is back.
dry-run = Dry run: nothing was changed.
sync-conflicts = { $count ->
    [one] One save has a conflict. Both versions are kept. Run `lemusync conflicts`.
   *[other] { $count } saves have a conflict. All versions are kept. Run `lemusync conflicts`.
}

action-nothing = up to date
action-adopt = already matches the repo
action-fast_forward = restored the latest version
action-upload = uploaded
action-upload_then_conflict = uploaded, conflict to resolve
action-conflict = conflict to resolve
action-local_missing = the save folder is missing
action-remote_behind = the repo went back in time; nothing changed
action-needs_account = no account on this device: run `lemusync accounts map`
action-skipped = skipped (the emulator is running)
action-resolved = conflict resolved; the other versions stay in history

warning-emulator_running = { $detail } is running, so its saves were skipped.
warning-not_granted = Not allowed yet: { $detail }. Run `lemusync grant add`.
warning-shared_folder_not_empty = { $detail } is shared by every account and isn't synced yet.
warning-orphaned_save = Orphaned save { $detail }: its account was deleted. Run `lemusync accounts recover` to keep it.
warning-config_merged = Changes from two devices were merged.
warning-player_renamed = Two players had the same name; one is now { $detail }.
warning-remote_behind = The repo is missing versions this device saw ({ $detail }). Nothing was changed.
warning-restore_rolled_back = An interrupted restore was undone in { $detail }.
warning-invalid_version = Ignored invalid data in the repo: { $detail }
warning-setup_incomplete = The repo was created, but this device isn't fully set up ({ $detail }). Run `lemusync join` with your passphrase to finish.
warning-unreadable_folder = A save folder couldn't be read safely and was skipped: { $detail }
warning-lease_renew_failed = Couldn't renew the lease ({ $detail }); retrying. The game keeps running.
warning-lease_release_failed = Couldn't release the lease ({ $detail }); it expires on its own.

scan-install = { $emulator } at { $base }
scan-needs-grant = Allow it: lemusync grant add { $path }
scan-account = account { $id } · { $name } · { $player }
scan-unknown = (none)
scan-save = save { $game }/{ $account } · { $files ->
    [one] 1 file
   *[other] { $files } files
}
scan-orphan = orphaned save { $game }/{ $account }: run `lemusync accounts recover`

status-device = Device: { $device }
status-repo = Repo: { $repo } on { $backend }
status-no-repo = No repo yet: run `lemusync init` or `lemusync join`.
status-stream = { $player } · { $emulator } { $game } · account { $account }

history-version = { $id } · { $files ->
    [one] 1 file
   *[other] { $files } files
} { $head }
history-head = (latest)
history-backup = local backup { $name }

doctor-ok = ok
doctor-fail = FAILED
doctor-check = { $status } { $name } { $detail }

help-play = Play a game: get the newest save, take the lease, start the emulator, upload when it closes.
help-play-emu = Only look at this emulator's games, for example cemu.
help-play-game = The game: an alias, its key or its name, for example botw or "breath of the wild".
help-play-player = The player (default: the only player in the repo).
help-play-args = Extra arguments for the emulator, after --.
help-conflicts = List saves with a conflict and their versions.
help-resolve = Resolve a conflict by keeping one version (the others stay in history).
help-resolve-game = The game: an alias, its key or its name, for example botw or 101c9400.
help-resolve-version = The version id to keep, from `lemusync conflicts`.
help-resolve-player = The player (needed when several players have a conflict on this game).
help-lease = Show or release leases (which device is playing what).
help-lease-status = Show every lease in the repo.
help-lease-release = Delete this device's lease on a game, for example one a crash left behind.
help-lease-release-force = Required: confirms you want to delete the lease.
help-lease-release-game = The game: an alias, its key or its name, for example botw or 101c9400.

play-pulling = Getting the newest save…
play-waiting = Another device is playing. Waiting for it to finish…
play-running = { $emulator } is running. Your save uploads when you close it.
play-uploading = { $emulator } closed. Uploading your save…
play-ctrl-c = The game is still running. Close it there; LemuSync then uploads your save.
play-done = { $emulator } exited with code { $code }.
prompt-lease-held = { $device } is playing this save (last seen { $ago } s ago).
prompt-lease-stale = { $device } was playing this save but stopped answering { $ago } s ago.
prompt-lease-keys = [w]ait / [p]lay anyway (a conflict if both save):
prompt-lease-keys-stale = [w]ait / [t]ake over / [p]lay anyway:
answer-wait = w
answer-play-anyway = p
answer-take-over = t
prompt-conflict = { $game } has a conflict. Which version do you want to play?
prompt-conflict-pick = Version number:
conflicts-none = No conflicts.
conflict-stream = { $player } · { $emulator } · { $game }
conflict-head = { $n }. { $version } · { $device } · { $files ->
    [one] 1 file
   *[other] { $files } files
}
resolve-done = Kept { $version } for { $player } · { $game }. The other versions stay in history.
lease-none = No leases.
lease-line = { $player } · { $emulator } · { $game } · { $device }, { $ago } s ago{ $ours ->
    [yes] {" "}(this device)
   *[no] {""}
}
lease-line-stale = { $player } · { $emulator } · { $game } · { $device }, expired { $ago } s ago{ $ours ->
    [yes] {" "}(this device)
   *[no] {""}
}
lease-released = { $count ->
    [0] This device has no lease on { $game }.
    [one] Released this device's lease on { $game }.
   *[other] Released { $count } leases of this device on { $game }.
}
error-lease_held = { $detail } is playing this save. Run `lemusync play` in a terminal to wait or play anyway.
error-needs_decision = { $detail } has a conflict. Run `lemusync conflicts`, then `lemusync resolve`.
error-usage-player_needed = Choose the player with --as <name>.
error-usage-version_invalid = { $detail } isn't a version id. Run `lemusync conflicts` to see them.
error-usage-no_conflict = { $detail } has no conflict.
error-usage-lease_live = That lease is live; it can't be taken over.

## config (config-and-cli §Commands)
help-config = Read and change this device's settings (config.toml).
help-config-path = Print where the config file is.
help-config-list = List every setting.
help-config-get = Show one setting.
help-config-get-key = The setting, for example conflict_policy or emulators.cemu.exe.
help-config-set = Change a setting. Comments in the file are kept.
help-config-set-key = The setting, for example leases.ttl_secs or emulators.cemu.games.101c9500.
help-config-set-values = The new value. emulators.<id>.exe takes several words: the command that starts the emulator.
help-config-unset = Remove a setting, back to its default.
help-config-unset-key = The setting.
config-unset-value = (not set)
config-set-done = { $key } = { $value }
config-unset-done = Removed { $key }.
error-usage-config_managed = That setting is changed with `{ $detail }`, not `lemusync config`.
error-usage-config_fixed = { $detail } can't be changed.
error-usage-config_unknown_key = There's no setting { $detail }. Run `lemusync config list`.
error-usage-config_unknown_emulator = There's no emulator { $detail }.
error-usage-config_bad_value = That isn't a valid value for { $detail }. Nothing was changed.
error-usage-config_one_value = { $detail } takes a single value.

## games
help-games = List the games installed on this device, with their names and paths. Local, no storage needed.
games-line = { $emulator } · { $name } · { $key } · { $path } · { $source }
games-none = No games found. Add your game folders in the emulator, or add one with `lemusync config set emulators.<id>.game_dirs <folder>`.
games-no-path = (no path)
games-source-config = from config
games-source-emulator = from the emulator
games-source-scan = from game_dirs
help-games-alias = Give a game a short name to type instead of its name or key.
help-games-alias-game = The game: its name, key or an alias.
help-games-alias-alias = The short name: 1 to 32 of a-z, 0-9, - and _.
help-games-alias-emu = Only look at this emulator's games, for example cemu.
help-games-unalias = Remove a game's short name.
help-games-unalias-alias = The short name.
games-line-aliases = { $line } · alias: { $aliases }
games-alias-done = { $alias } now means { $game }.
games-unalias-done = Removed the alias { $alias }.
error-usage-game_unknown = There's no game { $detail }. Run `lemusync games` to see the games on this device.
error-usage-game_ambiguous = More than one game matches: { $detail }. Add --emu <id>, type the game key, or give it a short name with `lemusync games alias <key> <alias>`.
error-usage-alias_invalid = { $detail } can't be an alias: use 1 to 32 of a-z, 0-9, - and _, starting with a letter or digit.
error-usage-alias_is_key = { $detail } is another game's key, so it can't be an alias.
error-usage-alias_taken = The alias { $detail } is already used by another game. Remove it first with `lemusync games unalias { $detail }`.
error-usage-alias_unknown = There's no alias { $detail }. Run `lemusync games` to see them.
error-usage-play_old_shape = The emulator now goes after --emu: { $detail }
history-game = { $game }
play-starting = Starting { $game }…

## Daemon: status texts (status-and-notifications §Status model)
status-synced = All saves synced
status-scanning = Checking for changes
status-uploading = { $count ->
    [one] Uploading 1 change
   *[other] Uploading { $count } changes
}
status-downloading = { $count ->
    [one] Downloading 1 change
   *[other] Downloading { $count } changes
}
status-syncing = { $count ->
    [one] Syncing 1 change
   *[other] Syncing { $count } changes
}
status-paused = Sync paused
status-offline = { $count ->
    [one] Offline · 1 change waiting
   *[other] Offline · { $count } changes waiting
}
status-conflicts = { $count ->
    [one] 1 conflict needs you
   *[other] { $count } conflicts need you
}
status-foreign-leases = { $count ->
    [one] 1 game is in use on another device
   *[other] { $count } games are in use on other devices
}
status-error-auth = Storage sign-in expired
status-error-quota = Storage is full
status-error-permission = LemuSync can't open a save folder
status-error-config = The config file has a problem
status-error-not-set-up = Not set up yet: run lemusync init or lemusync join
status-playing = Playing { $game } · { $player }

## Daemon: tray menu
tray-sync-now = Sync now
tray-pause = Pause
tray-pause-1h = For 1 hour
tray-pause-until-resume = Until I resume
tray-resume = Resume sync
tray-recent = Recent games
tray-quit = Quit

## Daemon: notifications
notice-conflict-title = Two devices changed { $game }
notice-conflict-body = Run lemusync conflicts to choose which save to keep.
notice-foreign-lease-title = { $game } is in use on { $device }
notice-foreign-lease-body = Choose Wait and LemuSync tells you when it's free.
notice-auth-body = Sign in to your storage again to keep syncing.
notice-quota-body = Free up space on your storage to keep syncing.
notice-action-wait = Wait
notice-remote-newer-title = Newer save of { $game } from { $device }
notice-remote-newer-body = Close { $emulator } and LemuSync updates it.

## service (systemd user unit on Linux)
help-service = Run the sync daemon in the background (systemd user service on Linux).
help-service-install = Install and start the daemon; it starts again at every login.
help-service-install-headless = Also keep syncing with nobody logged in (turns on linger). No tray or notifications then.
help-service-uninstall = Stop and remove the daemon's service.
help-service-start = Start the daemon.
help-service-stop = Stop the daemon.
help-service-status = Show whether the daemon is installed and running.
service-installed = The daemon is installed and running ({ $path }).
service-headless = It keeps syncing after you log out.
service-uninstalled = The daemon was stopped and removed.
service-linger-left = Linger was left as it was. To turn it off: loginctl disable-linger
service-not-installed = The daemon isn't installed. Run `lemusync service install`.
service-started = The daemon started.
service-stopped = The daemon stopped.
service-state = Daemon: { $enabled ->
    [yes] starts at login
   *[no] doesn't start at login
}, { $active ->
    [yes] running
   *[no] stopped
}, restarted { $restarts } times.
error-usage-service_unsupported = `lemusync service` works only on Linux for now.
error-usage-daemon_not_found = Can't find lemusyncd next to lemusync or on PATH.
error-usage-daemon_path_invalid = lemusyncd must be an existing file with an absolute path: { $detail }

## lemusyncd (the daemon)
daemon-already-running = LemuSync is already running for this user (another lemusyncd or lemusync command holds { $path }).
daemon-start-failed = lemusyncd couldn't start: { $error }

## Daemon: notifications for the tray's Play and Wait
notice-lease-free-title = { $game } is free to play
notice-lease-free-body = The other device let it go. Play it from the LemuSync menu.
notice-play-failed-title = Couldn't start { $game }

## CLI over the daemon (pause, resume, logs, status)
help-pause = Pause automatic sync in the daemon (until resume, or for a while).
help-pause-duration = How long, e.g. 30m, 1h, 1h30m (at most 30 days).
help-resume = Resume automatic sync in the daemon.
help-logs = Print the daemon's log.
help-logs-follow = Keep printing new lines.
pause-done = Sync paused until you run lemusync resume.
pause-done-for = { $minutes ->
    [one] Sync paused for 1 minute.
   *[other] Sync paused for { $minutes } minutes.
}
resume-done = Sync resumed.
status-daemon = Background service: { $state }
error-daemon = The background service failed: { $detail }
error-usage-daemon_not_running = The daemon isn't running, so there's nothing to pause or resume. Start it with `lemusync service start`.
error-usage-daemon_running_oauth = The Google Drive sign-in runs in this terminal, not in the daemon. Run `lemusync service stop`, sign in, then `lemusync service start`.
error-usage-duration_invalid = "{ $detail }" isn't a duration. Use e.g. 30m, 1h or 1h30m.
error-usage-no_log = The daemon hasn't written a log yet ({ $detail }).
error-usage-play_arg_not_utf8 = The daemon can only pass text arguments to the emulator: { $detail }
error-usage-path_not_utf8 = The daemon can only use text paths: { $detail }
