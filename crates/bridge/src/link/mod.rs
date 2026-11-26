//! The app's link to the daemon, in plain Rust: a snapshot call, the game list, "sync now"
//! and the event watcher. The FRB surface (Task 4) only wraps these. The app never runs an
//! engine on desktop (D35, D43); "daemon not running" is a value, never an error.
//! Spec: docs/specs/ui/app-shell.md (Connection, Data flow).

use std::path::PathBuf;
use std::sync::OnceLock;

use lemusync_ipc::socket;
use tokio::runtime::Runtime;

pub(crate) mod backoff;
pub(crate) mod calls;
pub(crate) mod flow_model;
pub(crate) mod flows;
// The link tests drive a fake daemon over a Unix socket; Windows IPC is Phase 2 Task 11.
#[cfg(all(test, unix))]
mod fake;
pub(crate) mod model;
pub(crate) mod play;
pub(crate) mod watch;

/// The client name the daemon sees in its logs.
pub(crate) const CLIENT_NAME: &str = "lemusync-app";

/// Where the daemon's socket is. `None` means there is no runtime dir, which reads as "not
/// running" (never a fallback to a shared folder, security §Local IPC).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Link {
    pub path: Option<PathBuf>,
}

impl Link {
    /// The user's own daemon socket.
    pub(crate) fn default_socket() -> Link {
        Link {
            // Only `NoRuntimeDir` can happen; either way there is no socket to try.
            path: socket::default_path().ok(),
        }
    }

    /// An explicit socket path (tests).
    #[cfg(test)]
    pub(crate) fn at(path: PathBuf) -> Link {
        Link { path: Some(path) }
    }
}

/// The runtime the watcher runs on. FRB's async runtime serves `async fn`s only; a sync fn
/// that takes a `StreamSink` runs on FRB's thread pool, where no tokio runtime is entered
/// (V-SHELL-1), so the watcher owns one worker thread, created on first use.
///
/// # Errors
/// The OS could not start the worker thread.
pub(crate) fn runtime() -> std::io::Result<&'static Runtime> {
    static RUNTIME: OnceLock<Runtime> = OnceLock::new();
    if let Some(rt) = RUNTIME.get() {
        return Ok(rt);
    }
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .thread_name("lemusync-link")
        .enable_all()
        .build()?;
    // A racing caller may have set it first; use whichever won, drop ours.
    Ok(RUNTIME.get_or_init(|| rt))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_keeps_the_path_and_the_default_never_invents_one() {
        assert_eq!(
            Link::at(PathBuf::from("/x/daemon.sock")).path,
            Some(PathBuf::from("/x/daemon.sock"))
        );
        assert_eq!(CLIENT_NAME, "lemusync-app");
    }

    #[test]
    fn the_runtime_runs_a_task_from_a_thread_without_one() {
        // Like FRB's thread pool: a plain thread, no tokio context.
        let out = std::thread::spawn(|| {
            assert!(tokio::runtime::Handle::try_current().is_err());
            let rt = runtime().expect("runtime");
            let (tx, rx) = std::sync::mpsc::channel();
            rt.spawn(async move {
                tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                tx.send(7).expect("send");
            });
            rx.recv_timeout(std::time::Duration::from_secs(5))
        })
        .join()
        .expect("thread");
        assert_eq!(out, Ok(7));
    }
}
