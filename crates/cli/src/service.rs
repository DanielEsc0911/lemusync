//! `lemusync service install | uninstall | start | stop | status`: the systemd user unit on
//! Linux (config-and-cli §Commands, reliability §Always on). Processes run with an argv
//! array through `Runner`, never a shell (B2). With `LEMUSYNC_TEST_ROOT` the unit goes under
//! `<root>/config/systemd/user/` and `RecordRunner` only appends argv to
//! `<root>/systemctl.log` (reading canned `show` output from `<root>/systemctl-show.txt`), so
//! tests never touch the real user manager.

use std::path::{Path, PathBuf};

use clap::Subcommand;
use fluent_bundle::FluentValue;
use lemusync_engine::EngineError;
use serde::Serialize;

use crate::i18n::{I18n, s};
use crate::output;

pub const UNIT: &str = "lemusyncd.service";
const DAEMON: &str = "lemusyncd";

#[derive(Subcommand)]
pub enum ServiceCmd {
    Install {
        #[arg(long)]
        headless: bool,
    },
    Uninstall,
    Start,
    Stop,
    Status,
}

/// Runs one process from an argv array and returns its stdout.
pub trait Runner {
    /// # Errors
    /// The process can't start or exits with a failure.
    fn run(&mut self, argv: &[&str]) -> Result<String, EngineError>;
}

/// The real thing: `std::process::Command`, no shell.
pub struct SystemRunner;

impl Runner for SystemRunner {
    fn run(&mut self, argv: &[&str]) -> Result<String, EngineError> {
        let (prog, rest) = argv
            .split_first()
            .ok_or_else(|| EngineError::Io("empty argv".into()))?;
        let out = std::process::Command::new(prog)
            .args(rest)
            .output()
            .map_err(|e| EngineError::Io(format!("{prog}: {e}")))?;
        if out.status.success() {
            Ok(String::from_utf8_lossy(&out.stdout).into_owned())
        } else {
            Err(EngineError::Io(format!(
                "{}: {}",
                argv.join(" "),
                String::from_utf8_lossy(&out.stderr).trim()
            )))
        }
    }
}

/// Test runner: records argv one line each; `show` answers from a canned file.
pub struct RecordRunner {
    pub root: PathBuf,
}

impl Runner for RecordRunner {
    fn run(&mut self, argv: &[&str]) -> Result<String, EngineError> {
        use std::io::Write;
        let mut log = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(self.root.join("systemctl.log"))?;
        writeln!(log, "{}", argv.join(" "))?;
        if argv.contains(&"show") {
            return Ok(
                std::fs::read_to_string(self.root.join("systemctl-show.txt")).unwrap_or_default(),
            );
        }
        Ok(String::new())
    }
}

/// `ExecStart` path, quoted for systemd: `"`, `\` escaped, `%` and `$` doubled.
fn exec_quote(path: &Path) -> Result<String, EngineError> {
    let text = path
        .to_str()
        .filter(|t| !t.chars().any(char::is_control))
        .ok_or_else(|| EngineError::Usage(format!("daemon_path_invalid:{}", path.display())))?;
    let mut q = String::from("\"");
    for c in text.chars() {
        match c {
            '"' | '\\' => {
                q.push('\\');
                q.push(c);
            }
            '%' => q.push_str("%%"),
            '$' => q.push_str("$$"),
            _ => q.push(c),
        }
    }
    q.push('"');
    Ok(q)
}

/// The unit file text. `daemon` must be absolute. The desktop unit belongs to the graphical
/// session: it starts after login and stops at logout, so lingering never starts it at boot,
/// before the desktop's keystore is up (desktop §Secret Service activation race). Only
/// `--headless`, which turns on linger on purpose, starts with `default.target`.
///
/// # Errors
/// A path systemd can't take (not UTF-8, control characters).
pub fn unit_text(daemon: &Path, headless: bool) -> Result<String, EngineError> {
    let exec = exec_quote(daemon)?;
    let (extra, session, target) = if headless {
        (" --headless", "", "default.target")
    } else {
        (
            "",
            "After=graphical-session.target\nPartOf=graphical-session.target\n",
            "graphical-session.target",
        )
    };
    Ok(format!(
        "# Written by `lemusync service install`. Remove with `lemusync service uninstall`.\n\
         [Unit]\n\
         Description=LemuSync save sync daemon\n\
         {session}\
         StartLimitIntervalSec=600\n\
         StartLimitBurst=5\n\
         \n\
         [Service]\n\
         Type=simple\n\
         ExecStart={exec}{extra}\n\
         Restart=on-failure\n\
         RestartSec=2\n\
         \n\
         [Install]\n\
         WantedBy={target}\n"
    ))
}

/// `lemusyncd`: next to the running `lemusync`, else the first on `PATH`. Must be absolute
/// and exist.
///
/// # Errors
/// `daemon_not_found` or `daemon_path_invalid`.
pub fn check_daemon(path: &Path) -> Result<PathBuf, EngineError> {
    if !path.is_absolute() || !path.is_file() {
        return Err(EngineError::Usage(format!(
            "daemon_path_invalid:{}",
            path.display()
        )));
    }
    Ok(path.to_owned())
}

fn find_daemon() -> Result<PathBuf, EngineError> {
    if let Some(given) = test_root().and(std::env::var_os("LEMUSYNC_TEST_DAEMON")) {
        return check_daemon(Path::new(&given));
    }
    let beside = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|d| d.join(DAEMON)));
    let on_path = std::env::var_os("PATH").and_then(|p| {
        std::env::split_paths(&p)
            .map(|d| d.join(DAEMON))
            .find(|c| c.is_absolute() && c.is_file())
    });
    match beside.filter(|b| b.is_file()).or(on_path) {
        Some(p) => check_daemon(&p),
        None => Err(EngineError::Usage("daemon_not_found".into())),
    }
}

fn test_root() -> Option<PathBuf> {
    std::env::var_os("LEMUSYNC_TEST_ROOT").map(PathBuf::from)
}

/// `~/.config/systemd/user/lemusyncd.service` (`XDG_CONFIG_HOME` when set and absolute).
fn unit_path() -> Result<PathBuf, EngineError> {
    if let Some(root) = test_root() {
        return Ok(root.join("config/systemd/user").join(UNIT));
    }
    let config = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .filter(|p| p.is_absolute())
                .map(|h| h.join(".config"))
        })
        .ok_or_else(|| EngineError::Io("no HOME".into()))?;
    Ok(config.join("systemd/user").join(UNIT))
}

/// What `systemctl --user show` says.
#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct ServiceStatus {
    pub api_version: u32,
    pub installed: bool,
    pub enabled: bool,
    pub active: bool,
    pub restarts: u32,
}

/// Parse `show -p LoadState,UnitFileState,ActiveState,NRestarts` output.
#[must_use]
pub fn parse_show(text: &str) -> ServiceStatus {
    let get = |k: &str| {
        text.lines()
            .find_map(|l| l.strip_prefix(k)?.strip_prefix('='))
            .unwrap_or_default()
            .trim()
    };
    ServiceStatus {
        api_version: lemusync_api::API_VERSION,
        installed: get("LoadState") == "loaded",
        enabled: get("UnitFileState") == "enabled",
        active: matches!(get("ActiveState"), "active" | "activating" | "reloading"),
        restarts: get("NRestarts").parse().unwrap_or(0),
    }
}

const SYSTEMCTL: [&str; 2] = ["systemctl", "--user"];

fn systemctl(r: &mut dyn Runner, args: &[&str]) -> Result<String, EngineError> {
    let full: Vec<&str> = SYSTEMCTL.iter().chain(args).copied().collect();
    r.run(&full)
}

/// Removes every `<dir>/*.wants/lemusyncd.service` (and `*.requires/`) symlink next to the unit.
/// `systemctl disable` only removes the links the unit's *current* `[Install]` names, so a
/// unit rewritten with a new target would leave the old link (e.g. `default.target.wants`)
/// and keep starting at boot. Returns whether any link was removed.
///
/// # Errors
/// A link that can't be removed.
pub fn remove_links(unit: &Path) -> Result<bool, EngineError> {
    let Some(dir) = unit.parent() else {
        return Ok(false);
    };
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Ok(false);
    };
    let mut removed = false;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !(name.ends_with(".wants") || name.ends_with(".requires")) {
            continue;
        }
        let link = entry.path().join(UNIT);
        if link
            .symlink_metadata()
            .is_ok_and(|m| m.file_type().is_symlink())
        {
            std::fs::remove_file(&link)?;
            removed = true;
        }
    }
    Ok(removed)
}

/// Write the unit, reload, enable and start it; `headless` also turns on linger. Links left
/// by an earlier install go first, so only the new target starts it.
///
/// # Errors
/// Write or process failures.
pub fn install(
    r: &mut dyn Runner,
    unit: &Path,
    daemon: &Path,
    headless: bool,
) -> Result<(), EngineError> {
    let text = unit_text(daemon, headless)?;
    remove_links(unit)?;
    lemusync_engine::fsx::write_atomic(unit, text.as_bytes())?;
    systemctl(r, &["daemon-reload"])?;
    systemctl(r, &["enable", "--now", UNIT])?;
    if headless {
        r.run(&["loginctl", "enable-linger"])?;
    }
    Ok(())
}

/// Everything `install` added: disable and stop, remove every `.wants` link and the file,
/// reload. Linger is never changed. `false` when nothing was installed.
///
/// # Errors
/// Remove or process failures.
pub fn uninstall(r: &mut dyn Runner, unit: &Path) -> Result<bool, EngineError> {
    if !unit.exists() {
        if remove_links(unit)? {
            systemctl(r, &["daemon-reload"])?;
            return Ok(true);
        }
        return Ok(false);
    }
    systemctl(r, &["disable", "--now", UNIT])?;
    remove_links(unit)?;
    std::fs::remove_file(unit)?;
    systemctl(r, &["daemon-reload"])?;
    Ok(true)
}

/// # Errors
/// The process fails.
pub fn status(r: &mut dyn Runner) -> Result<ServiceStatus, EngineError> {
    let out = systemctl(
        r,
        &[
            "show",
            "-p",
            "LoadState,UnitFileState,ActiveState,NRestarts",
            UNIT,
        ],
    )?;
    Ok(parse_show(&out))
}

/// The unit's state for `doctor`, through the same runner as `service status`. `None` off
/// Linux or when `systemctl` can't be asked.
#[must_use]
pub fn current() -> Option<ServiceStatus> {
    if !cfg!(target_os = "linux") {
        return None;
    }
    let mut runner: Box<dyn Runner> = match test_root() {
        Some(root) => Box::new(RecordRunner { root }),
        None => Box::new(SystemRunner),
    };
    status(runner.as_mut()).ok()
}

/// Where systemd keeps the linger flags (`<root>/linger` in tests).
#[must_use]
pub fn linger_dir() -> PathBuf {
    test_root().map_or_else(
        || PathBuf::from("/var/lib/systemd/linger"),
        |r| r.join("linger"),
    )
}

/// True in tests (`LEMUSYNC_TEST_ROOT`): nothing outside the root is asked.
#[cfg(target_os = "linux")]
#[must_use]
pub fn under_test() -> bool {
    test_root().is_some()
}

fn yes(b: bool) -> FluentValue<'static> {
    s(if b { "yes" } else { "no" })
}

/// The `service` subcommand.
///
/// # Errors
/// `service_unsupported` off Linux; otherwise as each step.
pub fn run(cmd: &ServiceCmd, t: &I18n, json: bool) -> Result<(), EngineError> {
    if !cfg!(target_os = "linux") {
        return Err(EngineError::Usage("service_unsupported".into()));
    }
    let mut runner: Box<dyn Runner> = match test_root() {
        Some(root) => Box::new(RecordRunner { root }),
        None => Box::new(SystemRunner),
    };
    let r = runner.as_mut();
    let unit = unit_path()?;
    let done = |key: &str, args: &[(&str, FluentValue<'static>)]| {
        if json {
            output::json(&serde_json::json!({
                "api_version": lemusync_api::API_VERSION,
                "done": key.trim_start_matches("service-"),
            }));
        } else {
            output::line(t, key, args);
        }
    };
    match cmd {
        ServiceCmd::Install { headless } => {
            let daemon = find_daemon()?;
            install(r, &unit, &daemon, *headless)?;
            done(
                "service-installed",
                &[("path", s(unit.display().to_string()))],
            );
            if *headless && !json {
                output::line(t, "service-headless", &[]);
            }
        }
        ServiceCmd::Uninstall => {
            let removed = uninstall(r, &unit)?;
            done(
                if removed {
                    "service-uninstalled"
                } else {
                    "service-not-installed"
                },
                &[],
            );
            if removed && !json {
                output::line(t, "service-linger-left", &[]);
            }
        }
        ServiceCmd::Start => {
            systemctl(r, &["start", UNIT])?;
            done("service-started", &[]);
        }
        ServiceCmd::Stop => {
            systemctl(r, &["stop", UNIT])?;
            done("service-stopped", &[]);
        }
        ServiceCmd::Status => {
            let st = status(r)?;
            if json {
                output::json(&st);
            } else if st.installed {
                output::line(
                    t,
                    "service-state",
                    &[
                        ("enabled", yes(st.enabled)),
                        ("active", yes(st.active)),
                        ("restarts", FluentValue::from(st.restarts)),
                    ],
                );
            } else {
                output::line(t, "service-not-installed", &[]);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOLDEN: &str =
        "# Written by `lemusync service install`. Remove with `lemusync service uninstall`.
[Unit]
Description=LemuSync save sync daemon
After=graphical-session.target
PartOf=graphical-session.target
StartLimitIntervalSec=600
StartLimitBurst=5

[Service]
Type=simple
ExecStart=\"/opt/lemu sync/lemusyncd\"
Restart=on-failure
RestartSec=2

[Install]
WantedBy=graphical-session.target
";

    #[test]
    fn golden_unit_normal_and_headless() {
        let p = Path::new("/opt/lemu sync/lemusyncd");
        assert_eq!(unit_text(p, false).unwrap(), GOLDEN);
        // Headless lingers on purpose, so it alone starts with `default.target`.
        assert_eq!(
            unit_text(p, true).unwrap(),
            GOLDEN
                .replace("lemusyncd\"\n", "lemusyncd\" --headless\n")
                .replace(
                    "After=graphical-session.target\nPartOf=graphical-session.target\n",
                    ""
                )
                .replace(
                    "WantedBy=graphical-session.target",
                    "WantedBy=default.target"
                )
        );
    }

    #[test]
    fn exec_path_is_escaped() {
        assert_eq!(
            exec_quote(Path::new("/a/%h$x\"\\")).unwrap(),
            "\"/a/%%h$$x\\\"\\\\\""
        );
        assert!(exec_quote(Path::new("/a\nb")).is_err());
    }

    #[test]
    fn relative_or_missing_daemon_refused() {
        assert!(check_daemon(Path::new("lemusyncd")).is_err());
        assert!(check_daemon(Path::new("/no/such/lemusyncd")).is_err());
    }

    #[cfg(unix)]
    struct Rec(Vec<String>);
    #[cfg(unix)]
    impl Runner for Rec {
        fn run(&mut self, argv: &[&str]) -> Result<String, EngineError> {
            self.0.push(argv.join(" "));
            Ok("LoadState=loaded\nUnitFileState=enabled\nActiveState=active\nNRestarts=3\n".into())
        }
    }

    // Symlinks need Unix; the unit and its links only exist on Linux.
    #[cfg(unix)]
    #[test]
    fn install_then_uninstall_order() {
        let dir = tempfile::tempdir().unwrap();
        let user = dir.path().join("systemd/user");
        let unit = user.join(UNIT);
        let link = |target: &str| {
            let d = user.join(format!("{target}.wants"));
            std::fs::create_dir_all(&d).unwrap();
            std::os::unix::fs::symlink(&unit, d.join(UNIT)).unwrap();
            d.join(UNIT)
        };
        // An older install's `default.target` link would keep starting it at boot.
        let old = link("default.target");
        let other = user.join("default.target.wants/other.service");
        std::os::unix::fs::symlink("/x", &other).unwrap();
        let mut r = Rec(Vec::new());
        install(&mut r, &unit, Path::new("/usr/bin/lemusyncd"), true).unwrap();
        assert!(unit.is_file());
        assert!(old.symlink_metadata().is_err(), "stale link kept");
        // `enable` made this one (the recording runner doesn't).
        let new = link("graphical-session.target");
        assert!(uninstall(&mut r, &unit).unwrap());
        assert!(!unit.exists());
        assert!(new.symlink_metadata().is_err(), "link left after uninstall");
        assert!(
            other.symlink_metadata().is_ok(),
            "another unit's link removed"
        );
        assert!(!uninstall(&mut r, &unit).unwrap());
        // A link with no unit file is still cleaned up.
        let orphan = link("graphical-session.target");
        assert!(uninstall(&mut r, &unit).unwrap());
        assert!(orphan.symlink_metadata().is_err());
        assert_eq!(
            r.0,
            [
                "systemctl --user daemon-reload",
                "systemctl --user enable --now lemusyncd.service",
                "loginctl enable-linger",
                "systemctl --user disable --now lemusyncd.service",
                "systemctl --user daemon-reload",
                "systemctl --user daemon-reload",
            ]
        );
        assert_eq!(
            status(&mut r).unwrap(),
            ServiceStatus {
                api_version: 1,
                installed: true,
                enabled: true,
                active: true,
                restarts: 3
            }
        );
        assert!(!parse_show("LoadState=not-found\n").installed);
    }
}
