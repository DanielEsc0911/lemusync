//! Calls for the game flows (3c): history, conflicts, leases, restore, resolve and lease
//! release. One short connection each, with a timeout, like `calls`. Every failure is a
//! [`ConnectionState`]: `Failed { code, .. }` carries the daemon's error code.
//! Spec: docs/specs/ui/app-shell.md (Data flow).

#![cfg_attr(
    not(test),
    expect(dead_code, reason = "reached from the FRB surface in 3c Task 4")
)]

use std::time::Duration;

use lemusync_api::{Request, Response};
use lemusync_ipc::IpcError;

use super::Link;
use super::calls::{connect, connection_from, exchange};
use super::flow_model::{ConflictDto, HistoryDto, LeaseDto, MovedDto};
use super::model::ConnectionState;

/// Reads go through the daemon's one-at-a-time queue, so they can wait behind a sync
/// (V-SHELL-5), like `Games`.
pub(crate) const READ_TIMEOUT: Duration = Duration::from_secs(30);
/// Restore and resolve move files (V-FLOW-1: a starting value). On a timeout the daemon
/// still finishes the work; only this call gives up waiting.
pub(crate) const WRITE_TIMEOUT: Duration = Duration::from_secs(120);

/// Sends one request on a fresh connection and picks the expected answer out of it. Any
/// other answer is a protocol error (reads as not running, as in `calls`).
async fn one<T>(
    link: &Link,
    request: Request,
    limit: Duration,
    pick: impl FnOnce(Response) -> Option<T>,
) -> Result<T, ConnectionState> {
    let mut client = connect(link).await?;
    let response = exchange(&mut client, request, limit)
        .await
        .map_err(super::calls::CallError::into_state)?;
    pick(response).ok_or_else(|| connection_from(IpcError::Protocol))
}

/// The versions of one player's stream of a game. `game` is the key; `emulator` narrows it
/// to one emulator, as `--emu` does.
pub(crate) async fn history(
    link: &Link,
    player: &str,
    emulator: &str,
    game: &str,
) -> Result<HistoryDto, ConnectionState> {
    let request = Request::History {
        game: game.into(),
        emu: Some(emulator.into()),
        player: Some(player.into()),
    };
    one(link, request, READ_TIMEOUT, |r| match r {
        Response::History(h) => Some(HistoryDto::from(&h)),
        _ => None,
    })
    .await
}

/// Every conflicted stream.
pub(crate) async fn conflicts(link: &Link) -> Result<Vec<ConflictDto>, ConnectionState> {
    one(link, Request::Conflicts, READ_TIMEOUT, |r| match r {
        Response::Conflicts(c) => Some(c.value.iter().map(ConflictDto::from).collect()),
        _ => None,
    })
    .await
}

/// Every lease in the repo, this device's included.
pub(crate) async fn leases(link: &Link) -> Result<Vec<LeaseDto>, ConnectionState> {
    one(link, Request::LeaseStatus, READ_TIMEOUT, |r| match r {
        Response::LeaseStatus(l) => Some(l.value.iter().map(LeaseDto::from).collect()),
        _ => None,
    })
    .await
}

/// Restores a version (or a local backup by name). The engine backs up the current save
/// first (sync-model).
pub(crate) async fn restore(
    link: &Link,
    player: &str,
    emulator: &str,
    game: &str,
    what: &str,
) -> Result<MovedDto, ConnectionState> {
    let request = Request::Restore {
        player: player.into(),
        emulator: emulator.into(),
        game: game.into(),
        what: what.into(),
    };
    one(link, request, WRITE_TIMEOUT, |r| match r {
        Response::Stream(s) => Some(MovedDto::from(&s)),
        _ => None,
    })
    .await
}

/// Resolves a conflict by keeping one head; the others stay in history.
pub(crate) async fn resolve(
    link: &Link,
    player: &str,
    emulator: &str,
    game: &str,
    keep: &str,
) -> Result<MovedDto, ConnectionState> {
    let request = Request::Resolve {
        player: player.into(),
        emulator: emulator.into(),
        game: game.into(),
        keep: keep.into(),
    };
    one(link, request, WRITE_TIMEOUT, |r| match r {
        Response::Resolved(s) => Some(MovedDto::from(&s.value)),
        _ => None,
    })
    .await
}

/// Releases this device's leases on a game (`lease release --force`); how many it released.
pub(crate) async fn lease_release(link: &Link, game: &str) -> Result<u32, ConnectionState> {
    let request = Request::LeaseRelease { game: game.into() };
    one(link, request, READ_TIMEOUT, |r| match r {
        Response::Released(n) => Some(n),
        _ => None,
    })
    .await
}

// The link tests drive a fake daemon over a Unix socket; Windows IPC is Phase 2 Task 11.
#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::link::fake::{Answer, Fake, Script, error_body};
    use lemusync_api::{
        ConflictInfo, HeadInfo, HistoryEntry, HistoryReport, LeaseInfo, StreamAction, StreamReport,
        WithWarnings,
    };

    fn with(name: &'static str, answer: Answer<Response>) -> Fake {
        Fake::start(Script {
            replies: vec![(name, answer)],
            ..Script::default()
        })
    }

    fn nowhere() -> Link {
        let dir = tempfile::tempdir().expect("tempdir");
        Link::at(dir.path().join("daemon.sock"))
    }

    fn history_report() -> HistoryReport {
        HistoryReport {
            api_version: lemusync_api::API_VERSION,
            name: None,
            versions: vec![HistoryEntry {
                id: "v1".into(),
                created_at: 2,
                device: "deck".into(),
                files: 1,
                bytes: 9,
                parents: vec![],
                head: true,
                takeover: None,
            }],
            backups: vec![],
        }
    }

    fn stream_report() -> StreamReport {
        StreamReport {
            player: "Ana".into(),
            emulator: "cemu".into(),
            game: "aa".into(),
            account: None,
            action: StreamAction::FastForward,
            version: Some("v1".into()),
            heads: vec!["v1".into()],
            done: true,
            files: 4,
            bytes: 40,
        }
    }

    #[tokio::test]
    async fn history_asks_for_one_players_stream_on_one_emulator() {
        let fake = with("history", Answer::Ok(Response::History(history_report())));
        let h = history(&fake.link, "Ana", "cemu", "aa")
            .await
            .expect("history");
        assert_eq!(h.versions[0].id, "v1");
        assert_eq!(h.versions[0].created_ms, 2000);
        assert_eq!(
            fake.requests(),
            [Request::History {
                game: "aa".into(),
                emu: Some("cemu".into()),
                player: Some("Ana".into()),
            }]
        );
    }

    #[tokio::test]
    async fn conflicts_and_leases_map_every_entry() {
        let conflict = ConflictInfo {
            player: "Ana".into(),
            emulator: "cemu".into(),
            game: "aa".into(),
            name: None,
            heads: vec![HeadInfo {
                version: "h1".into(),
                device_name: "pc".into(),
                player: "Ana".into(),
                created_at_ms: 1,
                files: 1,
                bytes: 1,
                changed_files: 1,
            }],
        };
        let fake = with(
            "conflicts",
            Answer::Ok(Response::Conflicts(WithWarnings {
                value: vec![conflict],
                warnings: vec![],
            })),
        );
        let got = conflicts(&fake.link).await.expect("conflicts");
        assert_eq!(got.len(), 1);
        assert_eq!(got[0].heads[0].device, "pc");
        assert_eq!(fake.requests(), [Request::Conflicts]);

        let lease = LeaseInfo {
            player: "Ana".into(),
            emulator: "cemu".into(),
            game: "aa".into(),
            device_id: "dev-2".into(),
            device: "pc".into(),
            renewed_at_ms: 7,
            live: true,
            ours: false,
        };
        let fake = with(
            "lease_status",
            Answer::Ok(Response::LeaseStatus(WithWarnings {
                value: vec![lease],
                warnings: vec![],
            })),
        );
        let got = leases(&fake.link).await.expect("leases");
        assert_eq!((got[0].device.as_str(), got[0].live), ("pc", true));
        assert_eq!(fake.requests(), [Request::LeaseStatus]);
    }

    #[tokio::test]
    async fn restore_and_resolve_send_the_ids_they_were_given() {
        let fake = with("restore", Answer::Ok(Response::Stream(stream_report())));
        let moved = restore(&fake.link, "Ana", "cemu", "aa", "v1")
            .await
            .expect("restore");
        assert_eq!((moved.files, moved.bytes), (4, 40));
        assert_eq!(
            fake.requests(),
            [Request::Restore {
                player: "Ana".into(),
                emulator: "cemu".into(),
                game: "aa".into(),
                what: "v1".into(),
            }]
        );

        let fake = with(
            "resolve",
            Answer::Ok(Response::Resolved(WithWarnings {
                value: stream_report(),
                warnings: vec![],
            })),
        );
        let moved = resolve(&fake.link, "Ana", "cemu", "aa", "h2")
            .await
            .expect("resolve");
        assert_eq!(moved.version.as_deref(), Some("v1"));
        assert_eq!(
            fake.requests(),
            [Request::Resolve {
                player: "Ana".into(),
                emulator: "cemu".into(),
                game: "aa".into(),
                keep: "h2".into(),
            }]
        );
    }

    #[tokio::test]
    async fn lease_release_returns_how_many_it_released() {
        let fake = with("lease_release", Answer::Ok(Response::Released(1)));
        assert_eq!(lease_release(&fake.link, "aa").await, Ok(1));
        assert_eq!(
            fake.requests(),
            [Request::LeaseRelease { game: "aa".into() }]
        );
    }

    #[tokio::test]
    async fn a_daemon_error_is_failed_with_its_code() {
        let fake = with("restore", Answer::Error(error_body("version_not_found")));
        assert_eq!(
            restore(&fake.link, "Ana", "cemu", "aa", "nope").await,
            Err(ConnectionState::Failed {
                code: "version_not_found".into(),
                message: "version_not_found happened".into(),
            })
        );
    }

    #[tokio::test]
    async fn the_wrong_answer_or_no_daemon_reads_as_not_running() {
        let fake = with("conflicts", Answer::Ok(Response::Done));
        assert_eq!(
            conflicts(&fake.link).await,
            Err(ConnectionState::NotRunning)
        );
        assert_eq!(leases(&nowhere()).await, Err(ConnectionState::NotRunning));
        assert_eq!(
            history(&Link { path: None }, "Ana", "cemu", "aa").await,
            Err(ConnectionState::NotRunning)
        );
    }

    #[tokio::test(start_paused = true)]
    async fn reads_give_up_after_thirty_seconds_and_writes_after_two_minutes() {
        let timeout = ConnectionState::Failed {
            code: "timeout".into(),
            message: "the daemon did not answer in time".into(),
        };
        let fake = with("history", Answer::Silent);
        let t0 = tokio::time::Instant::now();
        assert_eq!(
            history(&fake.link, "Ana", "cemu", "aa").await,
            Err(timeout.clone())
        );
        assert_eq!(t0.elapsed(), READ_TIMEOUT);

        let fake = with("resolve", Answer::Silent);
        let t0 = tokio::time::Instant::now();
        assert_eq!(
            resolve(&fake.link, "Ana", "cemu", "aa", "h1").await,
            Err(timeout)
        );
        assert_eq!(t0.elapsed(), WRITE_TIMEOUT);
    }
}
