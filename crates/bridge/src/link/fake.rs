//! A scripted daemon for the link tests: a real `lemusync_ipc::Server` on a socket in a temp
//! dir (never the user's socket). It records which requests arrive, in order.

use std::sync::{Arc, Mutex};

use lemusync_api::{
    DaemonStatus, ErrorBody, Event, GameInfo, GameSource, GamesReport, Request, Response, Session,
    Status, StatusReport, StatusState, StreamStatus, SyncReport,
};
use lemusync_ipc::Server;
use tokio::sync::{Semaphore, broadcast};

use super::Link;

/// How the fake answers one kind of request.
#[derive(Clone)]
pub(crate) enum Answer<T> {
    Ok(T),
    Error(ErrorBody),
    /// Never answers.
    Silent,
    /// Answers once the test adds a permit.
    Held(Arc<Semaphore>, T),
}

/// What the fake does, set before it starts.
#[derive(Clone)]
pub(crate) struct Script {
    pub status: Answer<StatusReport>,
    pub games: Answer<GamesReport>,
    pub sync: Answer<SyncReport>,
    /// Sent to every subscriber right after `Subscribed`.
    pub on_subscribe: Vec<Event>,
}

pub(crate) fn status_report() -> StatusReport {
    StatusReport {
        api_version: lemusync_api::API_VERSION,
        device: "deck".into(),
        device_id: "dev-1".into(),
        repo: None,
        backend: None,
        grants: vec!["/secret/grant".into()],
        players: vec!["Ana".into(), "Ben".into()],
        streams: vec![
            stream("Ana", "aa", Some(300)),
            stream("Ben", "aa", Some(200)),
        ],
        daemon: Some(DaemonStatus {
            version: "9.9.9".into(),
            status: status(Some(100)),
            session: Some(session()),
        }),
    }
}

fn stream(player: &str, game: &str, ms: Option<u64>) -> StreamStatus {
    StreamStatus {
        player: player.into(),
        emulator: "cemu".into(),
        game: game.into(),
        name: None,
        account: None,
        base: Some("/secret/base".into()),
        last_version_ms: ms,
    }
}

pub(crate) fn status(last_sync_ms: Option<u64>) -> Status {
    Status {
        state: StatusState::Synced,
        last_sync_ms,
        uploads: 0,
        downloads: 0,
        bytes: 0,
        paused_until_ms: None,
        queued: 0,
        conflicts: 0,
        foreign_leases: 0,
        error: None,
    }
}

pub(crate) fn session() -> Session {
    Session {
        game: "aa".into(),
        name: Some("Game aa".into()),
        emulator: "cemu".into(),
        player: "Ana".into(),
        device: "deck".into(),
        started_ms: 42,
    }
}

pub(crate) fn games_report() -> GamesReport {
    let game = |key: &str, name: &str| GameInfo {
        emulator: "cemu".into(),
        key: key.into(),
        name: Some(name.into()),
        path: Some(format!("/secret/games/{key}.rpx")),
        source: GameSource::Scan,
        aliases: vec![],
    };
    GamesReport {
        api_version: lemusync_api::API_VERSION,
        games: vec![game("aa", "Game aa"), game("bb", "Game bb")],
    }
}

pub(crate) fn sync_report() -> SyncReport {
    SyncReport {
        api_version: lemusync_api::API_VERSION,
        dry_run: false,
        streams: vec![],
        warnings: vec![],
    }
}

pub(crate) fn error_body(code: &str) -> ErrorBody {
    ErrorBody {
        code: code.into(),
        message: format!("{code} happened"),
        detail: None,
    }
}

impl Default for Script {
    fn default() -> Self {
        Script {
            status: Answer::Ok(status_report()),
            games: Answer::Ok(games_report()),
            sync: Answer::Ok(sync_report()),
            on_subscribe: vec![],
        }
    }
}

/// A running fake daemon. Dropping it stops it and removes the socket.
pub(crate) struct Fake {
    pub link: Link,
    log: Arc<Mutex<Vec<&'static str>>>,
    feed: broadcast::Sender<Option<Event>>,
    task: tokio::task::JoinHandle<()>,
    _dir: Option<tempfile::TempDir>,
}

impl Fake {
    pub(crate) fn start(script: Script) -> Fake {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("daemon.sock");
        Fake::start_at(&path, script, Some(dir))
    }

    /// Starts on `path` (inside a dir the caller keeps alive, or the `dir` handed over).
    pub(crate) fn start_at(
        path: &std::path::Path,
        script: Script,
        dir: Option<tempfile::TempDir>,
    ) -> Fake {
        let server = Server::bind(path).expect("bind");
        let link = Link::at(server.path().to_path_buf());
        let log = Arc::new(Mutex::new(Vec::new()));
        let (feed, _) = broadcast::channel(64);
        let task = tokio::spawn(accept_loop(server, script, Arc::clone(&log), feed.clone()));
        Fake {
            link,
            log,
            feed,
            task,
            _dir: dir,
        }
    }

    /// Requests seen so far, in arrival order: "subscribe", "status", "games", "sync".
    pub(crate) fn log(&self) -> Vec<&'static str> {
        self.log.lock().expect("log").clone()
    }

    pub(crate) fn count(&self, what: &str) -> usize {
        self.log().iter().filter(|w| **w == what).count()
    }

    /// Pushes an event to every current subscriber.
    pub(crate) fn push(&self, event: Event) {
        let _ = self.feed.send(Some(event));
    }

    /// Closes every current subscriber's connection.
    pub(crate) fn hang_up(&self) {
        let _ = self.feed.send(None);
    }
}

impl Drop for Fake {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn accept_loop(
    server: Server,
    script: Script,
    log: Arc<Mutex<Vec<&'static str>>>,
    feed: broadcast::Sender<Option<Event>>,
) {
    while let Ok(incoming) = server.accept().await {
        let (script, log, rx) = (script.clone(), Arc::clone(&log), feed.subscribe());
        tokio::spawn(async move {
            if let Ok(Some(conn)) = incoming.handshake().await {
                serve(conn, script, log, rx).await;
            }
        });
    }
}

async fn answer<T: Clone>(a: &Answer<T>, ok: impl FnOnce(T) -> Response) -> Option<Response> {
    match a {
        Answer::Ok(v) => Some(ok(v.clone())),
        Answer::Error(b) => Some(Response::Error(b.clone())),
        Answer::Silent => std::future::pending().await,
        Answer::Held(gate, v) => {
            let _ = gate
                .acquire()
                .await
                .map(tokio::sync::SemaphorePermit::forget);
            Some(ok(v.clone()))
        }
    }
}

async fn serve(
    mut conn: lemusync_ipc::Connection,
    script: Script,
    log: Arc<Mutex<Vec<&'static str>>>,
    mut rx: broadcast::Receiver<Option<Event>>,
) {
    let note = |w: &'static str| log.lock().expect("log").push(w);
    while let Some(req) = conn.recv().await {
        let reply = match req {
            Request::Status => {
                note("status");
                answer(&script.status, Response::Status).await
            }
            Request::Games => {
                note("games");
                answer(&script.games, Response::Games).await
            }
            Request::Sync { .. } => {
                note("sync");
                answer(&script.sync, Response::Sync).await
            }
            Request::Subscribe => {
                note("subscribe");
                let _ = conn.send(&Response::Subscribed).await;
                for e in &script.on_subscribe {
                    let _ = conn.event(e).await;
                }
                while let Ok(Some(e)) = rx.recv().await {
                    if conn.event(&e).await.is_err() {
                        break;
                    }
                }
                return;
            }
            _ => Some(Response::Error(error_body("unexpected"))),
        };
        if let Some(r) = reply
            && conn.send(&r).await.is_err()
        {
            return;
        }
    }
}
