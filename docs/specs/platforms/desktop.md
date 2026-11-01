# Desktop: Windows, macOS, Linux

> **Status:** Draft · **Related:** [rules](../../rules.md) (B3, P4), [architecture §Process model](../architecture.md#process-model), [reliability](../reliability.md#always-on-247), [status-and-notifications](../ui/status-and-notifications.md#desktop-tray-and-menu-bar), [emulator-profiles](../emulator-profiles.md), [security](../security.md) · **Code:** `crates/daemon` (`lemusyncd`: `src/main.rs`, run loop `src/run.rs`, logs `src/logging.rs`; Linux tray, notifications, power: `crates/daemon/src/platform/linux/`; systemd user unit: `crates/cli/src/service.rs`; other OSes no-op until Phase 2 Task 11), `packaging/` (planned)

## Purpose
Desktop is where most emulators run. The daemon gives true 24/7 sync with native tray and notifications. The Flutter app is a client of it.

## Matrix
| | Windows 10/11 | macOS | Linux |
|---|---|---|---|
| Arch | x64, arm64 | Apple Silicon + Intel (universal) | x86_64, arm64 |
| Service | Task Scheduler logon task | LaunchAgent via `SMAppService` | systemd user unit |
| Tray | Notification area | Menu bar extra | StatusNotifierItem / AppIndicator |
| Notifications | Toasts with actions | `UNUserNotificationCenter` | freedesktop notifications with actions |
| Sleep inhibition | `SetThreadExecutionState` | IOPM assertion | logind `Inhibit` |
| Process detection | Process list | Process list | `/proc` (Flatpak apps visible too) |
| Keystore | Credential Manager | Keychain | Secret Service |
| Packaging (now, [release §Artifacts](../release.md#artifacts)) | `.msi`, `.exe` installer | `.dmg` (ad-hoc signed until a Developer ID exists) | `.deb`, `.rpm`, `.AppImage`, `.tar.gz` |
| Packaging (later) | winget | Homebrew cask, notarised DMG | Flatpak, AUR |

## Quirks
- **Windows toasts** need an AppUserModelID registered by the installer (Start Menu shortcut). Portable mode falls back to tray balloons (Verify SP2).
- **macOS notifications** need an app identity, so `lemusyncd` ships inside the app bundle as a helper and is registered as a LaunchAgent from there. Distribute outside the Mac App Store, because sandboxing would block reading other apps' save folders.
- **macOS privacy (TCC):** `~/Documents` (e.g., RetroArch's default) triggers a permission prompt. Other apps' sandbox containers trigger the "access data from other apps" prompt. The grant flow explains this before the OS asks.
- **Linux GNOME:** no tray without the AppIndicator extension (see [status](../ui/status-and-notifications.md#desktop-tray-and-menu-bar)). XFCE and KDE show StatusNotifier icons natively. Test machines: [testing §Device lab](../testing.md#device-lab).
- **Flatpak packaging vs. access:** a sandboxed LemuSync needs filesystem permissions for emulator folders, including other Flatpaks' `~/.var/app/<id>/`. Spike SP2 decides between narrow static permissions, the document portal, or a native daemon package (Verify).
- **Steam Deck (Game Mode):** no tray is visible, but the daemon still runs. A Decky plugin is a later idea.
- **Portable emulators:** profiles use `{exe_dir}`. The user picks the executable in launcher setup.

## Linux results (SP2, XFCE)
Measured 2026-10-08 on Fedora 44, XFCE 4.20 on X11 (`xfce4-panel` 4.20.7, `xfce4-notifyd` 0.9.7, systemd 259, KWallet `ksecretd` from `kf6-kwallet` 6.30.0). Throwaway prototype, crates [`ksni` 0.3.6](https://docs.rs/ksni/0.3.6), [`notify-rust` 4.18.2](https://docs.rs/notify-rust/4.18.2), [`zbus` 5.19.0](https://docs.rs/zbus/5.19.0), [`keyring` 3.6.3](https://docs.rs/keyring/3.6.3), tokio 1.53.2.

- **Tray:** `ksni` (pure Rust StatusNotifierItem over D-Bus, no GTK) was chosen over `tray-icon`, which needs a GTK main loop and libappindicator on Linux. The item registered with `org.kde.StatusNotifierWatcher` (served by XFCE's "Status Tray Plugin", `libsystray.so`, panel plugin `systray`) as `org.kde.StatusNotifierItem-<pid>-1/StatusNotifierItem`. The icon and tooltip swapped idle → syncing → error every 3 s and screenshots of the panel showed each icon. A menu with 3 items plus a separator was exposed. XFCE needs nothing beyond its default systray plugin; it remembers the item in `/plugins/plugin-N/known-items`.
- **Started by systemd:** the same tray registered when the prototype ran as a systemd user unit. The unit inherits `DISPLAY`, `DBUS_SESSION_BUS_ADDRESS` and `XDG_*` because Fedora's `/etc/X11/xinit/xinitrc.d/50-systemd-user.sh` runs `systemctl --user import-environment DISPLAY XAUTHORITY` and `dbus-update-activation-environment --systemd` at X login. Without a watcher on the bus (private `dbus-run-session`), `ksni` fails with `Watcher(ServiceUnknown(...))`, so the daemon must treat the tray as optional and retry when a watcher appears.
- **Unit target:** `graphical-session.target` stays **inactive** under XFCE (it is only reached by sessions that start it, e.g. GNOME and KDE), so `WantedBy=graphical-session.target` would never start the daemon there. SP2 therefore chose `WantedBy=default.target`, with the tray created lazily. **Superseded for the desktop unit by [D39](../../decisions.md#d39-the-desktop-unit-starts-with-the-graphical-session-2026-10-29)** after the [activation race below](#secret-service-activation-race-under-linger). `Restart=on-failure` restarted the unit 2 s after `kill -9` (`NRestarts=1`, new PID, tray re-registered).
- **Linger:** `loginctl enable-linger` works for the user without a polkit prompt and reports `Linger=yes`. It starts the user manager (and `default.target` units) at boot without a login, so sync runs headless, with no tray and no notifications until a graphical session imports its environment. With a `default.target` desktop unit that also meant running before the desktop's keystore existed ([below](#secret-service-activation-race-under-linger)).
- **Notifications:** `GetServerInformation` → `Xfce Notify Daemon`, `0.9.7`, spec `1.2`. `GetCapabilities` → `action-icons actions body body-hyperlinks body-markup icon-static sound x-canonical-private-icon-only` (no `persistence`). A notification with 2 actions was shown. Without a click it expired and `notify-rust` reported `__closed`. Clicking "Keep this device" (manual run of `sp2-spike notify`) printed `notify: id=175` then `notify: action=keep-local`, so `ActionInvoked` reaches the daemon.
- **Desktop idle suspend ignores logind `idle` inhibitors:** `xfce4-power-manager` 4.20.0 (Fedora 44 `xfce4-power-manager-4.20.0-4.fc44`) runs its own idle-sleep timer and checks only its own `org.freedesktop.PowerManagement.Inhibit` list (log string "Idle sleep alarm timeout, but power manager is currently inhibited, action ignored"). The binary never reads logind's `BlockInhibited`. Live check: while `systemd-inhibit --what=idle --mode=block` was held, logind's `BlockInhibited` contained `idle` but xfpm's `HasInhibit` stayed `false`. See [reliability §Sleep and resume](../reliability.md#sleep-and-resume).

- **Re-check on KDE Plasma (Phase 2 Task 6, 2026-10-28):** the same PC now runs Plasma 6.7.5 on Wayland (`plasma-workspace` 6.7.5, `powerdevil` 6.7.5, systemd 259.10), with `ksni` 0.3.7, `notify-rust` 4.18.2, `zbus` 5.19.0 from `lemusyncd`'s own code (`cargo run -p lemusync-daemon --example desktop_check`). The item registered as `org.kde.StatusNotifierItem-<pid>-1/StatusNotifierItem` and its `IconName`/`ToolTip` followed each state. `GetServerInformation` → `Plasma`, `KDE`, `6.7.5`, spec `1.2`; `dbus-monitor` showed the `Notify` call with actions `wait`/`Esperar`, `open`/`Abrir` and urgency 2 (clicking was not tested in this run). `systemd-inhibit --list` showed `LemuSync … idle … block` while the guard lived and `LemuSync … sleep … Checkpoint saves before suspend … delay` while running; both were gone after exit. Here `org.freedesktop.PowerManagement.Inhibit` is served by `org_kde_powerdevil`: `Inhibit` returned a cookie and `UnInhibit` released it on drop (its `HasInhibit` stayed `false` during the 5 s hold).

- **`lemusyncd` smoke run (Phase 2 Task 8, 2026-10-28, same KDE Plasma 6.7.5 Wayland PC):** a release `lemusyncd` under `LEMUSYNC_TEST_ROOT` (no repo) bound its socket `0600` in a `0700` dir, answered `status`, `pause` and `subscribe` over IPC, registered its `StatusNotifierItem` and held the `sleep` delay lock (`systemd-inhibit --list`: `LemuSync … lemusyncd sleep … Checkpoint saves before suspend … delay`). RSS 14 MB at idle start. `SIGTERM` → exit 0, socket gone, log ends with `lemusyncd stopped`. The real session (service install, Cemu played by hand) is Task 8 Step 3.

## Secret Service activation race under linger
**What happened (2026-10-29, the maintainer's Fedora 44 KDE Plasma PC, from the journal):**
1. Linger was on for the user, so the user manager started at boot and with it `lemusyncd.service`, then `WantedBy=default.target` (written by `lemusync service install`).
2. At 19:38:12, before Plasma was up, the daemon made its first keystore call through `keyring`. Nothing owned `org.freedesktop.secrets` yet, and the call allowed auto-start, so D-Bus activated `gnome-keyring-daemon`, the only program with an activation file for that name in `/usr/share/dbus-1/services/`.
3. At login KWallet's `ksecretd` (the real store here: `kdewallet`, unlocked by PAM) could not take the name. Another app (Proton VPN) then got GNOME Keyring's "create new keyring password" prompt, and a new `~/.local/share/keyrings/Default.keyring` was created.

The SP2 notes ([above](#linux-results-sp2-xfce), [security §Credentials](../security.md#credentials)) came from XFCE without linger, where the session's own store was always up before the daemon, so the race never showed.

**Why it happened:** D-Bus activation is first come, first served. Whichever program is started for a well-known name first keeps it, and the session's real store, started later by the desktop, loses. A user unit that runs before login can therefore pick the session's secret store for the user. The fix removes both halves: the daemon no longer runs before the session, and it can no longer cause an activation at all.

**Fix:**
- The desktop unit belongs to the graphical session: `After=graphical-session.target`, `PartOf=graphical-session.target`, `WantedBy=graphical-session.target` ([D39](../../decisions.md#d39-the-desktop-unit-starts-with-the-graphical-session-2026-10-29)). Linger no longer starts it at boot, and it stops at logout. `service install --headless`, which turns linger on on purpose, keeps `default.target`; it is safe now because of the next point.
- The keystore client never starts, unlocks or creates a store ([D38](../../decisions.md#d38-the-linux-keystore-client-never-starts-unlocks-or-creates-a-store-2026-10-29), [security §Secret Service activation](../security.md#secret-service-activation)). The daemon doesn't touch secrets at startup; the first read happens when a step needs one, and "no owner yet" or "locked" is a wait retried with backoff, never a fatal error.
- `service install` and `uninstall` remove every `*.wants/lemusyncd.service` (and `*.requires/`) symlink next to the unit, because `systemctl disable` only removes the links named by the unit file's *current* `[Install]`. A unit rewritten from `default.target` to `graphical-session.target` would otherwise keep its old boot link. `uninstall` then removes the file and runs `daemon-reload`; it never changes linger.

**Rule:** LemuSync never starts or creates a secret store, and never decides which program provides one. It uses whatever owns `org.freedesktop.secrets` in the user's session.

**Consequence on XFCE:** `graphical-session.target` stays inactive there (measured in SP2), so the desktop unit doesn't start by itself; `lemusync service start` starts it for the session (V-DESK-6).

## Verify
- **V-DESK-6** Starting the desktop unit on XFCE, where `graphical-session.target` stays inactive (SP2). Find out whether XFCE 4.20's session can reach the target, or whether an XDG autostart entry that runs `systemctl --user start lemusyncd.service` is needed. Until then XFCE users run `lemusync service start` after login.
- **V-DESK-1** Tray + actionable notifications from the daemon on Windows and macOS (SP2). Linux XFCE is done ([above](#linux-results-sp2-xfce)). Windows and macOS deferred by the maintainer on 2026-10-08.
- **V-DESK-2** Flatpak permission strategy (SP2). Research so far: inside the sandbox an app sees only its own `~/.var/app/$FLATPAK_ID`, and `--filesystem=home` excludes `~/.var/app`, which must be requested per path ([Flatpak sandbox permissions](https://docs.flatpak.org/en/latest/sandbox-permissions.html)). Flathub wants static permissions "kept to an absolute minimum" and makes a portal mandatory when one covers the use case ([Flathub requirements](https://docs.flathub.org/docs/for-app-authors/requirements)). No test Flatpak was built. The decision entry is left to the maintainer.
- **V-DESK-3** `SMAppService` agent from a notarised bundle with a Rust helper (SP2).
- **V-DESK-4** Flutter Windows arm64 build support (SP1).
- **V-DESK-5** Flutter Linux desktop above 60 fps (SP1). On CachyOS (KDE Plasma 6.8 Wayland, 144 Hz) the probe draws about 61 fps under both the Wayland and X11 GDK backends ([architecture §Bridge](../architecture.md#bridge)). Find out whether the GTK embedder caps frame pacing at 60 Hz, and whether a setting or a newer Flutter release lifts it. Until then the 120 fps budget ([design-system](../ui/design-system.md#performance)) is not met on Linux.
