//! The daemon link for Dart: thin wrappers over `crate::link`. Each FRB fn is one line over a
//! `*_at` fn that takes the socket path, so tests never touch the user's own socket.
//! Spec: docs/specs/ui/app-shell.md (Code, Data flow).

use std::sync::Arc;

use flutter_rust_bridge::frb;
use tokio::sync::Notify;

use crate::frb_generated::StreamSink;
use crate::link::{Link, calls, watch};

pub use crate::link::calls::SyncNow;
pub use crate::link::model::*;

/// One watch of the daemon. Dart makes it, starts the event stream with [`WatchHandle::watch`]
/// and calls [`WatchHandle::stop`] when it stops listening. (A fn that takes a `StreamSink`
/// cannot also return the handle through FRB, so the handle owns the stream.)
#[frb(opaque)]
pub struct WatchHandle {
    stop: Arc<Notify>,
}

impl WatchHandle {
    #[frb(sync)]
    #[must_use]
    #[allow(
        clippy::new_without_default,
        reason = "FRB constructor, not a Default value"
    )]
    pub fn new() -> WatchHandle {
        WatchHandle {
            stop: Arc::new(Notify::new()),
        }
    }

    /// Starts watching the daemon; events go to `sink` until [`WatchHandle::stop`] or until
    /// the listener is gone. Returns at once.
    pub fn watch(&self, sink: StreamSink<LinkEvent>) {
        watch_at(Link::default_socket(), Arc::clone(&self.stop), move |e| {
            sink.add(e).is_ok()
        });
    }

    /// Ends the watch task at once. `notify_one` keeps the permit if the task is between
    /// waits, so a stop is never lost.
    pub fn stop(&self) {
        self.stop.notify_one();
    }
}

/// The daemon's state right now. Never fails: "not running" is a value.
pub async fn snapshot() -> Snapshot {
    snapshot_at(&Link::default_socket()).await
}

/// The game library, or `None` on any error.
pub async fn games() -> Option<GamesDto> {
    games_at(&Link::default_socket()).await
}

/// Asks the daemon to sync now.
pub async fn sync_now() -> SyncNow {
    sync_now_at(&Link::default_socket()).await
}

fn watch_at(
    link: Link,
    stop: Arc<Notify>,
    mut emit: impl FnMut(LinkEvent) -> bool + Send + 'static,
) {
    match crate::link::runtime() {
        Ok(rt) => {
            rt.spawn(watch::run(link, stop, emit));
        }
        Err(e) => {
            // No worker thread: say so once; the app shows the failure.
            emit(LinkEvent::Connection(ConnectionState::Failed {
                code: "runtime".into(),
                message: e.to_string(),
            }));
        }
    }
}

async fn snapshot_at(link: &Link) -> Snapshot {
    calls::status(link).await
}

async fn games_at(link: &Link) -> Option<GamesDto> {
    calls::games(link).await.ok()
}

async fn sync_now_at(link: &Link) -> SyncNow {
    calls::sync_now(link).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::link::model::ConnectionState;

    #[tokio::test]
    async fn snapshot_without_a_daemon_reads_not_running() {
        let dir = tempfile::tempdir().expect("tempdir");
        let snap = snapshot_at(&Link::at(dir.path().join("daemon.sock"))).await;
        assert_eq!(snap.connection, ConnectionState::NotRunning);
        assert!(snap.status.is_none());
    }

    #[tokio::test]
    async fn games_without_a_daemon_is_none() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert!(
            games_at(&Link::at(dir.path().join("daemon.sock")))
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn sync_now_without_a_daemon_is_not_running() {
        let dir = tempfile::tempdir().expect("tempdir");
        let out = sync_now_at(&Link::at(dir.path().join("daemon.sock"))).await;
        assert_eq!(out, SyncNow::NotRunning);
    }

    #[test]
    fn stop_ends_a_started_watch() {
        let dir = tempfile::tempdir().expect("tempdir");
        let (tx, rx) = std::sync::mpsc::channel();
        let handle = WatchHandle::new();
        watch_at(
            Link::at(dir.path().join("daemon.sock")),
            Arc::clone(&handle.stop),
            move |e| tx.send(e).is_ok(),
        );
        // No daemon: the watcher announces NotRunning and backs off.
        let first = rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("event");
        assert!(matches!(
            first,
            LinkEvent::Connection(ConnectionState::NotRunning)
        ));
        handle.stop();
        // The task ends, dropping `emit` and with it the sender.
        assert_eq!(
            rx.recv_timeout(std::time::Duration::from_secs(5)),
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected)
        );
    }
}
