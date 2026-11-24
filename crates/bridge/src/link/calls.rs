//! One-shot calls to the daemon (`Status`, `Games`, `Sync`), each on a fresh short connection
//! with a timeout, and the mapping from IPC errors to [`ConnectionState`].

use std::time::Duration;

use futures_util::Stream;
use lemusync_api::{ErrorBody, Event, Request, Response};
use lemusync_engine::{library, recent};
use lemusync_ipc::{Client, IpcError};

use super::model::{
    ConnectionState, GameEntry, GamesDto, RecentDto, SessionDto, Snapshot, StatusDto,
};
use super::{CLIENT_NAME, Link};

/// V-SHELL-12: starting values, tuned from the maintainer's run.
pub(crate) const STATUS_TIMEOUT: Duration = Duration::from_secs(10);
pub(crate) const SYNC_TIMEOUT: Duration = Duration::from_secs(10);
/// `Games` queues behind a running sync (V-SHELL-5).
pub(crate) const GAMES_TIMEOUT: Duration = Duration::from_secs(30);

/// What a "sync now" came to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SyncNow {
    Done,
    NotRunning,
    Failed { code: String, message: String },
    TimedOut,
}

/// A failed call on an open connection.
enum CallError {
    Timeout,
    State(ConnectionState),
}

impl CallError {
    fn into_state(self) -> ConnectionState {
        match self {
            CallError::Timeout => ConnectionState::Failed {
                code: "timeout".into(),
                message: "the daemon did not answer in time".into(),
            },
            CallError::State(s) => s,
        }
    }
}

/// Maps an error from connecting (the handshake) to what the app shows. A path inside
/// `UnsafePath` is dropped here: it never reaches Dart (B2).
pub(crate) fn connection_from(err: IpcError) -> ConnectionState {
    match err {
        IpcError::Unsupported => ConnectionState::Unsupported,
        IpcError::UnsafePath(_) => ConnectionState::Unsafe,
        IpcError::Rejected(body) => ConnectionState::Refused {
            message: body.message,
        },
        IpcError::NotRunning
        | IpcError::NoRuntimeDir
        | IpcError::Closed
        | IpcError::Io(_)
        | IpcError::Json(_)
        | IpcError::FrameTooLarge(_)
        | IpcError::Protocol
        | IpcError::HandshakeTimeout
        | IpcError::AlreadyRunning => ConnectionState::NotRunning,
    }
}

/// An error body from a call on a connection whose handshake worked: `Failed`, not `Refused`.
pub(crate) fn failed_from(body: ErrorBody) -> ConnectionState {
    ConnectionState::Failed {
        code: body.code,
        message: body.message,
    }
}

/// Maps an error from a call on an open connection.
fn call_error_from(err: IpcError) -> ConnectionState {
    match err {
        IpcError::Rejected(body) => failed_from(body),
        other => connection_from(other),
    }
}

async fn connect(link: &Link) -> Result<Client, ConnectionState> {
    let Some(path) = &link.path else {
        return Err(ConnectionState::NotRunning);
    };
    match tokio::time::timeout(STATUS_TIMEOUT, Client::connect(path, CLIENT_NAME)).await {
        Ok(Ok(client)) => Ok(client),
        Ok(Err(e)) => Err(connection_from(e)),
        Err(_) => Err(connection_from(IpcError::HandshakeTimeout)),
    }
}

/// Opens the event subscription: a connection of its own that only receives events.
pub(crate) async fn subscribe(
    link: &Link,
) -> Result<impl Stream<Item = Result<Event, IpcError>> + use<>, ConnectionState> {
    let client = connect(link).await?;
    match tokio::time::timeout(STATUS_TIMEOUT, client.subscribe()).await {
        Ok(Ok(stream)) => Ok(stream),
        Ok(Err(e)) => Err(call_error_from(e)),
        Err(_) => Err(CallError::Timeout.into_state()),
    }
}

async fn exchange(
    client: &mut Client,
    request: Request,
    limit: Duration,
) -> Result<Response, CallError> {
    match tokio::time::timeout(limit, client.call(request)).await {
        Err(_) => Err(CallError::Timeout),
        Ok(Err(e)) => Err(CallError::State(call_error_from(e))),
        Ok(Ok(Response::Error(body))) => Err(CallError::State(failed_from(body))),
        Ok(Ok(response)) => Ok(response),
    }
}

/// Connects once and asks for `Status`. Any failure is a value in `connection`.
pub(crate) async fn status(link: &Link) -> Snapshot {
    let mut client = match connect(link).await {
        Ok(c) => c,
        Err(state) => return Snapshot::offline(state),
    };
    let daemon_version = client.daemon().daemon_version.clone();
    match exchange(&mut client, Request::Status, STATUS_TIMEOUT).await {
        Ok(Response::Status(report)) => {
            let daemon = report.daemon.as_ref();
            Snapshot {
                connection: ConnectionState::Online { daemon_version },
                status: daemon.map(|d| StatusDto::from(&d.status)),
                session: daemon
                    .and_then(|d| d.session.as_ref())
                    .map(SessionDto::from),
                device: Some(report.device),
                players: report.players,
            }
        }
        Ok(_) => Snapshot::offline(connection_from(IpcError::Protocol)),
        Err(e) => Snapshot::offline(e.into_state()),
    }
}

/// The game library and the recent list, on a connection of its own. The library rule needs
/// the report's streams, so this asks for its own fresh `Status` first.
pub(crate) async fn games(link: &Link) -> Result<GamesDto, ConnectionState> {
    let mut client = connect(link).await?;
    let status = match exchange(&mut client, Request::Status, STATUS_TIMEOUT).await {
        Ok(Response::Status(r)) => r,
        Ok(_) => return Err(connection_from(IpcError::Protocol)),
        Err(e) => return Err(e.into_state()),
    };
    let games = match exchange(&mut client, Request::Games, GAMES_TIMEOUT).await {
        Ok(Response::Games(g)) => g,
        Ok(_) => return Err(connection_from(IpcError::Protocol)),
        Err(e) => return Err(e.into_state()),
    };
    Ok(GamesDto {
        games: library(&status, &games)
            .iter()
            .map(GameEntry::from)
            .collect(),
        recent: recent(&status, &games)
            .iter()
            .map(RecentDto::from)
            .collect(),
    })
}

/// Runs a sync now (the tray's and Ctrl+R's action).
pub(crate) async fn sync_now(link: &Link) -> SyncNow {
    let mut client = match connect(link).await {
        Ok(c) => c,
        Err(ConnectionState::Refused { message }) => {
            return SyncNow::Failed {
                code: "refused".into(),
                message,
            };
        }
        Err(_) => return SyncNow::NotRunning,
    };
    match exchange(&mut client, Request::Sync { dry_run: false }, SYNC_TIMEOUT).await {
        Ok(Response::Sync(_)) => SyncNow::Done,
        Err(CallError::Timeout) => SyncNow::TimedOut,
        Err(CallError::State(ConnectionState::Failed { code, message })) => {
            SyncNow::Failed { code, message }
        }
        Ok(_) | Err(CallError::State(_)) => SyncNow::NotRunning,
    }
}

// The link tests drive a fake daemon over a Unix socket; Windows IPC is Phase 2 Task 11.
#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::link::fake::{Answer, Fake, Script, error_body, games_report, status_report};
    use crate::link::model::{PlayerStreamDto, StateDto};
    use std::path::PathBuf;
    use std::sync::Arc;
    use tokio::sync::Semaphore;

    fn nowhere() -> Link {
        let dir = tempfile::tempdir().expect("tempdir");
        // The dir is dropped (deleted) at once: the path points at nothing.
        Link::at(dir.path().join("daemon.sock"))
    }

    #[tokio::test]
    async fn status_returns_the_daemons_facts() {
        let fake = Fake::start(Script::default());
        let snap = status(&fake.link).await;
        assert_eq!(
            snap.connection,
            ConnectionState::Online {
                daemon_version: env!("CARGO_PKG_VERSION").into()
            }
        );
        let st = snap.status.expect("status");
        assert_eq!(st.state, StateDto::Synced);
        assert_eq!(st.last_sync_ms, Some(100));
        let session = snap.session.expect("session");
        assert_eq!((session.game.as_str(), session.started_ms), ("aa", 42));
        assert_eq!(snap.device.as_deref(), Some("deck"));
        assert_eq!(snap.players, ["Ana", "Ben"]);
    }

    #[tokio::test]
    async fn status_without_a_socket_is_not_running() {
        let snap = status(&nowhere()).await;
        assert_eq!(snap, Snapshot::offline(ConnectionState::NotRunning));
        let snap = status(&Link { path: None }).await;
        assert_eq!(snap, Snapshot::offline(ConnectionState::NotRunning));
    }

    #[tokio::test]
    async fn connection_from_maps_every_ipc_error() {
        use std::io::{Error, ErrorKind};
        // A real JSON error, from a frame whose body is not JSON.
        let mut bad: &[u8] = &[0, 0, 0, 1, b'x'];
        let json = match lemusync_ipc::frame::read_frame::<u8, _>(&mut bad).await {
            Err(e @ IpcError::Json(_)) => e,
            other => panic!("expected a JSON error, got {other:?}"),
        };
        let rows: Vec<(IpcError, ConnectionState)> = vec![
            (IpcError::NotRunning, ConnectionState::NotRunning),
            (IpcError::NoRuntimeDir, ConnectionState::NotRunning),
            (IpcError::Unsupported, ConnectionState::Unsupported),
            (
                IpcError::UnsafePath(PathBuf::from("/secret/dir")),
                ConnectionState::Unsafe,
            ),
            (
                IpcError::Rejected(error_body("api_version_mismatch")),
                ConnectionState::Refused {
                    message: "api_version_mismatch happened".into(),
                },
            ),
            (IpcError::Closed, ConnectionState::NotRunning),
            (
                IpcError::Io(Error::new(ErrorKind::BrokenPipe, "x")),
                ConnectionState::NotRunning,
            ),
            (json, ConnectionState::NotRunning),
            (IpcError::FrameTooLarge(9), ConnectionState::NotRunning),
            (IpcError::Protocol, ConnectionState::NotRunning),
            (IpcError::HandshakeTimeout, ConnectionState::NotRunning),
            (IpcError::AlreadyRunning, ConnectionState::NotRunning),
        ];
        for (err, want) in rows {
            let label = format!("{err:?}");
            assert_eq!(connection_from(err), want, "{label}");
        }
    }

    #[test]
    fn an_error_body_on_an_open_connection_is_failed_not_refused() {
        assert_eq!(
            failed_from(error_body("busy")),
            ConnectionState::Failed {
                code: "busy".into(),
                message: "busy happened".into()
            }
        );
    }

    #[tokio::test]
    async fn a_handshake_refusal_carries_the_message() {
        // A server that answers Hello with an api_version error.
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("daemon.sock");
        let listener = tokio::net::UnixListener::bind(&path).expect("bind");
        let link = Link::at(path);
        tokio::spawn(async move {
            use lemusync_api::Hello;
            use lemusync_ipc::frame::{read_frame, write_frame};
            let (mut s, _) = listener.accept().await.expect("accept");
            let _hello = read_frame::<Hello, _>(&mut s).await;
            let refusal = Response::Error(ErrorBody {
                code: "api_version_mismatch".into(),
                message: "update LemuSync".into(),
                detail: None,
            });
            let _ = write_frame(&mut s, &refusal).await;
        });
        let snap = status(&link).await;
        assert_eq!(
            snap.connection,
            ConnectionState::Refused {
                message: "update LemuSync".into()
            }
        );
    }

    #[tokio::test]
    async fn a_response_error_to_status_is_failed() {
        let fake = Fake::start(Script {
            status: Answer::Error(error_body("busy")),
            ..Script::default()
        });
        let snap = status(&fake.link).await;
        assert_eq!(
            snap,
            Snapshot::offline(ConnectionState::Failed {
                code: "busy".into(),
                message: "busy happened".into()
            })
        );
    }

    #[tokio::test(start_paused = true)]
    async fn a_status_that_never_answers_times_out_after_ten_seconds() {
        let fake = Fake::start(Script {
            status: Answer::Silent,
            ..Script::default()
        });
        let t0 = tokio::time::Instant::now();
        let snap = status(&fake.link).await;
        assert_eq!(t0.elapsed(), STATUS_TIMEOUT);
        assert_eq!(
            snap.connection,
            ConnectionState::Failed {
                code: "timeout".into(),
                message: "the daemon did not answer in time".into()
            }
        );
    }

    #[tokio::test(start_paused = true)]
    async fn games_that_never_answer_fail_after_thirty_seconds() {
        let fake = Fake::start(Script {
            games: Answer::Silent,
            ..Script::default()
        });
        let t0 = tokio::time::Instant::now();
        let got = games(&fake.link).await;
        assert_eq!(t0.elapsed(), GAMES_TIMEOUT);
        assert!(matches!(got, Err(ConnectionState::Failed { code, .. }) if code == "timeout"));
    }

    #[tokio::test(start_paused = true)]
    async fn a_sync_that_never_answers_times_out_after_ten_seconds() {
        let fake = Fake::start(Script {
            sync: Answer::Silent,
            ..Script::default()
        });
        let t0 = tokio::time::Instant::now();
        assert_eq!(sync_now(&fake.link).await, SyncNow::TimedOut);
        assert_eq!(t0.elapsed(), SYNC_TIMEOUT);
    }

    #[tokio::test]
    async fn sync_now_maps_the_answers() {
        let ok = Fake::start(Script::default());
        assert_eq!(sync_now(&ok.link).await, SyncNow::Done);
        let bad = Fake::start(Script {
            sync: Answer::Error(error_body("locked")),
            ..Script::default()
        });
        assert_eq!(
            sync_now(&bad.link).await,
            SyncNow::Failed {
                code: "locked".into(),
                message: "locked happened".into()
            }
        );
        assert_eq!(sync_now(&nowhere()).await, SyncNow::NotRunning);
        assert_eq!(sync_now(&Link { path: None }).await, SyncNow::NotRunning);
    }

    #[tokio::test]
    async fn games_follow_the_engines_order_without_any_path() {
        let fake = Fake::start(Script::default());
        let dto = games(&fake.link).await.expect("games");
        let keys: Vec<&str> = dto.games.iter().map(|g| g.key.as_str()).collect();
        assert_eq!(keys, ["aa", "bb"], "newest stream first, never-synced last");
        assert!(dto.games.iter().all(|g| g.launchable));
        assert_eq!(
            dto.games[0].streams,
            [
                PlayerStreamDto {
                    player: "Ana".into(),
                    last_version_ms: Some(300)
                },
                PlayerStreamDto {
                    player: "Ben".into(),
                    last_version_ms: Some(200)
                }
            ]
        );
        // One recent item per player stream, newest first.
        let recent: Vec<(&str, &str)> = dto
            .recent
            .iter()
            .map(|r| (r.key.as_str(), r.player.as_str()))
            .collect();
        assert_eq!(recent, [("aa", "Ana"), ("aa", "Ben")]);
        assert!(!format!("{dto:?}").contains("/secret"));
        let _ = (status_report(), games_report());
    }

    #[tokio::test]
    async fn games_asks_for_a_fresh_status_first() {
        let fake = Fake::start(Script::default());
        games(&fake.link).await.expect("games");
        assert_eq!(fake.log(), ["status", "games"]);
    }

    #[tokio::test]
    async fn a_held_games_answer_completes_when_released() {
        let gate = Arc::new(Semaphore::new(0));
        let fake = Fake::start(Script {
            games: Answer::Held(Arc::clone(&gate), games_report()),
            ..Script::default()
        });
        let link = fake.link.clone();
        let task = tokio::spawn(async move { games(&link).await });
        tokio::task::yield_now().await;
        assert!(!task.is_finished());
        gate.add_permits(1);
        assert!(task.await.expect("join").is_ok());
    }
}
