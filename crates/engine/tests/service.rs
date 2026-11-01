//! `engine::Service` end to end on `Store::fs`, a `FakeProbe` and real `notify` over tempdirs.
//! Fake data only. Timings are shortened; every wait is a deadline loop on an observable
//! (an event, the status, a request's answer), never a fixed sleep followed by an assert.
#![allow(clippy::unwrap_used, clippy::panic)] // tests: a failure here is a failed test

mod common;

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use common::{Pc, SAVE, map};
use lemusync_api::{Event, Request, Response, StatusState, StreamAction};
use lemusync_core::schedule::ScheduleConfig;
use lemusync_engine::{Engine, MapChoice, Service, ServiceConfig, ServiceHandle};
use lemusync_store::Faults;
use tokio::sync::broadcast::Receiver;
use tokio::time::{Instant, timeout};

const DEADLINE: Duration = Duration::from_secs(20);

fn cfg() -> ServiceConfig {
    ServiceConfig {
        quiet: Duration::from_millis(200),
        schedule: ScheduleConfig {
            rescan_ms: 60 * 60_000,
            backoff_min_ms: 50,
            backoff_max_ms: 200,
        },
        poll: Duration::from_millis(30),
        renew: Duration::from_millis(500),
        finish: Duration::from_millis(500),
        release: Duration::from_millis(500),
    }
}

/// A PC with a repo, player Ana on account 80000001 and one synced version of the save.
async fn pc(root: &std::path::Path) -> Pc {
    let mut a = Pc::new(&root.join("a"), true, b"first");
    a.engine.backend_add_fs(&root.join("remote")).unwrap();
    a.engine.init("pass phrase").await.unwrap();
    a.engine.players_add("Ana").await.unwrap();
    map(&mut a, "Ana", "80000001", MapChoice::Ask).await;
    a.sync().await;
    a
}

fn save_file(pc: &Pc) -> PathBuf {
    pc.cemu.join(SAVE).join("80000001/0/game_data.sav")
}

struct Running {
    handle: ServiceHandle,
    task: std::thread::JoinHandle<Engine>,
}

fn start(engine: Engine, cfg: ServiceConfig) -> Running {
    start_subscribed(engine, cfg).0
}

/// Also subscribed before anything runs, so no event is missed.
fn start_subscribed(engine: Engine, cfg: ServiceConfig) -> (Running, Receiver<Event>) {
    let (service, handle) = Service::new(engine, cfg);
    let rx = handle.subscribe();
    let task = service.spawn().unwrap();
    (Running { handle, task }, rx)
}

impl Running {
    async fn stop(self) -> Engine {
        self.handle.stop().await;
        let task = self.task;
        timeout(DEADLINE, tokio::task::spawn_blocking(move || task.join()))
            .await
            .unwrap()
            .unwrap()
            .unwrap()
    }
}

/// Waits until the status satisfies `ok`.
async fn until(h: &ServiceHandle, what: &str, ok: impl Fn(&lemusync_api::Status) -> bool) {
    let end = Instant::now() + DEADLINE;
    let mut rx = h.subscribe();
    while !ok(&h.status()) {
        assert!(
            Instant::now() < end,
            "timed out waiting for {what}: {:?}",
            h.status()
        );
        let _ = timeout(Duration::from_millis(100), rx.recv()).await;
    }
}

/// The first event `pick` accepts.
async fn next<T>(rx: &mut Receiver<Event>, pick: impl Fn(&Event) -> Option<T>) -> T {
    let end = Instant::now() + DEADLINE;
    loop {
        let left = end.saturating_duration_since(Instant::now());
        match timeout(left, rx.recv()).await {
            Ok(Ok(e)) => {
                if let Some(t) = pick(&e) {
                    return t;
                }
            }
            Ok(Err(tokio::sync::broadcast::error::RecvError::Lagged(_))) => {}
            other => panic!("no matching event: {other:?}"),
        }
    }
}

/// Writes the save until a sync reaches state `want` (watch registration can lag).
async fn write_until_syncing(h: &ServiceHandle, file: &std::path::Path, body: &[u8]) {
    let mut rx = h.subscribe();
    std::fs::write(file, body).unwrap();
    next(&mut rx, |e| match e {
        Event::StatusChanged(s) if s.state == StatusState::Syncing => Some(()),
        _ => None,
    })
    .await;
}

async fn versions(engine: &mut Engine) -> usize {
    engine
        .history("Ana", "cemu", "101c9400")
        .await
        .unwrap()
        .versions
        .len()
}

/// The daemon's first sync at start has finished.
async fn started(h: &ServiceHandle) {
    until(h, "first sync", |s| s.last_sync_ms.is_some()).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_write_uploads_after_the_quiet_period() {
    let root = tempfile::tempdir().unwrap();
    let a = pc(root.path()).await;
    let file = save_file(&a);
    let svc = start(a.engine, cfg());
    started(&svc.handle).await;
    let first = svc.handle.status().last_sync_ms;
    let mut rx = svc.handle.subscribe();
    write_until_syncing(&svc.handle, &file, b"second").await;
    next(&mut rx, |e| match e {
        Event::StatusChanged(s) if s.state == StatusState::Synced && s.last_sync_ms > first => {
            Some(())
        }
        _ => None,
    })
    .await;
    let mut engine = svc.stop().await;
    assert_eq!(versions(&mut engine).await, 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn status_and_pause_are_answered_while_a_sync_is_stuck() {
    let root = tempfile::tempdir().unwrap();
    let a = pc(root.path()).await;
    let file = save_file(&a);
    let barrier = Arc::new(tokio::sync::Barrier::new(2));
    a.engine
        .arm_pause("upload.before_manifest", Arc::clone(&barrier));
    let svc = start(a.engine, cfg());
    started(&svc.handle).await;
    write_until_syncing(&svc.handle, &file, b"second").await;
    // The engine waits at the barrier: these must not.
    let quick = Duration::from_secs(2);
    let r = timeout(quick, svc.handle.call(Request::Status))
        .await
        .unwrap();
    assert!(matches!(r, Response::Status(_)), "{r:?}");
    let r = timeout(quick, svc.handle.call(Request::Pause { until_ms: None }))
        .await
        .unwrap();
    assert_eq!(r, Response::Paused);
    assert_eq!(svc.handle.status().state, StatusState::Syncing);
    barrier.wait().await;
    until(&svc.handle, "paused after the sync", |s| {
        s.state == StatusState::Paused
    })
    .await;
    let mut engine = svc.stop().await;
    assert_eq!(versions(&mut engine).await, 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn watch_mode_takes_the_lease_uploads_after_exit_and_releases() {
    let root = tempfile::tempdir().unwrap();
    let a = pc(root.path()).await;
    let file = save_file(&a);
    let probe = Arc::clone(&a.probe);
    let svc = start(a.engine, cfg());
    started(&svc.handle).await;
    let mut rx = svc.handle.subscribe();
    probe.set(true);
    // Keep writing until the change while running opened a session (and took the lease).
    let end = Instant::now() + DEADLINE;
    let mut n = 0u32;
    let session = loop {
        assert!(Instant::now() < end, "no session");
        n += 1;
        std::fs::write(&file, n.to_le_bytes()).unwrap();
        if let Ok(Ok(Event::SessionChanged(Some(s)))) =
            timeout(Duration::from_millis(300), rx.recv()).await
        {
            break s;
        }
    };
    assert_eq!(
        (session.player.as_str(), session.game.as_str()),
        ("Ana", "101c9400")
    );
    let Response::LeaseStatus(leases) = svc.handle.call(Request::LeaseStatus).await else {
        panic!("lease status")
    };
    assert!(leases.value.iter().any(|l| l.ours && l.live), "{leases:?}");
    // I3: while the emulator runs nothing is uploaded, even on a manual sync.
    let Response::Sync(r) = svc.handle.call(Request::Sync { dry_run: false }).await else {
        panic!("sync")
    };
    assert!(
        r.streams.iter().all(|s| s.action == StreamAction::Skipped),
        "{r:?}"
    );
    probe.set(false);
    next(&mut rx, |e| {
        matches!(e, Event::SessionChanged(None)).then_some(())
    })
    .await;
    // The upload after exit, then the release.
    let end = Instant::now() + DEADLINE;
    loop {
        let Response::LeaseStatus(leases) = svc.handle.call(Request::LeaseStatus).await else {
            panic!("lease status")
        };
        if leases.value.is_empty() {
            break;
        }
        assert!(Instant::now() < end, "lease not released: {leases:?}");
        let _ = timeout(Duration::from_millis(100), rx.recv()).await;
    }
    let mut engine = svc.stop().await;
    assert_eq!(versions(&mut engine).await, 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_store_outage_backs_off_then_syncs() {
    let root = tempfile::tempdir().unwrap();
    let mut a = pc(root.path()).await;
    std::fs::write(save_file(&a), b"second").unwrap();
    let faults = Faults::default();
    faults.cut_from(0);
    a.engine.set_store_faults(Some(faults.clone()));
    let svc = start(a.engine, cfg());
    until(&svc.handle, "offline", |s| s.state == StatusState::Offline).await;
    let tries = faults.requests();
    // Backoff retries keep coming while the store is down.
    let end = Instant::now() + DEADLINE;
    while faults.requests() <= tries {
        assert!(Instant::now() < end, "no retry");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    faults.heal();
    until(&svc.handle, "synced", |s| {
        s.state == StatusState::Synced && s.last_sync_ms.is_some()
    })
    .await;
    let mut engine = svc.stop().await;
    engine.set_store_faults(None);
    assert_eq!(versions(&mut engine).await, 2);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn while_paused_only_a_manual_sync_runs() {
    let root = tempfile::tempdir().unwrap();
    let a = pc(root.path()).await;
    let file = save_file(&a);
    let svc = start(a.engine, cfg());
    started(&svc.handle).await;
    assert_eq!(
        svc.handle.call(Request::Pause { until_ms: None }).await,
        Response::Paused
    );
    assert_eq!(svc.handle.status().state, StatusState::Paused);
    // The change is seen and queued, not synced.
    let end = Instant::now() + DEADLINE;
    let mut n = 0u32;
    while svc.handle.status().queued == 0 {
        assert!(Instant::now() < end, "change not seen");
        n += 1;
        std::fs::write(&file, n.to_le_bytes()).unwrap();
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    let Response::Sync(r) = svc.handle.call(Request::Sync { dry_run: false }).await else {
        panic!("sync")
    };
    assert!(
        r.streams.iter().any(|s| s.action == StreamAction::Upload),
        "{r:?}"
    );
    assert_eq!(svc.handle.status().state, StatusState::Paused);
    svc.stop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_suspend_mid_transfer_acks_in_time_and_resume_finishes_cleanly() {
    let root = tempfile::tempdir().unwrap();
    let a = pc(root.path()).await;
    let file = save_file(&a);
    // A barrier nobody fills: the upload hangs between blobs and manifest.
    a.engine.arm_pause(
        "upload.before_manifest",
        Arc::new(tokio::sync::Barrier::new(2)),
    );
    let c = cfg();
    let budget = c.finish + c.release + Duration::from_millis(500);
    let svc = start(a.engine, c);
    started(&svc.handle).await;
    write_until_syncing(&svc.handle, &file, b"second").await;
    let t0 = Instant::now();
    svc.handle.suspend().await;
    assert!(t0.elapsed() < budget, "ack took {:?}", t0.elapsed());
    assert!(t0.elapsed() < Duration::from_secs(4));
    svc.handle.resumed().await;
    until(&svc.handle, "synced after resume", |s| {
        s.state == StatusState::Synced && s.queued == 0
    })
    .await;
    let mut engine = svc.stop().await;
    // One new version, one head, the local file as written (I1–I6).
    assert_eq!(versions(&mut engine).await, 2);
    let h = engine.history("Ana", "cemu", "101c9400").await.unwrap();
    assert_eq!(h.versions.iter().filter(|v| v.head).count(), 1);
    assert_eq!(std::fs::read(&file).unwrap(), b"second");
    assert_eq!(engine.journal().unwrap(), vec![]);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn an_emulator_start_warns_of_a_newer_save_elsewhere_and_restores_nothing() {
    let root = tempfile::tempdir().unwrap();
    let a = pc(root.path()).await;
    let file = save_file(&a);
    // Device B joins, takes the same save and uploads a newer one.
    let mut b = Pc::new(&root.path().join("b"), false, b"first");
    common::join(&mut b, &root.path().join("remote")).await;
    map(&mut b, "Ana", "80000001", MapChoice::Replace).await;
    b.sync().await;
    std::fs::write(save_file(&b), b"from b").unwrap();
    b.sync().await;
    // Already running at daemon start: the first sync skips the stream (I3).
    a.probe.set(true);
    let (svc, mut rx) = start_subscribed(a.engine, cfg());
    let n = next(&mut rx, |e| match e {
        Event::RemoteNewer(n) => Some(n.clone()),
        _ => None,
    })
    .await;
    assert_eq!((n.player.as_str(), n.game.as_str()), ("Ana", "101c9400"));
    assert_eq!(std::fs::read(&file).unwrap(), b"first");
    svc.stop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_watch_mode_lease_is_renewed_while_the_emulator_runs() {
    let root = tempfile::tempdir().unwrap();
    let a = pc(root.path()).await;
    let file = save_file(&a);
    let probe = Arc::clone(&a.probe);
    let svc = start(a.engine, cfg());
    started(&svc.handle).await;
    let mut rx = svc.handle.subscribe();
    probe.set(true);
    let end = Instant::now() + DEADLINE;
    let mut n = 0u32;
    loop {
        assert!(Instant::now() < end, "no session");
        n += 1;
        std::fs::write(&file, n.to_le_bytes()).unwrap();
        if let Ok(Ok(Event::SessionChanged(Some(_)))) =
            timeout(Duration::from_millis(300), rx.recv()).await
        {
            break;
        }
    }
    let ours = |r: Response| match r {
        Response::LeaseStatus(l) => l.value.into_iter().find(|l| l.ours && l.live),
        other => panic!("{other:?}"),
    };
    let first = loop {
        if let Some(l) = ours(svc.handle.call(Request::LeaseStatus).await) {
            break l.renewed_at_ms;
        }
        assert!(Instant::now() < end, "no lease");
        let _ = timeout(Duration::from_millis(100), rx.recv()).await;
    };
    // `renew` is 500 ms in these tests: a later renewal shows up while it keeps running.
    let end = Instant::now() + DEADLINE;
    loop {
        let now = ours(svc.handle.call(Request::LeaseStatus).await).map(|l| l.renewed_at_ms);
        if now.is_some_and(|t| t > first) {
            break;
        }
        assert!(Instant::now() < end, "lease never renewed");
        let _ = timeout(Duration::from_millis(100), rx.recv()).await;
    }
    probe.set(false);
    svc.stop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_setup_change_through_the_service_refreshes_its_status_report() {
    let root = tempfile::tempdir().unwrap();
    let a = pc(root.path()).await;
    let svc = start(a.engine, cfg());
    started(&svc.handle).await;
    // Cached now.
    let Response::Status(before) = svc.handle.call(Request::Status).await else {
        panic!("status")
    };
    assert_eq!(before.players, vec!["Ana".to_owned()]);
    assert!(
        before.daemon.is_some(),
        "the daemon overlay is in the report"
    );
    let r = svc
        .handle
        .call(Request::PlayersAdd { name: "Bea".into() })
        .await;
    assert!(matches!(r, Response::Warnings(_)), "{r:?}");
    let Response::Status(after) = svc.handle.call(Request::Status).await else {
        panic!("status")
    };
    assert_eq!(after.players, vec!["Ana".to_owned(), "Bea".to_owned()]);
    // History with two players and none given: the CLI's "which player?".
    let r = svc
        .handle
        .call(Request::History {
            game: "101c9400".into(),
            emu: None,
            player: None,
        })
        .await;
    let Response::Error(e) = r else {
        panic!("{r:?}")
    };
    assert_eq!(
        lemusync_engine::EngineError::from_body(&e).to_string(),
        "player_needed"
    );
    svc.stop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn play_with_a_held_lease_and_nobody_to_ask_fails_with_exit_5() {
    let root = tempfile::tempdir().unwrap();
    let mut a = pc(root.path()).await;
    let mut b = Pc::new(&root.path().join("b"), false, b"first");
    common::join(&mut b, &root.path().join("remote")).await;
    map(&mut b, "Ana", "80000001", MapChoice::Replace).await;
    b.sync().await;
    let (stream, player) = a
        .engine
        .stream_for("Ana", "cemu", "101c9400")
        .await
        .unwrap();
    let _held = b.engine.lease_acquire(stream, player, false).await.unwrap();
    let game = root.path().join("botw.wux");
    std::fs::write(&game, b"fake").unwrap();
    a.engine
        .config_set("emulators.cemu.exe", &["/usr/bin/cemu".into()])
        .unwrap();
    a.engine
        .config_set(
            "emulators.cemu.games.101c9400",
            &[game.display().to_string()],
        )
        .unwrap();
    let svc = start(a.engine, cfg());
    started(&svc.handle).await;
    let r = svc
        .handle
        .call(Request::Play {
            player: "Ana".into(),
            emulator: "cemu".into(),
            game: "101c9400".into(),
            args: vec![],
        })
        .await;
    let Response::Error(e) = r else {
        panic!("{r:?}")
    };
    assert_eq!(
        lemusync_engine::EngineError::from_body(&e).exit_code(),
        5,
        "{e:?}"
    );
    svc.stop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_daemon_with_no_repo_is_not_set_up_not_a_config_error() {
    let root = tempfile::tempdir().unwrap();
    let a = Pc::new(&root.path().join("a"), true, b"first");
    let svc = start(a.engine, cfg());
    until(&svc.handle, "not set up", |s| {
        s.error == Some(lemusync_api::ErrorKind::NotSetUp)
    })
    .await;
    assert_eq!(svc.handle.status().state, StatusState::Error);
    svc.stop().await;
}

type LogBuf = Arc<std::sync::Mutex<Vec<u8>>>;

/// Every log line of this test binary (tests run in parallel: look for a line, never count).
fn log() -> LogBuf {
    static LOG: std::sync::OnceLock<LogBuf> = std::sync::OnceLock::new();
    LOG.get_or_init(|| {
        let buf = LogBuf::default();
        let w = buf.clone();
        let sub = tracing_subscriber::fmt()
            .with_writer(move || LogWriter(w.clone()))
            .finish();
        let _ = tracing::subscriber::set_global_default(sub);
        buf
    })
    .clone()
}

struct LogWriter(LogBuf);

impl std::io::Write for LogWriter {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(b);
        Ok(b.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn logged(what: &str) -> bool {
    String::from_utf8_lossy(&log().lock().unwrap()).contains(what)
}

/// A service whose store rejects the credentials: it stops in `error` (auth).
async fn auth_stopped(root: &std::path::Path) -> (Running, Faults) {
    let mut a = pc(root).await;
    let faults = Faults::default();
    faults.deny_from(0);
    a.engine.set_store_faults(Some(faults.clone()));
    let svc = start(a.engine, cfg());
    until(&svc.handle, "auth stop", |s| {
        s.state == StatusState::Error && s.error == Some(lemusync_api::ErrorKind::Auth)
    })
    .await;
    (svc, faults)
}

fn backend_ok(r: &Response) -> bool {
    let Response::Doctor(d) = r else {
        panic!("doctor: {r:?}")
    };
    d.checks.iter().any(|c| c.name == "backend" && c.ok)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_passing_doctor_ends_the_auth_stop() {
    log();
    let root = tempfile::tempdir().unwrap();
    let (svc, faults) = auth_stopped(root.path()).await;
    faults.heal();
    assert!(backend_ok(&svc.handle.call(Request::Doctor).await));
    until(&svc.handle, "synced after doctor", |s| {
        s.state == StatusState::Synced
    })
    .await;
    assert!(
        logged("status left error for syncing (credentials)"),
        "no state line"
    );
    assert!(logged("sync finished"), "no finished-sync line");
    svc.stop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn new_credentials_end_the_auth_stop() {
    let root = tempfile::tempdir().unwrap();
    let (svc, faults) = auth_stopped(root.path()).await;
    faults.heal();
    let path = root.path().join("remote").display().to_string();
    assert_eq!(
        svc.handle.call(Request::BackendAddFs { path }).await,
        Response::Done
    );
    until(&svc.handle, "synced after new credentials", |s| {
        s.state == StatusState::Synced
    })
    .await;
    svc.stop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_failing_doctor_leaves_the_auth_stop() {
    let root = tempfile::tempdir().unwrap();
    let (svc, faults) = auth_stopped(root.path()).await;
    assert!(!backend_ok(&svc.handle.call(Request::Doctor).await));
    let tries = faults.requests();
    tokio::time::sleep(Duration::from_millis(500)).await;
    assert_eq!(svc.handle.status().state, StatusState::Error);
    assert_eq!(faults.requests(), tries, "a sync ran while stopped");
    svc.stop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_manual_sync_from_error_shows_syncing_before_any_request() {
    let root = tempfile::tempdir().unwrap();
    let (svc, faults) = auth_stopped(root.path()).await;
    faults.heal();
    faults.delay(Duration::from_millis(300));
    // Let the work after the auth stop (the watch-plan refresh) finish first.
    let mut before = faults.requests();
    loop {
        tokio::time::sleep(Duration::from_millis(700)).await;
        let now = faults.requests();
        if now == before {
            break;
        }
        before = now;
    }
    let mut rx = svc.handle.subscribe();
    let h = svc.handle.clone();
    let call = tokio::spawn(async move { h.call(Request::Sync { dry_run: false }).await });
    let first = next(&mut rx, |e| match e {
        Event::StatusChanged(s) => Some(s.state),
        _ => None,
    })
    .await;
    assert_eq!(first, StatusState::Syncing);
    assert_eq!(faults.requests(), before, "the store saw a request first");
    let r = call.await.unwrap();
    assert!(matches!(r, Response::Sync(_)), "{r:?}");
    assert_eq!(svc.handle.status().state, StatusState::Synced);
    svc.stop().await;
}
