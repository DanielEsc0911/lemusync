//! The daemon link for Dart: thin wrappers over `crate::link`. Each FRB fn is one line over a
//! `*_at` fn that takes the socket path, so tests never touch the user's own socket.
//! Spec: docs/specs/ui/app-shell.md (Code, Data flow).

use std::sync::Arc;

use flutter_rust_bridge::frb;
use tokio::sync::Notify;

use crate::frb_generated::StreamSink;
use crate::link::play::Answerer;
use crate::link::{Link, calls, flows, play, watch};

pub use crate::link::calls::SyncNow;
pub use crate::link::flow_model::*;
pub use crate::link::model::*;
pub use crate::link::play::PlayUpdate;

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

/// One play session (3c). Dart makes it, starts it with [`PlayHandle::play`], answers its
/// questions with [`PlayHandle::answer`] and calls [`PlayHandle::stop`] when it stops
/// listening. One session per handle.
#[frb(opaque)]
pub struct PlayHandle {
    stop: Arc<Notify>,
    answerer: Arc<std::sync::Mutex<Option<Answerer>>>,
}

impl PlayHandle {
    #[frb(sync)]
    #[must_use]
    #[allow(
        clippy::new_without_default,
        reason = "FRB constructor, not a Default value"
    )]
    pub fn new() -> PlayHandle {
        PlayHandle {
            stop: Arc::new(Notify::new()),
            answerer: Arc::default(),
        }
    }

    /// Starts the session; updates go to `sink` until it ends, [`PlayHandle::stop`] or the
    /// listener is gone. Returns at once.
    pub fn play(
        &self,
        player: String,
        emulator: String,
        game: String,
        sink: StreamSink<PlayUpdate>,
    ) {
        self.play_at(Link::default_socket(), player, emulator, game, move |u| {
            sink.add(u).is_ok()
        });
    }

    /// Answers the question the session is waiting on; `false` when no session runs.
    #[frb(sync)]
    pub fn answer(&self, answer: PlayAnswerDto) -> bool {
        self.answerer
            .lock()
            .map(|a| a.as_ref().is_some_and(|a| a.answer(answer)))
            .unwrap_or(false)
    }

    /// Ends the session's link at once (the daemon keeps a running game's session).
    pub fn stop(&self) {
        self.stop.notify_one();
    }

    fn play_at(
        &self,
        link: Link,
        player: String,
        emulator: String,
        game: String,
        mut emit: impl FnMut(PlayUpdate) -> bool + Send + 'static,
    ) {
        let rt = match crate::link::runtime() {
            Ok(rt) => rt,
            Err(e) => {
                emit(PlayUpdate::Failed(ConnectionState::Failed {
                    code: "runtime".into(),
                    message: e.to_string(),
                }));
                return;
            }
        };
        let (mut rx, answerer) = {
            let _in_rt = rt.enter();
            play::start(link, player, emulator, game)
        };
        if let Ok(mut slot) = self.answerer.lock() {
            *slot = Some(answerer);
        }
        let (stop, slot) = (Arc::clone(&self.stop), Arc::clone(&self.answerer));
        rt.spawn(async move {
            loop {
                tokio::select! {
                    u = rx.recv() => {
                        if !u.is_some_and(&mut emit) {
                            break;
                        }
                    }
                    () = stop.notified() => break,
                }
            }
            // Dropping `rx` (and the answerer) ends the session's connection.
            if let Ok(mut a) = slot.lock() {
                a.take();
            }
        });
    }
}

/// The versions of one player's stream of a game.
///
/// # Errors
/// The connection state when the call could not be answered (`Failed` with the daemon's code).
pub async fn history(
    player: String,
    emulator: String,
    game: String,
) -> Result<HistoryDto, ConnectionState> {
    flows::history(&Link::default_socket(), &player, &emulator, &game).await
}

/// Every conflicted stream.
///
/// # Errors
/// As [`history`].
pub async fn conflicts() -> Result<Vec<ConflictDto>, ConnectionState> {
    flows::conflicts(&Link::default_socket()).await
}

/// Every lease in the repo.
///
/// # Errors
/// As [`history`].
pub async fn leases() -> Result<Vec<LeaseDto>, ConnectionState> {
    flows::leases(&Link::default_socket()).await
}

/// Restores a version or a local backup; the current save is backed up first.
///
/// # Errors
/// As [`history`].
pub async fn restore(
    player: String,
    emulator: String,
    game: String,
    what: String,
) -> Result<MovedDto, ConnectionState> {
    flows::restore(&Link::default_socket(), &player, &emulator, &game, &what).await
}

/// Resolves a conflict by keeping one head.
///
/// # Errors
/// As [`history`].
pub async fn resolve(
    player: String,
    emulator: String,
    game: String,
    keep: String,
) -> Result<MovedDto, ConnectionState> {
    flows::resolve(&Link::default_socket(), &player, &emulator, &game, &keep).await
}

/// Releases this device's leases on a game; how many.
///
/// # Errors
/// As [`history`].
pub async fn lease_release(game: String) -> Result<u32, ConnectionState> {
    flows::lease_release(&Link::default_socket(), &game).await
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

// These tests reach a daemon socket path; Windows IPC is Phase 2 Task 11.
#[cfg(all(test, unix))]
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
    fn a_play_without_a_daemon_fails_and_then_takes_no_answer() {
        let dir = tempfile::tempdir().expect("tempdir");
        let (tx, rx) = std::sync::mpsc::channel();
        let handle = PlayHandle::new();
        handle.play_at(
            Link::at(dir.path().join("daemon.sock")),
            "Ana".into(),
            "cemu".into(),
            "aa".into(),
            move |u| tx.send(u).is_ok(),
        );
        let first = rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("update");
        assert_eq!(first, PlayUpdate::Failed(ConnectionState::NotRunning));
        // The forwarder ends with the session, dropping `emit` and the answerer.
        assert_eq!(
            rx.recv_timeout(std::time::Duration::from_secs(5)),
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected)
        );
        assert!(!handle.answer(PlayAnswerDto::Wait));
    }

    #[test]
    fn stop_ends_a_play_on_a_question_and_the_daemon_sees_the_hang_up() {
        use crate::link::fake::{Fake, PlayStep, Script};
        use lemusync_api::{PlayQuestion, Response};
        let rt = crate::link::runtime().expect("runtime");
        let fake = {
            let _in = rt.enter();
            Fake::start(Script {
                play: vec![
                    PlayStep::Send(Box::new(Response::Prompt(PlayQuestion::Lease {
                        by: "pc".into(),
                        ago_s: 1,
                        stale: false,
                    }))),
                    PlayStep::Read,
                ],
                ..Script::default()
            })
        };
        let (tx, rx) = std::sync::mpsc::channel();
        let handle = PlayHandle::new();
        handle.play_at(
            fake.link.clone(),
            "Ana".into(),
            "cemu".into(),
            "aa".into(),
            move |u| tx.send(u).is_ok(),
        );
        let first = rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("update");
        assert!(matches!(first, PlayUpdate::Question(_)));
        handle.stop();
        assert_eq!(
            rx.recv_timeout(std::time::Duration::from_secs(5)),
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected)
        );
        for _ in 0..500 {
            if fake.log().contains(&"eof") {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(fake.log().contains(&"eof"), "{:?}", fake.log());
        assert!(!handle.answer(PlayAnswerDto::Wait), "the session is over");
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
