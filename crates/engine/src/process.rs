//! Is an emulator running? (I3; emulator-profiles §Linux paths: match the executable name, not `comm`.)
//! Starting one and waiting for it to exit (sync-model §Launcher mode): argv only, never a shell.

use std::collections::BTreeSet;
use std::ffi::OsString;
use std::sync::atomic::{AtomicBool, Ordering};

/// Starts the emulator. The CLI uses `OsLauncher`; tests use a fake.
pub trait Launcher: Send + Sync {
    /// # Errors
    /// The OS error when the process can't start.
    fn spawn(&self, argv: &[OsString]) -> std::io::Result<Box<dyn Child>>;
}

/// A started emulator. `wait` blocks, so the engine calls it on a blocking thread.
pub trait Child: Send {
    /// Blocks until the process exits; its exit code, -1 when killed by a signal.
    ///
    /// # Errors
    /// The OS error from waiting.
    fn wait(&mut self) -> std::io::Result<i32>;
}

/// `std::process::Command` with argv[0] as the program and no shell (B2). On Unix the
/// emulator gets its own process group, so Ctrl-C in the terminal never reaches it.
pub struct OsLauncher;

impl Launcher for OsLauncher {
    fn spawn(&self, argv: &[OsString]) -> std::io::Result<Box<dyn Child>> {
        let (exe, rest) = argv
            .split_first()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "empty argv"))?;
        let mut cmd = std::process::Command::new(exe);
        cmd.args(rest);
        #[cfg(unix)]
        std::os::unix::process::CommandExt::process_group(&mut cmd, 0);
        let child = cmd.spawn()?;
        Ok(Box::new(OsChild(child)))
    }
}

struct OsChild(std::process::Child);

impl Child for OsChild {
    fn wait(&mut self) -> std::io::Result<i32> {
        Ok(self.0.wait()?.code().unwrap_or(-1))
    }
}

pub trait ProcessProbe: Send + Sync {
    fn running(&self, names: &[String]) -> bool;

    /// The emulator ids with a running process, from one refresh of the process list for all
    /// of them (`names_by_emulator`: emulator id → its profile's process names).
    fn running_set(&self, names_by_emulator: &[(String, Vec<String>)]) -> BTreeSet<String>;
}

/// The OS process list. Persistent: it keeps what the last poll learned, so a poll only
/// inspects processes it hasn't seen (Task 16: a fresh full walk cost ≈ 23 ms per poll).
/// Linux reads `/proc` directly; other OSes reuse one `sysinfo::System`.
#[derive(Default)]
pub struct OsProbe(std::sync::Mutex<Inner>);

#[cfg(target_os = "linux")]
type Inner = linux::ProcCache;

#[cfg(not(target_os = "linux"))]
#[derive(Default)]
struct Inner(Option<sysinfo::System>);

impl OsProbe {
    /// Each running process's executable file name (when readable) and `comm`.
    fn snapshot(&self) -> Vec<(Option<String>, String)> {
        let mut inner = self
            .0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        #[cfg(target_os = "linux")]
        {
            inner.refresh(&linux::RealProc)
        }
        #[cfg(not(target_os = "linux"))]
        {
            use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, UpdateKind};
            let sys = inner.0.get_or_insert_with(sysinfo::System::new);
            sys.refresh_processes_specifics(
                ProcessesToUpdate::All,
                true,
                ProcessRefreshKind::nothing().with_exe(UpdateKind::OnlyIfNotSet),
            );
            sys.processes()
                .values()
                .map(|p| {
                    let exe = p
                        .exe()
                        .and_then(|e| e.file_name())
                        .map(|n| n.to_string_lossy().into_owned());
                    (exe, p.name().to_string_lossy().into_owned())
                })
                .collect()
        }
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use std::collections::HashMap;

    /// Where process facts come from; `/proc` in production, a fake in tests.
    pub(super) trait ProcSource {
        fn pids(&self) -> Vec<u32>;
        /// `None` when the process is gone.
        fn comm(&self, pid: u32) -> Option<String>;
        /// The executable's file name; `None` when unreadable (another user's process).
        fn exe_name(&self, pid: u32) -> Option<String>;
    }

    pub(super) struct RealProc;

    impl ProcSource for RealProc {
        fn pids(&self) -> Vec<u32> {
            std::fs::read_dir("/proc")
                .map(|d| {
                    d.filter_map(|e| e.ok()?.file_name().to_str()?.parse().ok())
                        .collect()
                })
                .unwrap_or_default()
        }

        fn comm(&self, pid: u32) -> Option<String> {
            let s = std::fs::read_to_string(format!("/proc/{pid}/comm")).ok()?;
            Some(s.strip_suffix('\n').unwrap_or(&s).to_owned())
        }

        fn exe_name(&self, pid: u32) -> Option<String> {
            let target = std::fs::read_link(format!("/proc/{pid}/exe")).ok()?;
            let name = target.file_name()?.to_string_lossy().into_owned();
            // A replaced binary reads "name (deleted)"; the name is still the program's.
            Some(name.strip_suffix(" (deleted)").unwrap_or(&name).to_owned())
        }
    }

    struct Entry {
        comm: String,
        exe: Option<String>,
    }

    /// Names per pid. Each refresh lists the pids, re-reads `comm` (cheap) of known ones,
    /// re-inspects `exe` when `comm` changed (an `exec` keeps the pid), inspects new pids
    /// and drops vanished ones.
    #[derive(Default)]
    pub(crate) struct ProcCache(HashMap<u32, Entry>);

    impl ProcCache {
        pub(super) fn refresh(&mut self, src: &impl ProcSource) -> Vec<(Option<String>, String)> {
            let mut next = HashMap::with_capacity(self.0.len());
            for pid in src.pids() {
                let Some(comm) = src.comm(pid) else { continue };
                let entry = match self.0.remove(&pid) {
                    Some(e) if e.comm == comm => e,
                    _ => Entry {
                        exe: src.exe_name(pid),
                        comm,
                    },
                };
                next.insert(pid, entry);
            }
            self.0 = next;
            self.0
                .values()
                .map(|e| (e.exe.clone(), e.comm.clone()))
                .collect()
        }

        #[cfg(test)]
        pub(super) fn len(&self) -> usize {
            self.0.len()
        }
    }
}

/// Linux truncates `comm` to 15 bytes; a full-length `comm` matches a longer name it starts
/// (only used when the executable name is unreadable or differs). Erring towards "running"
/// keeps I3.
const COMM_MAX: usize = 15;

fn comm_matches(comm: &str, wanted: &str) -> bool {
    same_name(comm, wanted)
        || (cfg!(target_os = "linux")
            && comm.len() == COMM_MAX
            && wanted.len() > COMM_MAX
            && wanted.as_bytes().starts_with(comm.as_bytes()))
}

fn any_matches(procs: &[(Option<String>, String)], names: &[String]) -> bool {
    procs.iter().any(|(exe, comm)| {
        names
            .iter()
            .any(|n| exe.as_deref().is_some_and(|e| same_name(e, n)) || comm_matches(comm, n))
    })
}

impl ProcessProbe for OsProbe {
    fn running(&self, names: &[String]) -> bool {
        any_matches(&self.snapshot(), names)
    }

    fn running_set(&self, names_by_emulator: &[(String, Vec<String>)]) -> BTreeSet<String> {
        let procs = self.snapshot();
        names_by_emulator
            .iter()
            .filter(|(_, names)| any_matches(&procs, names))
            .map(|(id, _)| id.clone())
            .collect()
    }
}

/// Executable names are case-insensitive on Windows and macOS file systems.
fn same_name(found: &str, wanted: &str) -> bool {
    if cfg!(any(windows, target_os = "macos")) {
        found.eq_ignore_ascii_case(wanted)
    } else {
        found == wanted
    }
}

/// For tests: reports "running" when set.
#[derive(Default)]
pub struct FakeProbe(AtomicBool);

impl FakeProbe {
    pub fn set(&self, running: bool) {
        self.0.store(running, Ordering::SeqCst);
    }
}

impl ProcessProbe for FakeProbe {
    fn running(&self, _names: &[String]) -> bool {
        self.0.load(Ordering::SeqCst)
    }

    /// Every emulator asked about, when set.
    fn running_set(&self, names_by_emulator: &[(String, Vec<String>)]) -> BTreeSet<String> {
        if self.0.load(Ordering::SeqCst) {
            names_by_emulator.iter().map(|(id, _)| id.clone()).collect()
        } else {
            BTreeSet::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fake_probe_reports_what_it_is_told() {
        let p = FakeProbe::default();
        assert!(!p.running(&["Cemu".into()]));
        p.set(true);
        assert!(p.running(&["Cemu".into()]));
    }

    #[cfg(target_os = "linux")]
    mod proc_cache {
        use super::super::any_matches;
        use super::super::linux::{ProcCache, ProcSource};
        use std::cell::RefCell;
        use std::collections::HashMap;

        /// pid → (comm, exe name); counts `exe` reads.
        #[derive(Default)]
        struct FakeProc {
            procs: HashMap<u32, (String, Option<String>)>,
            exe_reads: RefCell<usize>,
        }

        impl FakeProc {
            fn set(&mut self, pid: u32, comm: &str, exe: Option<&str>) {
                self.procs
                    .insert(pid, (comm.to_owned(), exe.map(str::to_owned)));
            }
        }

        impl ProcSource for FakeProc {
            fn pids(&self) -> Vec<u32> {
                self.procs.keys().copied().collect()
            }
            fn comm(&self, pid: u32) -> Option<String> {
                self.procs.get(&pid).map(|p| p.0.clone())
            }
            fn exe_name(&self, pid: u32) -> Option<String> {
                *self.exe_reads.borrow_mut() += 1;
                self.procs.get(&pid).and_then(|p| p.1.clone())
            }
        }

        fn names(n: &[&str]) -> Vec<String> {
            n.iter().map(|s| (*s).to_owned()).collect()
        }

        #[test]
        fn matches_the_exe_name_or_comm_and_inspects_each_pid_once() {
            let mut src = FakeProc::default();
            src.set(1, "systemd", Some("systemd"));
            src.set(2, "Main", Some("Cemu")); // a thread-renamed comm: found by exe
            src.set(3, "dolphin-emu", None); // another user's process: comm only
            let mut cache = ProcCache::default();
            let procs = cache.refresh(&src);
            assert!(any_matches(&procs, &names(&["Cemu"])));
            assert!(any_matches(&procs, &names(&["dolphin-emu"])));
            assert!(!any_matches(&procs, &names(&["Main.exe", "ryujinx"])));
            assert_eq!(*src.exe_reads.borrow(), 3);
            cache.refresh(&src);
            assert_eq!(
                *src.exe_reads.borrow(),
                3,
                "known pids are not re-inspected"
            );
        }

        #[test]
        fn an_exec_changes_comm_and_is_re_inspected() {
            let mut src = FakeProc::default();
            src.set(7, "bash", Some("bash"));
            let mut cache = ProcCache::default();
            assert!(!any_matches(&cache.refresh(&src), &names(&["Cemu"])));
            src.set(7, "Cemu", Some("Cemu"));
            assert!(any_matches(&cache.refresh(&src), &names(&["Cemu"])));
            assert_eq!(*src.exe_reads.borrow(), 2);
        }

        #[test]
        fn a_vanished_pid_is_dropped() {
            let mut src = FakeProc::default();
            src.set(9, "Cemu", Some("Cemu"));
            let mut cache = ProcCache::default();
            assert!(any_matches(&cache.refresh(&src), &names(&["Cemu"])));
            src.procs.clear();
            assert!(!any_matches(&cache.refresh(&src), &names(&["Cemu"])));
            assert_eq!(cache.len(), 0);
        }

        #[test]
        fn a_truncated_comm_matches_the_longer_name() {
            let procs = vec![(None, "duckstation-qt-".to_owned())];
            assert!(any_matches(&procs, &names(&["duckstation-qt-x64"])));
            assert!(!any_matches(&procs, &names(&["duckstation"])));
        }
    }

    #[test]
    fn names_ignore_case_only_where_the_file_system_does() {
        assert!(same_name("Cemu", "Cemu"));
        assert_eq!(
            same_name("cemu.exe", "Cemu.exe"),
            cfg!(any(windows, target_os = "macos"))
        );
    }

    #[test]
    fn the_os_probe_does_not_see_a_made_up_process() {
        assert!(!OsProbe::default().running(&["lemusync-no-such-process-7f3a".into()]));
    }

    #[test]
    fn running_set_names_only_the_running_emulators() {
        let me = std::env::current_exe().unwrap();
        let me = me.file_name().unwrap().to_string_lossy().into_owned();
        let set = OsProbe::default().running_set(&[
            ("me".into(), vec![me]),
            ("ghost".into(), vec!["lemusync-no-such-process-7f3a".into()]),
        ]);
        assert_eq!(set, BTreeSet::from(["me".to_owned()]));
        let p = FakeProbe::default();
        let ask = [("cemu".to_owned(), vec![])];
        assert!(p.running_set(&ask).is_empty());
        p.set(true);
        assert_eq!(p.running_set(&ask), BTreeSet::from(["cemu".to_owned()]));
    }

    /// `cargo test --release -p lemusync-engine --lib running_set_timing -- --ignored --nocapture`
    #[test]
    #[ignore = "measurement, run by hand"]
    fn running_set_timing() {
        let probe = OsProbe::default();
        let ask = [(
            "cemu".to_owned(),
            vec!["Cemu".to_owned(), "Cemu.exe".to_owned()],
        )];
        let _ = probe.running_set(&ask);
        let mut runs: Vec<_> = (0..20)
            .map(|_| {
                let t = std::time::Instant::now();
                let _ = probe.running_set(&ask);
                t.elapsed()
            })
            .collect();
        runs.sort();
        let n = std::fs::read_dir("/proc").map_or(0, Iterator::count);
        println!(
            "running_set median {:?} max {:?} (/proc entries {n})",
            runs[10], runs[19]
        );
    }
}
