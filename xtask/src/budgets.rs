//! Daemon budgets (docs/specs/reliability.md §Budgets).
//!
//! `budgets [idle-secs]` (Linux, manual): runs the release `lemusyncd --headless` fully isolated
//! (every location under a tempdir, one fake Cemu install so the process poll has names to look
//! for, no repo so the OS keystore is never read) and prints cold start to "IPC ready", RSS and
//! idle CPU from `/proc/<pid>/stat` over `idle-secs` (default 600).
//!
//! `budgets-check` (CI): fails when the release `lemusyncd` size or the engine bench's warm
//! rescan time is more than 20 % over the reference values in the spec's "CI reference" table.

use std::path::Path;
#[cfg(target_os = "linux")]
use std::path::PathBuf;
#[cfg(target_os = "linux")]
use std::process::{Command, Stdio};
use std::time::Duration;
#[cfg(target_os = "linux")]
use std::time::Instant;

const SPEC: &str = "docs/specs/reliability.md";
/// Allowed growth over the reference before the check fails.
const SLACK: f64 = 1.20;

pub(crate) fn check(root: &Path) -> Result<(), Vec<String>> {
    let spec =
        std::fs::read_to_string(root.join(SPEC)).map_err(|e| vec![format!("{SPEC}: {e}")])?;
    let size = std::fs::metadata(root.join("target/release/lemusyncd"))
        .map_err(|e| {
            vec![format!(
                "target/release/lemusyncd (build it --release first): {e}"
            )]
        })?
        .len();
    let rescan_ns = rescan_ns(root)?;
    #[allow(clippy::cast_precision_loss)] // sizes and times far below 2^52
    let measured = [
        ("lemusyncd_bytes", size as f64),
        ("rescan_10k_ms", rescan_ns / 1e6),
    ];
    let mut errors = Vec::new();
    for (key, value) in measured {
        match reference(&spec, key) {
            Some(r) => {
                println!("{key}: {value:.1} (reference {r}, limit {:.1})", r * SLACK);
                if let Some(e) = compare(key, value, r) {
                    errors.push(e);
                }
            }
            None => errors.push(format!("{SPEC}: no CI reference row for `{key}`")),
        }
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// The mean of the last `rescan_10k_files_warm` criterion run, in nanoseconds.
fn rescan_ns(root: &Path) -> Result<f64, Vec<String>> {
    let path = root.join("target/criterion/rescan_10k_files_warm/new/estimates.json");
    let text = std::fs::read_to_string(&path).map_err(|e| {
        vec![format!(
            "{} (run `cargo bench -p lemusync-engine --bench budgets` first): {e}",
            path.display()
        )]
    })?;
    let json: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| vec![format!("{}: {e}", path.display())])?;
    json["mean"]["point_estimate"]
        .as_f64()
        .ok_or_else(|| vec![format!("{}: no mean.point_estimate", path.display())])
}

/// The value of a "CI reference" table row: `` | `key` | 18 230 232 | … ``.
fn reference(spec: &str, key: &str) -> Option<f64> {
    let cell = format!("`{key}`");
    spec.lines().find_map(|line| {
        let mut cells = line.split('|').map(str::trim).filter(|c| !c.is_empty());
        if cells.next()? != cell {
            return None;
        }
        let digits: String = cells
            .next()?
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        digits.parse().ok()
    })
}

fn compare(key: &str, value: f64, reference: f64) -> Option<String> {
    (value > reference * SLACK).then(|| {
        format!(
            "{key} regressed: {value:.1} is more than 20 % over the reference {reference} ({SPEC})"
        )
    })
}

pub(crate) fn measure(root: &Path, idle_secs: &str) -> Result<(), Vec<String>> {
    let idle: u64 = if idle_secs.is_empty() {
        600
    } else {
        idle_secs
            .parse()
            .map_err(|_| vec![format!("idle seconds `{idle_secs}` is not a number")])?
    };
    run(root, Duration::from_secs(idle)).map_err(|e| vec![e])
}

#[cfg(not(target_os = "linux"))]
fn run(_: &Path, _: Duration) -> Result<(), String> {
    Err("budgets measures /proc and runs on Linux only".into())
}

#[cfg(target_os = "linux")]
fn run(root: &Path, idle: Duration) -> Result<(), String> {
    let bin = root.join("target/release");
    for b in ["lemusyncd", "lemusync"] {
        if !bin.join(b).exists() {
            return Err(format!(
                "target/release/{b} missing: cargo build --release -p lemusync-daemon -p lemusync-cli"
            ));
        }
    }
    let tmp = Tmp::new()?;
    let t = tmp.0.as_path();
    // One fake Cemu install (empty folders), granted, so the poller walks /proc for its names.
    for d in [
        "home/.local/share/Cemu/mlc01/usr/save",
        "home/.config/Cemu",
        "run",
    ] {
        std::fs::create_dir_all(t.join(d)).map_err(|e| e.to_string())?;
    }
    for d in ["home/.local/share/Cemu", "home/.config/Cemu"] {
        let ok = isolated(&bin.join("lemusync"), t)
            .args(["grant", "add"])
            .arg(t.join(d))
            .stdout(Stdio::null())
            .status()
            .map_err(|e| e.to_string())?
            .success();
        if !ok {
            return Err("lemusync grant add failed".into());
        }
    }
    let socket = t.join("run/daemon.sock");

    let mut starts = Vec::new();
    for _ in 0..5 {
        let (child, ready) = start(&bin, t, &socket)?;
        starts.push(ready);
        stop(child)?;
    }
    starts.sort();
    println!(
        "cold start to IPC ready: median {:.1} ms, max {:.1} ms (5 runs)",
        ms(starts[2]),
        ms(starts[4])
    );

    let (child, _) = start(&bin, t, &socket)?;
    let pid = child.id();
    std::thread::sleep(Duration::from_secs(5)); // first sync attempt and poll settle
    let (rss0, _) = memory(pid)?;
    let cpu0 = cpu_ticks(pid)?;
    println!("idle for {} s …", idle.as_secs());
    std::thread::sleep(idle);
    let cpu1 = cpu_ticks(pid)?;
    let (rss1, hwm) = memory(pid)?;
    stop(child)?;
    let ticks = cpu1 - cpu0;
    #[allow(clippy::cast_precision_loss)] // tick counts are small
    let pct = ticks as f64 / 100.0 / idle.as_secs_f64() * 100.0;
    println!(
        "idle CPU: {ticks} ticks (utime+stime, 100 Hz) over {} s = {pct:.4} %",
        idle.as_secs()
    );
    println!(
        "RSS: {:.1} MiB after start, {:.1} MiB after idle, peak {:.1} MiB",
        kib_mib(rss0),
        kib_mib(rss1),
        kib_mib(hwm)
    );
    let size = std::fs::metadata(bin.join("lemusyncd"))
        .map_err(|e| e.to_string())?
        .len();
    println!("lemusyncd size: {size} bytes");
    Ok(())
}

/// A command whose home, XDG folders and `LemuSync` locations are all under `t`.
#[cfg(target_os = "linux")]
fn isolated(bin: &Path, t: &Path) -> Command {
    let mut c = Command::new(bin);
    c.env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", t.join("home"))
        .env("XDG_CONFIG_HOME", t.join("home/.config"))
        .env("XDG_DATA_HOME", t.join("home/.local/share"))
        .env("XDG_STATE_HOME", t.join("home/.local/state"))
        .env("XDG_RUNTIME_DIR", t.join("run"))
        .env("LEMUSYNC_TEST_ROOT", t)
        .env("LANG", "C");
    c
}

#[cfg(target_os = "linux")]
fn start(bin: &Path, t: &Path, socket: &Path) -> Result<(std::process::Child, Duration), String> {
    let begin = Instant::now();
    let mut child = isolated(&bin.join("lemusyncd"), t)
        .arg("--headless")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| e.to_string())?;
    while std::os::unix::net::UnixStream::connect(socket).is_err() {
        if child.try_wait().map_err(|e| e.to_string())?.is_some() || begin.elapsed().as_secs() > 20
        {
            let _ = child.kill();
            return Err("lemusyncd did not get ready".into());
        }
        std::thread::sleep(Duration::from_micros(200));
    }
    Ok((child, begin.elapsed()))
}

#[cfg(target_os = "linux")]
fn stop(mut child: std::process::Child) -> Result<(), String> {
    Command::new("kill")
        .args(["-TERM", &child.id().to_string()])
        .status()
        .map_err(|e| e.to_string())?;
    child.wait().map_err(|e| e.to_string())?;
    Ok(())
}

/// utime + stime of `pid`, in clock ticks (fields 14 and 15 of `/proc/<pid>/stat`).
#[cfg(target_os = "linux")]
fn cpu_ticks(pid: u32) -> Result<u64, String> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).map_err(|e| e.to_string())?;
    // The command name may hold spaces: count fields after its closing parenthesis.
    let rest = stat.rsplit_once(')').map(|(_, r)| r).unwrap_or_default();
    let f: Vec<&str> = rest.split_whitespace().collect();
    let get = |i: usize| f.get(i).and_then(|v| v.parse::<u64>().ok());
    // After ")" field 3 is index 0, so utime (14) is 11 and stime (15) is 12.
    Ok(get(11).ok_or("no utime")? + get(12).ok_or("no stime")?)
}

/// (`VmRSS`, `VmHWM`) of `pid` in KiB.
#[cfg(target_os = "linux")]
fn memory(pid: u32) -> Result<(u64, u64), String> {
    let s = std::fs::read_to_string(format!("/proc/{pid}/status")).map_err(|e| e.to_string())?;
    let kib = |k: &str| {
        s.lines()
            .find_map(|l| l.strip_prefix(k))
            .and_then(|v| v.trim().trim_end_matches("kB").trim().parse::<u64>().ok())
            .ok_or_else(|| format!("no {k}"))
    };
    Ok((kib("VmRSS:")?, kib("VmHWM:")?))
}

#[cfg(target_os = "linux")]
fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1000.0
}

#[cfg(target_os = "linux")]
#[allow(clippy::cast_precision_loss)]
fn kib_mib(k: u64) -> f64 {
    k as f64 / 1024.0
}

/// A temp folder removed on drop.
#[cfg(target_os = "linux")]
struct Tmp(PathBuf);

#[cfg(target_os = "linux")]
impl Tmp {
    fn new() -> Result<Self, String> {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| e.to_string())?
            .as_nanos();
        let p =
            std::env::temp_dir().join(format!("lemusync-budgets-{}-{nanos}", std::process::id()));
        std::fs::create_dir_all(&p).map_err(|e| e.to_string())?;
        Ok(Self(p))
    }
}

#[cfg(target_os = "linux")]
impl Drop for Tmp {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC: &str = "| Key | Reference |\n|---|---|\n| `lemusyncd_bytes` | 18 230 232 |\n| `rescan_10k_ms` | 21.5 |\n";

    #[test]
    fn reference_rows_are_read_with_spaces_and_decimals() {
        assert_eq!(reference(SPEC, "lemusyncd_bytes"), Some(18_230_232.0));
        assert_eq!(reference(SPEC, "rescan_10k_ms"), Some(21.5));
        assert_eq!(reference(SPEC, "missing"), None);
    }

    #[test]
    fn only_more_than_twenty_percent_over_fails() {
        assert!(compare("k", 120.0, 100.0).is_none());
        assert!(compare("k", 80.0, 100.0).is_none());
        assert!(compare("k", 120.1, 100.0).is_some());
    }
}
