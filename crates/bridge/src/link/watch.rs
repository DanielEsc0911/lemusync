//! The event watcher: subscribe first, then snapshot, then follow events; reconnect with
//! backoff; stop at once when told to (B3). Spec: docs/specs/ui/app-shell.md (Data flow).

use std::pin::pin;
use std::sync::Arc;

use futures_util::future::BoxFuture;
use futures_util::{FutureExt, Stream, StreamExt};
use lemusync_api::Event;
use lemusync_ipc::IpcError;
use tokio::sync::Notify;

use super::calls;
use super::model::{ConnectionState, GamesDto, LinkEvent, ProgressDto, SessionDto, StatusDto};
use super::{Link, backoff};

type Fetch<'a> = BoxFuture<'a, Result<GamesDto, ConnectionState>>;

/// Watches the daemon until `stop` is notified or `emit` returns `false` (the listener is
/// gone). Both end it at once, whatever it is waiting for: connect, backoff, an event or the
/// game list.
pub(crate) async fn run(link: Link, stop: Arc<Notify>, mut emit: impl FnMut(LinkEvent) -> bool) {
    tokio::select! {
        () = stop.notified() => {}
        () = follow(&link, &mut emit) => {}
    }
}

/// Emits `state` as a `Connection` event unless it is the one already shown.
fn announce(
    shown: &mut Option<ConnectionState>,
    state: ConnectionState,
    emit: &mut impl FnMut(LinkEvent) -> bool,
) -> bool {
    if shown.as_ref() == Some(&state) {
        return true;
    }
    *shown = Some(state.clone());
    emit(LinkEvent::Connection(state))
}

async fn wait(attempt: &mut u32) {
    tokio::time::sleep(backoff::delay(*attempt)).await;
    *attempt = attempt.saturating_add(1);
}

/// The reconnect loop. Returns only when the listener is gone.
async fn follow(link: &Link, emit: &mut impl FnMut(LinkEvent) -> bool) {
    let mut attempt = 0_u32;
    let mut shown: Option<ConnectionState> = None;
    loop {
        // Subscribe first: an event between the snapshot and the subscription cannot be lost.
        let events = match calls::subscribe(link).await {
            Ok(events) => events,
            Err(state) => {
                if !announce(&mut shown, state, emit) {
                    return;
                }
                wait(&mut attempt).await;
                continue;
            }
        };
        let snapshot = calls::status(link).await;
        if !matches!(snapshot.connection, ConnectionState::Online { .. }) {
            if !announce(&mut shown, snapshot.connection, emit) {
                return;
            }
            wait(&mut attempt).await;
            continue;
        }
        attempt = 0;
        let last_sync = snapshot.status.as_ref().and_then(|s| s.last_sync_ms);
        shown = Some(snapshot.connection.clone());
        if !emit(LinkEvent::Snapshot(snapshot)) {
            return;
        }
        if !pump(link, events, last_sync, emit).await {
            return;
        }
        // The stream ended: the daemon went away.
        if !announce(&mut shown, ConnectionState::NotRunning, emit) {
            return;
        }
        wait(&mut attempt).await;
    }
}

async fn next_fetch(fetch: &mut Option<Fetch<'_>>) -> Result<GamesDto, ConnectionState> {
    match fetch {
        Some(f) => f.await,
        None => std::future::pending().await,
    }
}

/// Follows one subscription. `true` when it ended (reconnect), `false` when the listener is
/// gone. The first game fetch starts at once; later ones are triggered by a new
/// `last_sync_ms` and by the end of a session, and coalesce: one in flight, at most one more
/// queued behind it. The fetch is polled in the same select as the events, so events keep
/// flowing while `Games` waits behind a sync.
async fn pump(
    link: &Link,
    events: impl Stream<Item = Result<Event, IpcError>>,
    mut last_sync: Option<u64>,
    emit: &mut impl FnMut(LinkEvent) -> bool,
) -> bool {
    let mut events = pin!(events);
    let mut fetch: Option<Fetch<'_>> = Some(calls::games(link).boxed());
    let mut again = false;
    // Before any fetch worked on this connection, a failure gets exactly one retry (the
    // first trigger may never be followed by another).
    let mut fetched = false;
    let mut retried = false;
    loop {
        let mut trigger = false;
        tokio::select! {
            event = events.next() => {
                let Some(Ok(event)) = event else { return true };
                let out = match event {
                    Event::StatusChanged(status) => {
                        trigger = status.last_sync_ms != last_sync;
                        last_sync = status.last_sync_ms;
                        Some(LinkEvent::Status(StatusDto::from(&status)))
                    }
                    Event::SessionChanged(session) => {
                        trigger = session.is_none();
                        Some(LinkEvent::Session(session.as_ref().map(SessionDto::from)))
                    }
                    Event::TransferProgress(p) => Some(LinkEvent::Progress(ProgressDto::from(&p))),
                    // Ignored in 3b; their facts reach Home through the status counts.
                    _ => None,
                };
                if let Some(out) = out && !emit(out) {
                    return false;
                }
            }
            result = next_fetch(&mut fetch), if fetch.is_some() => {
                fetch = None;
                // A failed fetch emits nothing: the next trigger retries.
                let failed_first = result.is_err() && !fetched && !retried;
                if let Ok(games) = result {
                    fetched = true;
                    if !emit(LinkEvent::Games(games)) {
                        return false;
                    }
                }
                if failed_first {
                    retried = true;
                    again = false;
                    fetch = Some(calls::games(link).boxed());
                } else if std::mem::take(&mut again) {
                    fetch = Some(calls::games(link).boxed());
                }
            }
        }
        if trigger {
            if fetch.is_some() {
                again = true;
            } else {
                fetch = Some(calls::games(link).boxed());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::link::calls::GAMES_TIMEOUT;
    use crate::link::fake::{Answer, Fake, Script, error_body, games_report, status};
    use crate::link::model::{ConnectionState, ProgressDto, StateDto};
    use lemusync_api::{ConflictInfo, Event, TransferPhase, TransferProgress};
    use std::time::Duration;
    use tokio::sync::{Semaphore, mpsc};
    use tokio::time::timeout;

    type Rx = mpsc::UnboundedReceiver<LinkEvent>;

    fn start(link: Link, stop: &Arc<Notify>) -> (tokio::task::JoinHandle<()>, Rx) {
        let (tx, rx) = mpsc::unbounded_channel();
        let task = tokio::spawn(run(link, Arc::clone(stop), move |e| tx.send(e).is_ok()));
        (task, rx)
    }

    async fn next(rx: &mut Rx) -> LinkEvent {
        timeout(Duration::from_secs(60), rx.recv())
            .await
            .expect("an event in time")
            .expect("the watcher is alive")
    }

    /// Lets everything pending run, in virtual time.
    async fn settle() {
        tokio::time::sleep(Duration::from_secs(5)).await;
    }

    fn progress_event(done: u32) -> Event {
        Event::TransferProgress(TransferProgress {
            player: "Ana".into(),
            emulator: "cemu".into(),
            game: "aa".into(),
            phase: TransferPhase::Uploading,
            files_done: done,
            files_total: 9,
            bytes_done: 0,
            bytes_total: 0,
        })
    }

    fn progress_dto(done: u32) -> ProgressDto {
        match LinkEvent::Progress(ProgressDto {
            player: "Ana".into(),
            emulator: "cemu".into(),
            game: "aa".into(),
            phase: crate::link::model::PhaseDto::Uploading,
            files_done: done,
            files_total: 9,
            bytes_done: 0,
            bytes_total: 0,
        }) {
            LinkEvent::Progress(p) => p,
            _ => unreachable!(),
        }
    }

    /// Skips events until the next `Snapshot`/`Games`... returns every event up to and
    /// including the first one matching `until`.
    async fn collect_until(rx: &mut Rx, until: impl Fn(&LinkEvent) -> bool) -> Vec<LinkEvent> {
        let mut out = Vec::new();
        loop {
            let e = next(rx).await;
            let done = until(&e);
            out.push(e);
            if done {
                return out;
            }
        }
    }

    fn is_games(e: &LinkEvent) -> bool {
        matches!(e, LinkEvent::Games(_))
    }

    #[tokio::test(start_paused = true)]
    async fn subscribes_before_it_asks_for_status() {
        let fake = Fake::start(Script::default());
        let stop = Arc::new(Notify::new());
        let (task, mut rx) = start(fake.link.clone(), &stop);
        collect_until(&mut rx, is_games).await;
        let log = fake.log();
        let sub = log
            .iter()
            .position(|w| *w == "subscribe")
            .expect("subscribe");
        let st = log.iter().position(|w| *w == "status").expect("status");
        assert!(sub < st, "{log:?}");
        stop.notify_one();
        task.await.expect("join");
    }

    #[tokio::test(start_paused = true)]
    async fn emits_the_snapshot_then_the_events_and_drops_the_ignored_ones() {
        let fake = Fake::start(Script {
            on_subscribe: vec![
                Event::StatusChanged(status(Some(100))),
                Event::SessionChanged(None),
                progress_event(1),
                Event::ConflictDetected(ConflictInfo {
                    player: "Ana".into(),
                    emulator: "cemu".into(),
                    game: "aa".into(),
                    name: None,
                    heads: vec![],
                }),
                Event::Error(error_body("boom")),
                progress_event(2),
            ],
            ..Script::default()
        });
        let stop = Arc::new(Notify::new());
        let (task, mut rx) = start(fake.link.clone(), &stop);
        // Games is fetched twice (snapshot, then SessionChanged(None)); wait for the end.
        let mut all = collect_until(
            &mut rx,
            |e| matches!(e, LinkEvent::Progress(p) if p.files_done == 2),
        )
        .await;
        settle().await;
        while let Ok(e) = rx.try_recv() {
            all.push(e);
        }
        let non_games: Vec<&LinkEvent> = all.iter().filter(|e| !is_games(e)).collect();
        assert!(matches!(non_games[0], LinkEvent::Snapshot(s)
            if s.status.is_some() && matches!(s.connection, ConnectionState::Online { .. })));
        assert_eq!(
            non_games[1],
            &LinkEvent::Status(crate::link::model::StatusDto::from(&status(Some(100))))
        );
        assert_eq!(non_games[2], &LinkEvent::Session(None));
        assert_eq!(non_games[3], &LinkEvent::Progress(progress_dto(1)));
        assert_eq!(non_games[4], &LinkEvent::Progress(progress_dto(2)));
        assert_eq!(
            non_games.len(),
            5,
            "conflict and error events are dropped: {non_games:?}"
        );
        assert_eq!(all.iter().filter(|e| is_games(e)).count(), 2);
        assert_eq!(fake.count("games"), 2);
        let _ = (StateDto::Synced, games_report());
        stop.notify_one();
        task.await.expect("join");
    }

    #[tokio::test(start_paused = true)]
    async fn events_keep_flowing_while_games_is_held() {
        let gate = Arc::new(Semaphore::new(0));
        let fake = Fake::start(Script {
            games: Answer::Held(Arc::clone(&gate), games_report()),
            ..Script::default()
        });
        let stop = Arc::new(Notify::new());
        let (task, mut rx) = start(fake.link.clone(), &stop);
        assert!(matches!(next(&mut rx).await, LinkEvent::Snapshot(_)));
        settle().await;
        assert_eq!(fake.count("games"), 1);
        fake.push(Event::StatusChanged(status(Some(100))));
        fake.push(progress_event(3));
        assert!(matches!(next(&mut rx).await, LinkEvent::Status(_)));
        assert_eq!(next(&mut rx).await, LinkEvent::Progress(progress_dto(3)));
        assert!(rx.try_recv().is_err(), "Games is not out yet");
        gate.add_permits(1);
        assert!(is_games(&next(&mut rx).await));
        stop.notify_one();
        task.await.expect("join");
    }

    #[tokio::test(start_paused = true)]
    async fn a_games_call_that_never_answers_does_not_stop_the_events() {
        let fake = Fake::start(Script {
            games: Answer::Silent,
            ..Script::default()
        });
        let stop = Arc::new(Notify::new());
        let (task, mut rx) = start(fake.link.clone(), &stop);
        assert!(matches!(next(&mut rx).await, LinkEvent::Snapshot(_)));
        tokio::time::sleep(GAMES_TIMEOUT + Duration::from_secs(1)).await;
        fake.push(progress_event(4));
        assert_eq!(next(&mut rx).await, LinkEvent::Progress(progress_dto(4)));
        assert!(rx.try_recv().is_err(), "a failed fetch emits nothing");
        stop.notify_one();
        task.await.expect("join");
    }

    async fn quiet_started() -> (Fake, Arc<Notify>, tokio::task::JoinHandle<()>, Rx) {
        let fake = Fake::start(Script::default());
        let stop = Arc::new(Notify::new());
        let (task, mut rx) = start(fake.link.clone(), &stop);
        collect_until(&mut rx, is_games).await;
        settle().await;
        assert_eq!(fake.count("games"), 1);
        (fake, stop, task, rx)
    }

    #[tokio::test(start_paused = true)]
    async fn a_failed_first_games_fetch_is_retried_once() {
        let fake = Fake::start(Script {
            games: Answer::FailOnce(
                Arc::new(std::sync::atomic::AtomicBool::new(false)),
                error_body("busy"),
                games_report(),
            ),
            ..Script::default()
        });
        let stop = Arc::new(Notify::new());
        let (task, mut rx) = start(fake.link.clone(), &stop);
        // No `last_sync_ms` change happens: only the retry can produce `Games`.
        collect_until(&mut rx, is_games).await;
        settle().await;
        assert_eq!(fake.count("games"), 2);
        stop.notify_one();
        task.await.expect("join");
    }

    #[tokio::test(start_paused = true)]
    async fn same_last_sync_means_no_extra_games_request() {
        let (fake, stop, task, mut rx) = quiet_started().await;
        fake.push(Event::StatusChanged(status(Some(100))));
        assert!(matches!(next(&mut rx).await, LinkEvent::Status(_)));
        settle().await;
        assert_eq!(fake.count("games"), 1);
        stop.notify_one();
        task.await.expect("join");
    }

    #[tokio::test(start_paused = true)]
    async fn a_new_last_sync_means_exactly_one_extra_request() {
        let (fake, stop, task, mut rx) = quiet_started().await;
        fake.push(Event::StatusChanged(status(Some(200))));
        assert!(matches!(next(&mut rx).await, LinkEvent::Status(_)));
        assert!(is_games(&next(&mut rx).await));
        settle().await;
        assert_eq!(fake.count("games"), 2);
        // The same time again is not new.
        fake.push(Event::StatusChanged(status(Some(200))));
        assert!(matches!(next(&mut rx).await, LinkEvent::Status(_)));
        settle().await;
        assert_eq!(fake.count("games"), 2);
        stop.notify_one();
        task.await.expect("join");
    }

    #[tokio::test(start_paused = true)]
    async fn the_end_of_a_session_means_exactly_one_extra_request() {
        let (fake, stop, task, mut rx) = quiet_started().await;
        fake.push(Event::SessionChanged(Some(crate::link::fake::session())));
        assert!(matches!(next(&mut rx).await, LinkEvent::Session(Some(_))));
        settle().await;
        assert_eq!(fake.count("games"), 1, "a session start fetches nothing");
        fake.push(Event::SessionChanged(None));
        assert_eq!(next(&mut rx).await, LinkEvent::Session(None));
        assert!(is_games(&next(&mut rx).await));
        settle().await;
        assert_eq!(fake.count("games"), 2);
        stop.notify_one();
        task.await.expect("join");
    }

    #[tokio::test(start_paused = true)]
    async fn two_triggers_while_one_is_in_flight_make_exactly_one_more() {
        let gate = Arc::new(Semaphore::new(0));
        let fake = Fake::start(Script {
            games: Answer::Held(Arc::clone(&gate), games_report()),
            ..Script::default()
        });
        let stop = Arc::new(Notify::new());
        let (task, mut rx) = start(fake.link.clone(), &stop);
        assert!(matches!(next(&mut rx).await, LinkEvent::Snapshot(_)));
        settle().await;
        assert_eq!(fake.count("games"), 1, "the first fetch is in flight");
        fake.push(Event::StatusChanged(status(Some(200))));
        fake.push(Event::SessionChanged(None));
        fake.push(Event::StatusChanged(status(Some(300))));
        for _ in 0..3 {
            next(&mut rx).await;
        }
        settle().await;
        assert_eq!(
            fake.count("games"),
            1,
            "nothing starts while one is in flight"
        );
        gate.add_permits(10);
        assert!(is_games(&next(&mut rx).await));
        assert!(is_games(&next(&mut rx).await));
        settle().await;
        assert_eq!(fake.count("games"), 2, "one more, not three");
        assert!(rx.try_recv().is_err());
        stop.notify_one();
        task.await.expect("join");
    }

    #[tokio::test(start_paused = true)]
    async fn a_closed_connection_reconnects_after_the_first_backoff() {
        let (fake, stop, task, mut rx) = quiet_started().await;
        fake.hang_up();
        assert_eq!(
            next(&mut rx).await,
            LinkEvent::Connection(ConnectionState::NotRunning)
        );
        let t0 = tokio::time::Instant::now();
        let evs = collect_until(&mut rx, |e| matches!(e, LinkEvent::Snapshot(_))).await;
        assert_eq!(evs.len(), 1);
        assert_eq!(t0.elapsed(), Duration::from_secs(1));
        assert_eq!(fake.count("subscribe"), 2);
        stop.notify_one();
        task.await.expect("join");
    }

    #[tokio::test(start_paused = true)]
    async fn without_a_daemon_it_announces_once_and_backs_off_up_to_thirty_seconds() {
        let dir = tempfile::tempdir().expect("tempdir");
        let link = Link::at(dir.path().join("daemon.sock"));
        let stop = Arc::new(Notify::new());
        let (task, mut rx) = start(link, &stop);
        assert_eq!(
            next(&mut rx).await,
            LinkEvent::Connection(ConnectionState::NotRunning)
        );
        tokio::time::sleep(Duration::from_secs(200)).await;
        assert!(rx.try_recv().is_err(), "the same state is announced once");
        stop.notify_one();
        task.await.expect("join");
    }

    #[tokio::test(start_paused = true)]
    async fn the_daemon_appearing_later_is_found_after_doubling_waits() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("daemon.sock");
        let stop = Arc::new(Notify::new());
        let t0 = tokio::time::Instant::now();
        let (task, mut rx) = start(Link::at(path.clone()), &stop);
        assert!(matches!(next(&mut rx).await, LinkEvent::Connection(_)));
        // Attempts fail at 0 s, 1 s and 3 s; the next is at 7 s (waits of 1, 2, 4 s).
        tokio::time::sleep(Duration::from_secs(5)).await;
        let _fake = Fake::start_at(&path, Script::default(), None);
        let evs = collect_until(&mut rx, |e| matches!(e, LinkEvent::Snapshot(_))).await;
        assert_eq!(evs.len(), 1, "no repeated announcement: {evs:?}");
        assert_eq!(t0.elapsed(), Duration::from_secs(7));
        stop.notify_one();
        task.await.expect("join");
    }

    #[tokio::test(start_paused = true)]
    async fn stop_ends_the_backoff_sleep_at_once() {
        let dir = tempfile::tempdir().expect("tempdir");
        let stop = Arc::new(Notify::new());
        let (task, mut rx) = start(Link::at(dir.path().join("daemon.sock")), &stop);
        assert!(matches!(next(&mut rx).await, LinkEvent::Connection(_)));
        let t0 = tokio::time::Instant::now();
        stop.notify_one();
        task.await.expect("join");
        assert_eq!(t0.elapsed(), Duration::ZERO);
    }

    #[tokio::test(start_paused = true)]
    async fn stop_ends_a_connected_idle_watcher_at_once() {
        let (_fake, stop, task, _rx) = quiet_started().await;
        let t0 = tokio::time::Instant::now();
        stop.notify_one();
        task.await.expect("join");
        assert_eq!(t0.elapsed(), Duration::ZERO);
    }

    #[tokio::test(start_paused = true)]
    async fn stop_ends_a_watcher_with_games_in_flight_at_once() {
        let fake = Fake::start(Script {
            games: Answer::Silent,
            ..Script::default()
        });
        let stop = Arc::new(Notify::new());
        let (task, mut rx) = start(fake.link.clone(), &stop);
        assert!(matches!(next(&mut rx).await, LinkEvent::Snapshot(_)));
        settle().await;
        let t0 = tokio::time::Instant::now();
        stop.notify_one();
        task.await.expect("join");
        assert_eq!(t0.elapsed(), Duration::ZERO);
    }

    #[tokio::test(start_paused = true)]
    async fn a_listener_that_is_gone_ends_the_watcher_without_another_connect() {
        let fake = Fake::start(Script::default());
        let stop = Arc::new(Notify::new());
        let (tx, rx) = mpsc::unbounded_channel();
        drop(rx);
        let task = tokio::spawn(run(fake.link.clone(), stop, move |e| tx.send(e).is_ok()));
        timeout(Duration::from_secs(60), task)
            .await
            .expect("ends")
            .expect("join");
        assert_eq!(fake.count("subscribe"), 1);
        settle().await;
        assert_eq!(fake.count("subscribe"), 1);
    }
}
