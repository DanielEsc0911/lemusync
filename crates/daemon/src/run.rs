//! The daemon's run loop around the engine service: the IPC server, the tray, notifications
//! and sleep hooks (architecture §Process model, reliability §Sleep and resume).
//!
//! The engine runs on its own thread (`Service::spawn`); everything here talks to it through
//! a [`ServiceHandle`]. This loop maps the status model onto the tray view and events onto
//! coalesced notices, turns tray clicks into requests, and acknowledges a suspend only after
//! the service checkpointed (within 4 s).

use std::future::Future;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use std::collections::HashMap;

use lemusync_api::{
    ErrorBody, ErrorKind as ApiErrorKind, Event, GamesReport, PlayAnswer, Request, Response,
    Session, Status, StatusReport, StatusState,
};
use lemusync_engine::{PlayCall, PlayFrame, ServiceHandle};
use lemusync_ipc::{Incoming, Server};
use tokio::sync::{broadcast, mpsc};

use crate::i18n::{I18n, s as fs};
use crate::notices::{Coalescer, NoticeEvent, notice};
use crate::platform::{
    IdleGuard, NoticeId, Notifier, Power, RecentItem, SleepEdge, Tray, TrayAction, start_notifier,
    start_power, start_tray,
};
use crate::view::{ErrorKind, SessionInput, StatusInput, ViewState, tray_view};

/// The whole suspend checkpoint, ack included (reliability §Sleep and resume: logind's
/// `InhibitDelayMaxUSec` is 5 s by default).
pub const SUSPEND_BUDGET: Duration = Duration::from_secs(4);

/// Desktop presence. `None` in `--headless` mode.
pub struct Desktop {
    pub tray: Box<dyn Tray>,
    pub tray_actions: mpsc::Receiver<TrayAction>,
    pub notifier: Box<dyn Notifier>,
    pub notice_actions: mpsc::Receiver<(NoticeId, String)>,
    pub power: Box<dyn Power>,
}

impl Desktop {
    /// The platform's tray, notifier and power hooks. Must run inside a tokio runtime.
    #[must_use]
    pub fn start() -> Self {
        let (tray_tx, tray_actions) = mpsc::channel(16);
        let (notice_tx, notice_actions) = mpsc::channel(16);
        Self {
            tray: start_tray(tray_tx),
            tray_actions,
            notifier: start_notifier(notice_tx),
            notice_actions,
            power: start_power(),
        }
    }
}

/// `lemusync_api::Status` → the tray's local input (`view` doesn't know the wire types).
#[must_use]
pub fn status_input(s: &Status) -> StatusInput {
    let state = match s.state {
        StatusState::Synced => ViewState::Synced,
        StatusState::Scanning => ViewState::Scanning,
        StatusState::Syncing => ViewState::Syncing,
        StatusState::Paused => ViewState::Paused,
        StatusState::Offline => ViewState::Offline,
        StatusState::Attention => ViewState::Attention,
        StatusState::Error => ViewState::Error(match s.error {
            Some(ApiErrorKind::Auth) => ErrorKind::Auth,
            Some(ApiErrorKind::Quota) => ErrorKind::Quota,
            Some(ApiErrorKind::Permission) => ErrorKind::Permission,
            Some(ApiErrorKind::NotSetUp) => ErrorKind::NotSetUp,
            Some(ApiErrorKind::Config) | None => ErrorKind::Config,
        }),
    };
    let mut input = StatusInput::new(state);
    input.uploads = s.uploads;
    input.downloads = s.downloads;
    input.queued = s.queued;
    input.conflicts = s.conflicts;
    input.foreign_leases = s.foreign_leases;
    input.paused = state == ViewState::Paused || s.paused_until_ms.is_some();
    input
}

/// `lemusync_api::Session` → the tray's session overlay.
#[must_use]
pub fn session_input(s: &Session) -> SessionInput {
    SessionInput {
        game: s.name.clone().unwrap_or_else(|| s.game.clone()),
        player: s.player.clone(),
    }
}

/// What an event asks the player, with its coalescing key and kind.
#[must_use]
pub fn notice_for(e: &Event, last: Option<&Status>) -> Option<(String, &'static str, NoticeEvent)> {
    let stream = |p: &str, e: &str, g: &str| format!("{p}/{e}/{g}");
    match e {
        Event::ConflictDetected(c) => Some((
            stream(&c.player, &c.emulator, &c.game),
            "conflict",
            NoticeEvent::Conflict {
                game: c.name.clone().unwrap_or_else(|| c.game.clone()),
            },
        )),
        Event::LeaseWarning(l) if l.live && !l.ours => Some((
            stream(&l.player, &l.emulator, &l.game),
            "lease",
            NoticeEvent::ForeignLease {
                game: l.game.clone(),
                device: l.device.clone(),
            },
        )),
        Event::RemoteNewer(n) => Some((
            stream(&n.player, &n.emulator, &n.game),
            "remote_newer",
            NoticeEvent::RemoteNewer {
                game: n.name.clone().unwrap_or_else(|| n.game.clone()),
                device: n.device.clone(),
                emulator: n.emulator.clone(),
            },
        )),
        Event::StatusChanged(s) if s.error != last.and_then(|l| l.error) => match s.error {
            Some(ApiErrorKind::Auth) => Some((String::new(), "auth", NoticeEvent::AuthError)),
            Some(ApiErrorKind::Quota) => Some((String::new(), "quota", NoticeEvent::QuotaError)),
            _ => None,
        },
        _ => None,
    }
}

/// The request a tray click means; `None` for `Quit`. "Play" is launcher mode with the
/// non-interactive policy: a held lease or a conflict becomes a notification, nothing
/// launches.
#[must_use]
pub fn tray_request(a: &TrayAction, now_ms: u64) -> Option<Request> {
    match a {
        TrayAction::SyncNow => Some(Request::Sync { dry_run: false }),
        TrayAction::Pause1h => Some(Request::Pause {
            until_ms: Some(now_ms.saturating_add(60 * 60_000)),
        }),
        TrayAction::PauseUntilResume => Some(Request::Pause { until_ms: None }),
        TrayAction::Resume => Some(Request::Resume),
        TrayAction::Play(r) => {
            let (player, emulator, game) = parse_play_ref(r)?;
            Some(Request::Play {
                player,
                emulator,
                game,
                args: Vec::new(),
            })
        }
        TrayAction::Quit => None,
    }
}

/// Separates the parts of a "Play" item's id (unit separator: never in a name).
const SEP: char = '\u{1f}';

fn play_ref(player: &str, emulator: &str, game: &str) -> String {
    format!("{player}{SEP}{emulator}{SEP}{game}")
}

fn parse_play_ref(r: &str) -> Option<(String, String, String)> {
    let mut parts = r.split(SEP);
    let (p, e, g) = (parts.next()?, parts.next()?, parts.next()?);
    parts
        .next()
        .is_none()
        .then(|| (p.to_owned(), e.to_owned(), g.to_owned()))
}

/// The tray's "Play" items, newest play first: every synced stream, so the user sees why one
/// can't be played. A game this device has no file for (see `launchable`) is listed disabled,
/// with a hint. Streams with no known last version go last, then by label. The player is in the
/// label when there are several. The order and the cap are `lemusync_engine::recent`'s; this
/// only builds the localised labels and the ids.
#[must_use]
pub fn recent_games(status: &StatusReport, games: &GamesReport, t: &I18n) -> Vec<RecentItem> {
    let many = status.players.len() > 1;
    lemusync_engine::recent(status, games)
        .into_iter()
        .map(|i| {
            let name = i.name.unwrap_or_else(|| i.game.clone());
            let mut label = if many {
                format!("{name} · {}", i.player)
            } else {
                name
            };
            if !i.launchable {
                label = format!("{label} ({})", t.t("tray-recent-no-file", &[]));
            }
            RecentItem {
                id: play_ref(&i.player, &i.emulator, &i.game),
                label,
                launchable: i.launchable,
            }
        })
        .collect()
}

/// A failed tray "Play" as a notice: a held lease and a conflict get their own (the lease one
/// offers "Wait"); anything else says why in the player's language.
#[must_use]
pub fn play_failed(game: &str, e: &ErrorBody, t: &I18n) -> NoticeEvent {
    let detail = e.detail.clone().unwrap_or_default();
    match e.code.as_str() {
        "lease_held" => NoticeEvent::ForeignLease {
            game: game.to_owned(),
            device: detail,
        },
        "needs_decision" => NoticeEvent::Conflict {
            game: game.to_owned(),
        },
        code => {
            let (key, data) = detail.split_once(':').unwrap_or((detail.as_str(), ""));
            let reason = t
                .get(&format!("error-{code}-{key}"), &[("detail", fs(data))])
                .or_else(|| t.get(&format!("error-{code}"), &[("detail", fs(detail.clone()))]))
                .unwrap_or_else(|| e.message.clone());
            NoticeEvent::PlayFailed {
                game: game.to_owned(),
                reason,
            }
        }
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
}

/// Serves IPC clients until the task is aborted. `quit` hears a `Shutdown` request.
async fn serve(server: Server, handle: ServiceHandle, quit: mpsc::Sender<()>) {
    loop {
        match server.accept().await {
            Ok(incoming) => {
                tokio::spawn(connection(incoming, handle.clone(), quit.clone()));
            }
            Err(e) => {
                tracing::warn!("ipc accept failed: {e}");
                // A broken listener must not spin.
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        }
    }
}

async fn connection(incoming: Incoming, handle: ServiceHandle, quit: mpsc::Sender<()>) {
    let mut conn = match incoming.handshake().await {
        Ok(Some(c)) => c,
        Ok(None) => return,
        Err(e) => {
            tracing::info!("ipc handshake failed: {e}");
            return;
        }
    };
    tracing::info!("ipc client {} connected", conn.client());
    while let Some(request) = conn.recv().await {
        match request {
            Request::Subscribe => {
                let mut rx = handle.subscribe();
                if conn
                    .send(&lemusync_api::Response::Subscribed)
                    .await
                    .is_err()
                    || conn
                        .event(&Event::StatusChanged(handle.status()))
                        .await
                        .is_err()
                {
                    return;
                }
                loop {
                    match rx.recv().await {
                        Ok(e) => {
                            if conn.event(&e).await.is_err() {
                                return;
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(n)) => {
                            tracing::info!("ipc subscriber lagged by {n} events");
                        }
                        Err(broadcast::error::RecvError::Closed) => return,
                    }
                }
            }
            Request::Play {
                player,
                emulator,
                game,
                args,
            } => {
                let call = PlayCall {
                    player,
                    emulator,
                    game,
                    args,
                    io: None,
                };
                if !play(&mut conn, &handle, call).await {
                    return;
                }
            }
            Request::Shutdown => {
                let r = handle.call(Request::Shutdown).await;
                let _ = conn.send(&r).await;
                let _ = quit.send(()).await;
                return;
            }
            other => {
                let r = handle.call(other).await;
                if conn.send(&r).await.is_err() {
                    return;
                }
            }
        }
    }
}

/// `play` for a client: steps and questions go out as frames, answers come back. Once the
/// client is gone the session still runs to the end (upload, lease release), every later
/// question gets the non-interactive answer. `false` when the client is gone.
async fn play(
    conn: &mut lemusync_ipc::Connection,
    handle: &ServiceHandle,
    mut call: PlayCall,
) -> bool {
    let (tx, mut rx) = mpsc::unbounded_channel();
    call.io = Some(tx);
    let h = handle.clone();
    let done = h.play(call);
    tokio::pin!(done);
    let mut alive = true;
    loop {
        tokio::select! {
            r = &mut done => {
                return alive && conn.send(&r).await.is_ok();
            }
            Some(frame) = rx.recv() => match frame {
                PlayFrame::Event(e) => {
                    alive = alive && conn.send(&Response::PlayEvent(e)).await.is_ok();
                }
                PlayFrame::Ask(q, reply) => {
                    let answer = if alive && conn.send(&Response::Prompt(q)).await.is_ok() {
                        match conn.recv().await {
                            Some(Request::Answer(a)) => a,
                            Some(_) => PlayAnswer::Refuse,
                            None => {
                                alive = false;
                                PlayAnswer::Refuse
                            }
                        }
                    } else {
                        alive = false;
                        PlayAnswer::Refuse
                    };
                    let _ = reply.send(answer);
                }
            },
        }
    }
}

/// `(player, emulator, game)`.
type Stream = (String, String, String);

/// How often "Wait" checks whether a foreign lease is gone, and for how long at most.
const WAIT_POLL: Duration = Duration::from_secs(30);
const WAIT_MAX: Duration = Duration::from_hours(12);

/// After "Wait" on a foreign-lease notice: polls `lease_status` until no other device holds a
/// live lease on the stream, then sends `LeaseFree`.
async fn wait_for_lease(
    handle: ServiceHandle,
    stream: Stream,
    game: String,
    out: mpsc::Sender<(NoticeEvent, Option<Stream>)>,
    poll: Duration,
) {
    let end = tokio::time::Instant::now() + WAIT_MAX;
    while tokio::time::Instant::now() < end {
        if let Response::LeaseStatus(l) = handle.call(Request::LeaseStatus).await {
            let held = l.value.iter().any(|l| {
                l.live
                    && !l.ours
                    && (&l.player, &l.emulator, &l.game) == (&stream.0, &stream.1, &stream.2)
            });
            if !held {
                let _ = out.send((NoticeEvent::LeaseFree { game }, None)).await;
                return;
            }
        }
        tokio::time::sleep(poll).await;
    }
}

/// Fetches the tray's "Play" items.
async fn fetch_recent(
    handle: ServiceHandle,
    lang: &'static str,
    out: mpsc::Sender<Vec<RecentItem>>,
) {
    let (Response::Status(st), Response::Games(g)) = (
        handle.call(Request::Status).await,
        handle.call(Request::Games).await,
    ) else {
        return;
    };
    let items = recent_games(&st, &g, &I18n::new(lang)); // not `Send`: gone before the await
    let _ = out.send(items).await;
}

/// Desktop state the loop keeps between events.
struct Presence {
    tray: Box<dyn Tray>,
    notifier: Box<dyn Notifier>,
    power: Box<dyn Power>,
    i18n: I18n,
    status: Status,
    session: Option<Session>,
    coalesce: Coalescer,
    idle: Option<IdleGuard>,
    recent: Vec<RecentItem>,
    /// Foreign-lease notices that offer "Wait": the stream and the game's label.
    waitable: HashMap<NoticeId, (Stream, String)>,
}

impl Presence {
    fn show(&self) {
        let session = self.session.as_ref().map(session_input);
        let mut input = status_input(&self.status);
        input.recent.clone_from(&self.recent);
        let view = tray_view(&input, session.as_ref(), &self.i18n);
        self.tray.show(&view);
    }

    fn on_event(&mut self, e: &Event) {
        if let Some((stream, kind, ev)) = notice_for(e, Some(&self.status))
            && self.coalesce.allow(&stream, kind, now_ms())
        {
            let id = self.notifier.notify(notice(&ev, &self.i18n));
            if let (Event::LeaseWarning(l), NoticeEvent::ForeignLease { game, .. }) = (e, &ev) {
                self.waitable.insert(
                    id,
                    (
                        (l.player.clone(), l.emulator.clone(), l.game.clone()),
                        game.clone(),
                    ),
                );
            }
        }
        match e {
            Event::StatusChanged(s) => {
                self.status = s.clone();
                // Transfers keep the machine from idle-sleeping (never a user suspend).
                if s.state == StatusState::Syncing {
                    if self.idle.is_none() {
                        self.idle = Some(self.power.block_idle("Syncing saves"));
                    }
                } else {
                    self.idle = None;
                }
                self.show();
            }
            Event::SessionChanged(s) => {
                self.session.clone_from(s);
                self.show();
            }
            _ => {}
        }
    }
}

/// The menu item `id` names, if a click on it may launch the game.
fn playable<'a>(recent: &'a [RecentItem], id: &str) -> Option<&'a RecentItem> {
    recent.iter().find(|i| i.launchable && i.id == id)
}

/// Sends the request a tray click means. A failed "Play" comes back as a notice.
fn tray_click(
    a: &TrayAction,
    handle: &ServiceHandle,
    presence: Option<&Presence>,
    own_tx: &mpsc::Sender<(NoticeEvent, Option<Stream>)>,
) {
    let Some(r) = tray_request(a, now_ms()) else {
        return;
    };
    let h = handle.clone();
    let (
        TrayAction::Play(id),
        Request::Play {
            player,
            emulator,
            game,
            ..
        },
    ) = (a, &r)
    else {
        tokio::spawn(async move { h.call(r).await });
        return;
    };
    // Only a launchable item of the current menu plays; a stale or disabled one does nothing.
    let Some(item) = presence.and_then(|p| playable(&p.recent, id)) else {
        return;
    };
    let label = item.label.clone();
    let stream = (player.clone(), emulator.clone(), game.clone());
    let out = own_tx.clone();
    let lang = presence.map_or_else(I18n::env_lang, |p| p.i18n.lang());
    tokio::spawn(async move {
        if let Response::Error(e) = h.call(r).await {
            tracing::info!("tray play did not launch ({})", e.code);
            let ev = play_failed(&label, &e, &I18n::new(lang));
            let _ = out.send((ev, Some(stream))).await;
        }
    });
}

/// Runs the daemon until `shutdown` resolves, a client sends `Shutdown` or the tray's Quit
/// is clicked; then stops the service and drops the server (which unlinks the socket).
pub async fn run(
    handle: ServiceHandle,
    server: Server,
    desktop: Option<Desktop>,
    i18n: I18n,
    shutdown: impl Future<Output = ()>,
) {
    let (quit_tx, mut quit_rx) = mpsc::channel(1);
    let ipc = tokio::spawn(serve(server, handle.clone(), quit_tx));
    let mut events = handle.subscribe();
    let lang = i18n.lang();
    let (mut tray_rx, mut notice_rx) = (None, None);
    let mut presence = desktop.map(|d| {
        tray_rx = Some(d.tray_actions);
        notice_rx = Some(d.notice_actions);
        Presence {
            tray: d.tray,
            notifier: d.notifier,
            power: d.power,
            i18n,
            status: handle.status(),
            session: None,
            coalesce: Coalescer::default(),
            idle: None,
            recent: Vec::new(),
            waitable: HashMap::new(),
        }
    });
    let mut sleep_rx = presence.as_ref().map(|p| p.power.sleep_events());
    // Notices from background tasks (tray "Play", "Wait"), with the stream "Wait" would poll.
    let (own_tx, mut own_rx) = mpsc::channel::<(NoticeEvent, Option<Stream>)>(16);
    let (recent_tx, mut recent_rx) = mpsc::channel(4);
    if let Some(p) = &presence {
        p.show();
        tokio::spawn(fetch_recent(handle.clone(), lang, recent_tx.clone()));
    }
    let mut last_sync = None;
    tokio::pin!(shutdown);
    loop {
        tokio::select! {
            () = &mut shutdown => break,
            _ = quit_rx.recv() => break,
            e = events.recv() => match e {
                Ok(e) => {
                    if let Some(p) = &mut presence {
                        p.on_event(&e);
                        // A session ended (launcher mode may have uploaded by now).
                        if matches!(e, Event::SessionChanged(None)) {
                            tokio::spawn(fetch_recent(handle.clone(), lang, recent_tx.clone()));
                        }
                        // A finished sync may have added games: refresh the "Play" items.
                        if let Event::StatusChanged(s) = &e
                            && s.last_sync_ms != last_sync
                        {
                            last_sync = s.last_sync_ms;
                            tokio::spawn(fetch_recent(handle.clone(), lang, recent_tx.clone()));
                        }
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => {}
                Err(broadcast::error::RecvError::Closed) => break,
            },
            Some(a) = recv_opt(tray_rx.as_mut()) => {
                if a == TrayAction::Quit {
                    break;
                }
                tray_click(&a, &handle, presence.as_ref(), &own_tx);
            }
            Some((id, action)) = recv_opt(notice_rx.as_mut()) => {
                let target = presence.as_mut().and_then(|p| p.waitable.remove(&id));
                match (action.as_str(), target) {
                    (crate::notices::ACTION_WAIT, Some((stream, game))) => {
                        tokio::spawn(wait_for_lease(handle.clone(), stream, game, own_tx.clone(), WAIT_POLL));
                    }
                    (other, _) => tracing::info!("notification {} action {other}: nothing to do", id.0),
                }
            }
            Some((ev, stream)) = own_rx.recv() => {
                if let Some(p) = &mut presence {
                    let id = p.notifier.notify(notice(&ev, &p.i18n));
                    if let (NoticeEvent::ForeignLease { game, .. }, Some(stream)) = (&ev, stream) {
                        p.waitable.insert(id, (stream, game.clone()));
                    }
                }
            }
            Some(recent) = recent_rx.recv() => {
                if let Some(p) = &mut presence {
                    p.recent = recent;
                    p.show();
                }
            }
            Some(edge) = recv_opt(sleep_rx.as_mut()) => match edge {
                SleepEdge::Suspending => {
                    if tokio::time::timeout(SUSPEND_BUDGET, handle.suspend()).await.is_err() {
                        tracing::warn!("suspend checkpoint ran over {SUSPEND_BUDGET:?}");
                    }
                    if let Some(p) = &presence {
                        p.power.ack_suspend();
                    }
                }
                SleepEdge::Resumed => handle.resumed().await,
            },
        }
    }
    tracing::info!("stopping");
    handle.stop().await;
    ipc.abort();
    let _ = ipc.await; // the server (and its socket file) is dropped by now
}

/// `rx.recv()`, or never when there is no receiver (headless).
async fn recv_opt<T>(rx: Option<&mut mpsc::Receiver<T>>) -> Option<T> {
    match rx {
        Some(rx) => rx.recv().await,
        None => std::future::pending().await,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lemusync_api::{ConflictInfo, LeaseInfo, StatusFacts};
    use lemusync_engine::RECENT_MAX;

    #[test]
    fn every_status_state_maps_onto_the_view() {
        let table = [
            (StatusFacts::default(), ViewState::Synced),
            (
                StatusFacts {
                    scanning: true,
                    ..Default::default()
                },
                ViewState::Scanning,
            ),
            (
                StatusFacts {
                    syncing: true,
                    uploads: 2,
                    ..Default::default()
                },
                ViewState::Syncing,
            ),
            (
                StatusFacts {
                    paused: true,
                    ..Default::default()
                },
                ViewState::Paused,
            ),
            (
                StatusFacts {
                    offline: true,
                    queued: 3,
                    ..Default::default()
                },
                ViewState::Offline,
            ),
            (
                StatusFacts {
                    conflicts: 1,
                    ..Default::default()
                },
                ViewState::Attention,
            ),
            (
                StatusFacts {
                    error: Some(ApiErrorKind::Quota),
                    ..Default::default()
                },
                ViewState::Error(ErrorKind::Quota),
            ),
        ];
        for (facts, want) in table {
            let input = status_input(&Status::from_facts(&facts));
            assert_eq!(input.state, want);
            assert_eq!(input.uploads, facts.uploads);
            assert_eq!(input.queued, facts.queued);
        }
        let t = I18n::new("en");
        let s = Status::from_facts(&StatusFacts {
            syncing: true,
            uploads: 3,
            ..Default::default()
        });
        assert_eq!(
            tray_view(&status_input(&s), None, &t).status_line,
            "Uploading 3 changes"
        );
    }

    #[test]
    fn events_become_notices_only_when_they_need_the_player() {
        let conflict = Event::ConflictDetected(ConflictInfo {
            player: "A".into(),
            emulator: "cemu".into(),
            game: "g".into(),
            name: Some("BotW".into()),
            heads: vec![],
        });
        let (key, kind, ev) = notice_for(&conflict, None).unwrap();
        assert_eq!((key.as_str(), kind), ("A/cemu/g", "conflict"));
        assert_eq!(
            ev,
            NoticeEvent::Conflict {
                game: "BotW".into()
            }
        );
        let lease = |live| {
            Event::LeaseWarning(LeaseInfo {
                player: "A".into(),
                emulator: "cemu".into(),
                game: "g".into(),
                device_id: "d".into(),
                device: "Deck".into(),
                renewed_at_ms: 0,
                live,
                ours: false,
            })
        };
        assert!(notice_for(&lease(true), None).is_some());
        assert!(notice_for(&lease(false), None).is_none());
        let auth = Status::from_facts(&StatusFacts {
            error: Some(ApiErrorKind::Auth),
            ..Default::default()
        });
        let ev = Event::StatusChanged(auth.clone());
        assert!(notice_for(&ev, None).is_some());
        // Still the same error: no new notice.
        assert!(notice_for(&ev, Some(&auth)).is_none());
        let synced = Event::StatusChanged(Status::from_facts(&StatusFacts::default()));
        assert!(notice_for(&synced, None).is_none());
    }

    #[test]
    fn tray_clicks_become_requests() {
        assert_eq!(
            tray_request(&TrayAction::Pause1h, 1_000),
            Some(Request::Pause {
                until_ms: Some(1_000 + 3_600_000)
            })
        );
        assert_eq!(
            tray_request(&TrayAction::SyncNow, 0),
            Some(Request::Sync { dry_run: false })
        );
        assert_eq!(tray_request(&TrayAction::Quit, 0), None);
        let id = play_ref("Ana", "cemu", "101c9400");
        assert_eq!(
            tray_request(&TrayAction::Play(id), 0),
            Some(Request::Play {
                player: "Ana".into(),
                emulator: "cemu".into(),
                game: "101c9400".into(),
                args: vec![],
            })
        );
        assert_eq!(tray_request(&TrayAction::Play("junk".into()), 0), None);
    }

    #[test]
    fn not_set_up_is_its_own_error() {
        let s = Status::from_facts(&StatusFacts {
            error: Some(ApiErrorKind::NotSetUp),
            ..Default::default()
        });
        assert_eq!(
            status_input(&s).state,
            ViewState::Error(ErrorKind::NotSetUp)
        );
    }

    fn recent_fixture() -> (StatusReport, GamesReport) {
        use lemusync_api::{GameInfo, GameSource, StreamStatus};
        let stream = |p: &str, g: &str, ms: Option<u64>| StreamStatus {
            player: p.into(),
            emulator: "cemu".into(),
            game: g.into(),
            name: None,
            account: None,
            base: None,
            last_version_ms: ms,
        };
        let game = |k: &str, path: Option<&str>| GameInfo {
            emulator: "cemu".into(),
            key: k.into(),
            name: Some(format!("Game {k}")),
            path: path.map(Into::into),
            source: GameSource::Config,
            aliases: vec![],
        };
        let st = StatusReport {
            api_version: 1,
            device: "pc".into(),
            device_id: "d".into(),
            repo: None,
            backend: None,
            grants: vec![],
            players: vec!["Ana".into()],
            streams: vec![
                stream("Ana", "AA", Some(10)),
                stream("Ana", "bb", None),
                stream("Ana", "cc", Some(30)),
                stream("Ana", "dd", Some(20)),
            ],
            daemon: None,
        };
        let games = GamesReport {
            api_version: 1,
            games: vec![
                game("aa", Some("/g/a")),
                game("bb", Some("/g/b")),
                game("cc", Some("/g/c")),
            ],
        };
        (st, games)
    }

    #[test]
    fn play_items_are_newest_first_and_unlaunchable_ones_are_disabled() {
        let t = I18n::new("en");
        let (mut st, games) = recent_fixture();
        let r = recent_games(&st, &games, &t);
        let labels: Vec<&str> = r.iter().map(|i| i.label.as_str()).collect();
        // Newest first, never-synced last; "dd" is not in the catalog: its key, disabled.
        assert_eq!(
            labels,
            ["Game cc", "dd (no game file)", "Game aa", "Game bb"]
        );
        let launchable: Vec<bool> = r.iter().map(|i| i.launchable).collect();
        assert_eq!(launchable, [true, false, true, true]);
        assert_eq!(r[0].id, play_ref("Ana", "cemu", "cc"));
        // The stream's own name beats the key; the player shows when there are several.
        st.streams[3].name = Some("Bayonetta 2".into());
        st.players.push("Bea".into());
        assert_eq!(
            recent_games(&st, &games, &I18n::new("es"))[1].label,
            "Bayonetta 2 · Ana (sin archivo del juego)"
        );
    }

    #[test]
    fn two_players_give_one_item_per_stream_newest_first() {
        let (mut st, games) = recent_fixture();
        st.players.push("Bea".into());
        st.streams = vec![
            lemusync_api::StreamStatus {
                game: "aa".into(),
                last_version_ms: Some(100),
                player: "Bea".into(),
                ..st.streams[0].clone()
            },
            lemusync_api::StreamStatus {
                game: "aa".into(),
                last_version_ms: Some(200),
                ..st.streams[0].clone()
            },
        ];
        let r = recent_games(&st, &games, &I18n::new("en"));
        let got: Vec<(String, &str)> = r.iter().map(|i| (i.id.clone(), i.label.as_str())).collect();
        assert_eq!(
            got,
            [
                (play_ref("Ana", "cemu", "aa"), "Game aa · Ana"),
                (play_ref("Bea", "cemu", "aa"), "Game aa · Bea"),
            ]
        );
        st.players.pop();
        st.streams.pop();
        assert_eq!(
            recent_games(&st, &games, &I18n::new("en"))[0].label,
            "Game aa"
        );
    }

    #[test]
    fn play_items_stop_at_the_cap() {
        let (mut st, games) = recent_fixture();
        let first = st.streams[0].clone();
        st.streams = (0..20u64)
            .map(|n| lemusync_api::StreamStatus {
                game: format!("g{n}"),
                last_version_ms: Some(n),
                ..first.clone()
            })
            .collect();
        let r = recent_games(&st, &games, &I18n::new("en"));
        assert_eq!(r.len(), RECENT_MAX);
        assert_eq!(r[0].id, play_ref("Ana", "cemu", "g19"));
    }

    #[test]
    fn only_a_launchable_menu_item_plays() {
        let item = |id: &str, launchable| RecentItem {
            id: id.into(),
            label: id.into(),
            launchable,
        };
        let menu = vec![item("a", true), item("b", false)];
        assert!(playable(&menu, "a").is_some());
        assert!(playable(&menu, "b").is_none());
        assert!(playable(&menu, "gone").is_none());
    }

    #[test]
    fn a_failed_tray_play_says_why() {
        let t = I18n::new("en");
        let body = |code: &str, detail: &str| ErrorBody {
            code: code.into(),
            message: "m".into(),
            detail: Some(detail.into()),
        };
        assert_eq!(
            play_failed("BotW", &body("lease_held", "Deck"), &t),
            NoticeEvent::ForeignLease {
                game: "BotW".into(),
                device: "Deck".into()
            }
        );
        assert_eq!(
            play_failed("BotW", &body("needs_decision", "g"), &t),
            NoticeEvent::Conflict {
                game: "BotW".into()
            }
        );
        let NoticeEvent::PlayFailed { reason, .. } =
            play_failed("BotW", &body("emulator_running", "Cemu"), &t)
        else {
            panic!("play failed")
        };
        assert!(reason.contains("Cemu"), "{reason}");
    }
}
