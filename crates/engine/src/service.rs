//! The always-on sync service: one task owns the one [`Engine`] and runs its work one command
//! at a time (architecture §Process model, D35). It feeds the pure machines in `core`
//! (quiet period, scheduler and backoff, watch-mode sessions) from the save-folder watcher,
//! the process poller, requests and sleep edges, and sleeps until the next deadline (no busy
//! loop, reliability §Budgets).
//!
//! `Status`, `Pause`, `Resume` and `Subscribe` are answered from the service's own state at
//! once, even while engine work runs: every engine call goes through `drive`, which keeps
//! serving those while the call is in flight and queues everything else.
//!
//! Watch mode never weakens I3: the engine skips streams whose emulator runs, the service only
//! queues a stream for upload once its emulator exited and its files were quiet, and a live
//! lease held elsewhere is never taken over: the service warns and uploads anyway at exit
//! (a conflict at worst, D6).

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::future::Future;
use std::time::Duration;

use std::sync::Arc;

use lemusync_api::{
    DaemonStatus, ErrorBody, Event, LeaseInfo, Request, Response, Session, Status, StatusReport,
    StreamAction, SyncReport, TransferProgress,
};
use lemusync_core::StreamId;
use lemusync_core::schedule::{Action, Quiet, ScheduleConfig, Scheduler, SyncOutcome, Trigger};
use lemusync_core::watch_session::{SessionAction, WatchSessions};
use tokio::sync::{broadcast, mpsc, oneshot, watch};

use crate::events::{self, StatusModel};
use crate::remote_play::{ChannelPrompt, PROMPT_TIMEOUT};
use crate::{
    Acquire, DEFAULT_POLL, Engine, EngineError, Launcher, LeaseHandle, OsLauncher, PlayCall,
    ProcessPoller, SaveWatcher, SyncOptions, WatchTarget, now_ms, redact, remote,
};

/// Timings of the service. Tests shorten them.
#[derive(Clone, Debug)]
pub struct ServiceConfig {
    /// A stream syncs once its files were quiet this long.
    pub quiet: Duration,
    /// Rescan and backoff timings.
    pub schedule: ScheduleConfig,
    /// Process poll interval for watch mode.
    pub poll: Duration,
    /// Watch-mode lease renewal interval.
    pub renew: Duration,
    /// On a suspend or stop, how long running engine work may take to finish before it is
    /// stopped at its next await (the journal covers the rest, I1–I6).
    pub finish: Duration,
    /// Time left after `finish` to release leases before the suspend is acknowledged.
    pub release: Duration,
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            quiet: Duration::from_secs(10),
            schedule: ScheduleConfig::default(),
            poll: DEFAULT_POLL,
            renew: Duration::from_millis(lemusync_core::lease::RENEW_MS.unsigned_abs()),
            // 3 s + 1 s stays within the 4 s suspend budget (reliability §Sleep and resume).
            finish: Duration::from_secs(3),
            release: Duration::from_secs(1),
        }
    }
}

enum Command {
    Request(Request, oneshot::Sender<Response>),
    Play(PlayCall, oneshot::Sender<Response>),
    Suspend(oneshot::Sender<()>),
    Resumed,
    Stop(oneshot::Sender<()>),
}

/// The handle every client of the service uses (IPC connections, the tray, sleep hooks).
/// Cheap to clone.
#[derive(Clone)]
pub struct ServiceHandle {
    tx: mpsc::Sender<Command>,
    events: broadcast::Sender<Event>,
    status: watch::Receiver<Status>,
}

impl ServiceHandle {
    /// Runs one request. Once the service stopped: an error with code `shutting_down`.
    pub async fn call(&self, request: Request) -> Response {
        let (tx, rx) = oneshot::channel();
        if self.tx.send(Command::Request(request, tx)).await.is_err() {
            return stopped();
        }
        rx.await.unwrap_or_else(|_| stopped())
    }

    /// Runs `play` (launcher mode) on the service's engine. Steps and questions go to
    /// `call.io`; without one the non-interactive policy applies (a held lease or a conflict
    /// fails, nothing launches). A `Request::Play` through [`ServiceHandle::call`] is the
    /// same with no `io`.
    pub async fn play(&self, call: PlayCall) -> Response {
        let (tx, rx) = oneshot::channel();
        if self.tx.send(Command::Play(call, tx)).await.is_err() {
            return stopped();
        }
        rx.await.unwrap_or_else(|_| stopped())
    }

    /// Every event from now on.
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.events.subscribe()
    }

    /// The current status, at once.
    #[must_use]
    pub fn status(&self) -> Status {
        self.status.borrow().clone()
    }

    /// The machine is about to sleep. Returns once running work finished or stopped at its
    /// next atomic step and leases were released (within `finish + release`), or at once when
    /// the service is gone. The caller then acknowledges the suspend.
    pub async fn suspend(&self) {
        let (tx, rx) = oneshot::channel();
        if self.tx.send(Command::Suspend(tx)).await.is_ok() {
            let _ = rx.await;
        }
    }

    /// The machine woke up: rescan and sync (`Trigger::Resume`).
    pub async fn resumed(&self) {
        let _ = self.tx.send(Command::Resumed).await;
    }

    /// Graceful stop: finish (or stop) the running step, release leases, end `run`.
    pub async fn stop(&self) {
        let (tx, rx) = oneshot::channel();
        if self.tx.send(Command::Stop(tx)).await.is_ok() {
            let _ = rx.await;
        }
    }
}

fn stopped() -> Response {
    Response::Error(ErrorBody {
        code: "shutting_down".into(),
        message: "the daemon is stopping".into(),
        detail: None,
    })
}

fn error_body(e: &EngineError) -> ErrorBody {
    e.to_body()
}

fn bad_request(why: &str) -> Response {
    Response::Error(ErrorBody {
        code: "bad_request".into(),
        message: why.into(),
        detail: None,
    })
}

/// Owns the engine. Build with [`Service::new`], then await [`Service::run`].
pub struct Service {
    engine: Engine,
    cfg: ServiceConfig,
    rx: mpsc::Receiver<Command>,
    model: StatusModel,
    launcher: Arc<dyn Launcher>,
}

/// Work waiting for the engine.
enum Work {
    Change(StreamId),
    Running(String, bool),
    Request(Request, oneshot::Sender<Response>),
    Play(PlayCall, oneshot::Sender<Response>),
}

/// The channels `drive` listens on besides the engine call.
struct Inputs {
    cmd: mpsc::Receiver<Command>,
    watch: mpsc::Receiver<StreamId>,
    poll: mpsc::Receiver<(String, bool)>,
    progress: mpsc::Receiver<TransferProgress>,
    connection: mpsc::Receiver<lemusync_api::ConnectionEvent>,
}

/// Everything the loop knows besides the engine, so an engine call can run while this is
/// updated.
struct Core {
    cfg: ServiceConfig,
    model: StatusModel,
    scheduler: Scheduler,
    quiet: Quiet,
    sessions: WatchSessions,
    targets: Vec<WatchTarget>,
    /// Our watch-mode leases.
    leases: BTreeMap<StreamId, LeaseHandle>,
    /// Streams changed during an emulator run: lease released after their next upload.
    release_after: BTreeSet<StreamId>,
    /// Streams whose live lease another device holds (the `foreign_leases` fact).
    foreign: BTreeSet<StreamId>,
    /// Streams changed and not yet synced (the `queued` fact).
    dirty: BTreeSet<StreamId>,
    /// Conflicts already announced, by `(player, emulator, game)`.
    known_conflicts: BTreeSet<(String, String, String)>,
    /// Last `status` report, answered at once.
    report: Option<StatusReport>,
    /// The running session overlay, by emulator.
    session: Option<Session>,
    work: VecDeque<Work>,
    /// Non-dry `Sync` requests waiting for the next sync to finish.
    sync_waiters: Vec<oneshot::Sender<Response>>,
    suspend: Option<oneshot::Sender<()>>,
    suspended: bool,
    stopping: bool,
    /// Told once `run` has released everything.
    stop: Vec<oneshot::Sender<()>>,
    /// Running engine work must end by then (suspend or stop), as Unix ms.
    cancel_at: Option<u64>,
    renew_at: Option<u64>,
    /// A request changed the setup: reload the watched folders and the status report.
    refresh: bool,
    launcher: Arc<dyn Launcher>,
}

impl Service {
    /// The service and its handle. Nothing runs until [`Service::run`] is awaited.
    #[must_use]
    pub fn new(engine: Engine, cfg: ServiceConfig) -> (Service, ServiceHandle) {
        let (tx, rx) = mpsc::channel(64);
        let model = StatusModel::new();
        let handle = ServiceHandle {
            tx,
            events: model.sender(),
            status: model.watch(),
        };
        (
            Service {
                engine,
                cfg,
                rx,
                model,
                launcher: Arc::new(OsLauncher),
            },
            handle,
        )
    }

    /// What `play` launches with (tests use a fake).
    pub fn set_launcher(&mut self, launcher: Arc<dyn Launcher>) {
        self.launcher = launcher;
    }

    /// Runs [`Service::run`] on its own thread with a current-thread tokio runtime: the run
    /// future isn't `Send` (the state DB connection isn't `Sync`), so it can't be
    /// `tokio::spawn`ed. The thread's result is the engine, after the stop.
    ///
    /// # Errors
    /// `Io` when the thread or its runtime can't start.
    pub fn spawn(self) -> Result<std::thread::JoinHandle<Engine>, EngineError> {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        Ok(std::thread::Builder::new()
            .name("lemusync-engine".into())
            .spawn(move || rt.block_on(self.run()))?)
    }

    /// Runs until [`ServiceHandle::stop`] (or a `Shutdown` request), then releases leases and
    /// returns the engine (dropping it releases the state lock).
    #[allow(clippy::too_many_lines)] // the one loop, kept in one place
    pub async fn run(self) -> Engine {
        let Service {
            mut engine,
            cfg,
            rx,
            model,
            launcher,
        } = self;
        let (progress_tx, progress) = mpsc::channel(256);
        engine.set_progress(std::sync::Arc::new(move |p: &TransferProgress| {
            // Lossy on purpose: progress is advisory and must never block a transfer.
            let _ = progress_tx.try_send(p.clone());
        }));
        let (conn_tx, connection) = mpsc::channel(64);
        engine.set_connection_sink(std::sync::Arc::new(
            move |c: &lemusync_api::ConnectionEvent| {
                let _ = conn_tx.try_send(c.clone());
            },
        ));
        let (watch_tx, watch) = mpsc::channel(256);
        let mut watcher = match SaveWatcher::start(Vec::new(), watch_tx) {
            Ok(w) => Some(w),
            Err(e) => {
                tracing::warn!("watcher unavailable, rescans only: {e}");
                None
            }
        };
        let (poll_tx, poll) = mpsc::channel(64);
        let mut names = engine.process_names_by_emulator().unwrap_or_default();
        let mut poller = ProcessPoller::start(
            engine.process_probe(),
            names.clone(),
            cfg.poll,
            poll_tx.clone(),
        );
        let mut inputs = Inputs {
            cmd: rx,
            watch,
            poll,
            progress,
            connection,
        };
        let mut core = Core {
            scheduler: Scheduler::new(cfg.schedule),
            quiet: Quiet::new(u64::try_from(cfg.quiet.as_millis()).unwrap_or(u64::MAX)),
            cfg,
            model,
            sessions: WatchSessions::new(),
            targets: Vec::new(),
            leases: BTreeMap::new(),
            release_after: BTreeSet::new(),
            foreign: BTreeSet::new(),
            dirty: BTreeSet::new(),
            known_conflicts: BTreeSet::new(),
            report: None,
            session: None,
            work: VecDeque::new(),
            sync_waiters: Vec::new(),
            suspend: None,
            suspended: false,
            stopping: false,
            stop: Vec::new(),
            cancel_at: None,
            renew_at: None,
            refresh: false,
            launcher,
        };
        refresh_targets(&mut engine, &mut core, &mut inputs, watcher.as_mut()).await;

        loop {
            if core.stopping {
                break;
            }
            if let Some(ack) = core.suspend.take() {
                release_all(&mut engine, &mut core, &mut inputs).await;
                core.suspended = true;
                core.cancel_at = None;
                let _ = ack.send(());
                continue;
            }
            let now = now_ms();
            for s in core.quiet.take_due(now) {
                core.scheduler.on_trigger(Trigger::Change(s), now);
            }
            if let Some(w) = core.work.pop_front() {
                do_work(w, &mut engine, &mut core, &mut inputs).await;
                if std::mem::take(&mut core.refresh) {
                    if let Some(Ok(report)) = drive(engine.status(), &mut core, &mut inputs).await {
                        core.report = Some(report);
                    }
                    if refresh_targets(&mut engine, &mut core, &mut inputs, watcher.as_mut()).await
                    {
                        let fresh = engine.process_names_by_emulator().unwrap_or_default();
                        if fresh != names {
                            names = fresh;
                            poller = ProcessPoller::start(
                                engine.process_probe(),
                                names.clone(),
                                core.cfg.poll,
                                poll_tx.clone(),
                            );
                        }
                    }
                }
                continue;
            }
            if core.renew_at.is_some_and(|t| t <= now) {
                renew_all(&mut engine, &mut core, &mut inputs).await;
                continue;
            }
            let next = if core.suspended {
                Action::Idle
            } else {
                core.scheduler.next_action(now)
            };
            let mut wake = match next {
                Action::SyncNow => {
                    run_sync(&mut engine, &mut core, &mut inputs, false).await;
                    let changed =
                        refresh_targets(&mut engine, &mut core, &mut inputs, watcher.as_mut())
                            .await;
                    if changed {
                        let fresh = engine.process_names_by_emulator().unwrap_or_default();
                        if fresh != names {
                            names = fresh;
                            poller = ProcessPoller::start(
                                engine.process_probe(),
                                names.clone(),
                                core.cfg.poll,
                                poll_tx.clone(),
                            );
                        }
                    }
                    continue;
                }
                Action::WaitUntil(t) => Some(t),
                Action::Idle => None,
            };
            for t in [core.quiet.next_deadline(), core.renew_at]
                .into_iter()
                .flatten()
            {
                wake = Some(wake.map_or(t, |w| w.min(t)));
            }
            let sleep = wake.map(|t| Duration::from_millis(t.saturating_sub(now_ms())));
            tokio::select! {
                c = inputs.cmd.recv() => match c {
                    Some(c) => core.command(c),
                    None => break,
                },
                Some(s) = inputs.watch.recv() => core.work.push_back(Work::Change(s)),
                Some((e, r)) = inputs.poll.recv() => core.work.push_back(Work::Running(e, r)),
                Some(p) = inputs.progress.recv() => core.model.progress(&p),
                Some(c) = inputs.connection.recv() => core.connection(c),
                () = sleep_for(sleep) => {}
            }
        }
        drop(poller);
        drop(watcher);
        release_all(&mut engine, &mut core, &mut inputs).await;
        for w in core.sync_waiters.drain(..) {
            let _ = w.send(stopped());
        }
        for w in core.work.drain(..).filter_map(|w| match w {
            Work::Request(_, tx) | Work::Play(_, tx) => Some(tx),
            _ => None,
        }) {
            let _ = w.send(stopped());
        }
        for s in core.stop.drain(..) {
            let _ = s.send(());
        }
        engine
    }
}

async fn sleep_for(d: Option<Duration>) {
    match d {
        Some(d) => tokio::time::sleep(d).await,
        None => std::future::pending().await,
    }
}

impl Core {
    /// A command that arrives while nothing or something runs. Never touches the engine.
    fn command(&mut self, c: Command) {
        let now = now_ms();
        match c {
            Command::Request(r, tx) => match r {
                Request::Pause { until_ms } => {
                    self.scheduler.pause(until_ms);
                    self.model.update(|f| {
                        f.paused = true;
                        f.paused_until_ms = until_ms;
                    });
                    let _ = tx.send(Response::Paused);
                }
                Request::Resume => {
                    self.scheduler.resume();
                    self.model.update(|f| {
                        f.paused = false;
                        f.paused_until_ms = None;
                    });
                    let _ = tx.send(Response::Resumed);
                }
                Request::Subscribe => {
                    let _ = tx.send(Response::Subscribed);
                }
                Request::Status if self.report.is_some() => {
                    if let Some(report) = self.report.clone() {
                        let _ = tx.send(Response::Status(self.with_daemon(report)));
                    }
                }
                Request::Play {
                    player,
                    emulator,
                    game,
                    args,
                } => self.work.push_back(Work::Play(
                    PlayCall {
                        player,
                        emulator,
                        game,
                        args,
                        io: None,
                    },
                    tx,
                )),
                Request::Answer(_) => {
                    let _ = tx.send(bad_request("no question is waiting for an answer"));
                }
                Request::Sync { dry_run: false } => {
                    self.scheduler.on_trigger(Trigger::Manual, now);
                    self.sync_waiters.push(tx);
                }
                Request::Shutdown => {
                    let _ = tx.send(Response::ShuttingDown);
                    self.begin_stop(None, now);
                }
                other => self.work.push_back(Work::Request(other, tx)),
            },
            Command::Play(call, tx) => self.work.push_back(Work::Play(call, tx)),
            Command::Suspend(ack) => {
                self.suspend = Some(ack);
                self.cancel_at = Some(now.saturating_add(ms(self.cfg.finish)));
            }
            Command::Resumed => {
                self.suspended = false;
                self.scheduler.on_trigger(Trigger::Resume, now);
                // Leases may have expired while asleep: take them again for running sessions.
                self.renew_at = Some(now);
            }
            Command::Stop(done) => self.begin_stop(Some(done), now),
        }
    }

    fn begin_stop(&mut self, done: Option<oneshot::Sender<()>>, now: u64) {
        self.stopping = true;
        self.stop.extend(done);
        if self.cancel_at.is_none() {
            self.cancel_at = Some(now.saturating_add(ms(self.cfg.finish)));
        }
    }

    /// A status report with the daemon's live state (architecture §Process model).
    fn with_daemon(&self, mut report: StatusReport) -> StatusReport {
        report.daemon = Some(DaemonStatus {
            version: env!("CARGO_PKG_VERSION").into(),
            status: self.model.status(),
            session: self.session.clone(),
        });
        report
    }

    fn connection(&mut self, c: lemusync_api::ConnectionEvent) {
        if c == lemusync_api::ConnectionEvent::Recovered {
            self.model.update(|f| f.offline = false);
        }
        self.model.emit(Event::Connection(c));
    }

    fn target(&self, s: &StreamId) -> Option<&WatchTarget> {
        self.targets.iter().find(|t| t.stream == *s)
    }

    fn set_counts(&mut self) {
        let (queued, foreign) = (
            u32::try_from(self.dirty.len()).unwrap_or(u32::MAX),
            u32::try_from(self.foreign.len()).unwrap_or(u32::MAX),
        );
        self.model.update(|f| {
            f.queued = queued;
            f.foreign_leases = foreign;
        });
    }
}

fn ms(d: Duration) -> u64 {
    u64::try_from(d.as_millis()).unwrap_or(u64::MAX)
}

/// Runs one engine call while serving quick commands and queueing the rest. `None` when the
/// call was stopped at its next await because a suspend or stop ran out of time.
async fn drive<F: Future>(fut: F, core: &mut Core, inputs: &mut Inputs) -> Option<F::Output> {
    tokio::pin!(fut);
    loop {
        let left = core
            .cancel_at
            .map(|t| Duration::from_millis(t.saturating_sub(now_ms())));
        tokio::select! {
            biased;
            out = &mut fut => {
                drain_progress(core, inputs);
                return Some(out);
            }
            Some(c) = inputs.cmd.recv() => core.command(c),
            Some(s) = inputs.watch.recv() => core.work.push_back(Work::Change(s)),
            Some((e, r)) = inputs.poll.recv() => core.work.push_back(Work::Running(e, r)),
            Some(p) = inputs.progress.recv() => core.model.progress(&p),
            Some(c) = inputs.connection.recv() => core.connection(c),
            () = sleep_for(left) => {
                tracing::warn!("stopped running work at its next step (suspend or stop)");
                return None;
            }
        }
    }
}

fn drain_progress(core: &mut Core, inputs: &mut Inputs) {
    while let Ok(p) = inputs.progress.try_recv() {
        core.model.progress(&p);
    }
}

/// One whole sync (`dry_run` only for a `Sync { dry_run: true }` request, which doesn't touch
/// the scheduler). Returns the report for a waiting request.
async fn run_sync(
    engine: &mut Engine,
    core: &mut Core,
    inputs: &mut Inputs,
    dry_run: bool,
) -> Option<Result<SyncReport, EngineError>> {
    if !dry_run {
        core.scheduler.on_sync_started();
    }
    let before: BTreeSet<StreamId> = core.dirty.clone();
    core.model.sync_started();
    let out = drive(engine.sync(SyncOptions { dry_run }), core, inputs).await;
    core.model.sync_ended();
    let now = now_ms();
    if dry_run {
        return out;
    }
    let jitter = f64::from(lemusync_crypto::random_bytes::<1>().map_or(0, |b| b[0])) / 256.0;
    let response = match &out {
        Some(Ok(report)) => {
            core.scheduler.on_sync_done(SyncOutcome::Ok, now, jitter);
            core.dirty.retain(|s| !before.contains(s));
            core.model.update(|f| {
                f.error = None;
                f.offline = false;
                f.last_sync_ms = Some(now);
            });
            core.set_counts();
            after_sync(engine, core, inputs, report).await;
            Response::Sync(report.clone())
        }
        Some(Err(e)) => {
            let outcome = events::outcome(e);
            core.scheduler.on_sync_done(outcome, now, jitter);
            let kind = events::error_kind(e);
            core.model.update(|f| {
                // A keystore that isn't open yet is a wait, not a lost connection.
                f.offline = outcome == SyncOutcome::Transient
                    && !matches!(e, EngineError::SecretsUnavailable(_));
                f.error = kind;
            });
            tracing::warn!("sync failed ({}): {}", e.code(), redact(&e.to_string()));
            core.model.emit(Event::Error(error_body(e)));
            Response::Error(error_body(e))
        }
        None => {
            // Stopped for a suspend or stop: try again on resume (no backoff, nothing failed).
            core.scheduler.on_sync_done(SyncOutcome::Other, now, jitter);
            core.scheduler.on_trigger(Trigger::Resume, now);
            stopped()
        }
    };
    for w in core.sync_waiters.drain(..) {
        let _ = w.send(response.clone());
    }
    out
}

/// After a good sync: conflicts, leases to release, the cached status report.
async fn after_sync(engine: &mut Engine, core: &mut Core, inputs: &mut Inputs, r: &SyncReport) {
    let conflicted = r.streams.iter().any(|s| {
        matches!(
            s.action,
            StreamAction::Conflict | StreamAction::UploadThenConflict
        )
    });
    let mut count = 0;
    if conflicted {
        if let Some(Ok((list, _))) = drive(engine.conflicts(), core, inputs).await {
            count = u32::try_from(list.len()).unwrap_or(u32::MAX);
            for c in list {
                let key = (c.player.clone(), c.emulator.clone(), c.game.clone());
                if core.known_conflicts.insert(key) {
                    core.model.emit(Event::ConflictDetected(c));
                }
            }
        }
    } else {
        core.known_conflicts.clear();
    }
    core.model.update(|f| f.conflicts = count);
    // Watch mode: the upload after exit happened; release those leases (D6: never needed for
    // safety, so a failure only lets the lease go stale).
    let done: Vec<StreamId> = core
        .release_after
        .iter()
        .filter(|s| {
            core.target(s)
                .is_none_or(|t| !core.sessions.is_running(&t.emulator))
        })
        .copied()
        .collect();
    for s in done {
        core.release_after.remove(&s);
        core.foreign.remove(&s);
        if let Some(h) = core.leases.remove(&s)
            && let Some(Err(e)) = drive(engine.lease_release(h), core, inputs).await
        {
            tracing::warn!("lease release failed: {}", redact(&e.to_string()));
        }
    }
    if core.leases.is_empty() {
        core.renew_at = None;
    }
    core.set_counts();
    if let Some(Ok(report)) = drive(engine.status(), core, inputs).await {
        core.report = Some(report);
    }
}

/// Reloads the watched folders. Storage unreachable → keep the last list. `true` if the list
/// changed.
async fn refresh_targets(
    engine: &mut Engine,
    core: &mut Core,
    inputs: &mut Inputs,
    watcher: Option<&mut SaveWatcher>,
) -> bool {
    match drive(engine.watch_plan(), core, inputs).await {
        Some(Ok(targets)) if targets != core.targets => {
            if let Some(w) = watcher {
                w.set_folders(
                    targets
                        .iter()
                        .map(|t| (t.stream, t.folder.clone()))
                        .collect(),
                );
            }
            core.targets = targets;
            true
        }
        Some(Err(e)) => {
            tracing::info!(
                "watch targets not refreshed ({}), keeping the last {}",
                e.code(),
                core.targets.len()
            );
            false
        }
        Some(Ok(_)) | None => false,
    }
}

async fn do_work(w: Work, engine: &mut Engine, core: &mut Core, inputs: &mut Inputs) {
    let now = now_ms();
    match w {
        Work::Change(s) => {
            core.dirty.insert(s);
            let emulator = core.target(&s).map(|t| t.emulator.clone());
            match emulator {
                Some(emu) => {
                    for a in core.sessions.on_change(s, &emu, now) {
                        session_action(a, engine, core, inputs).await;
                    }
                }
                // Not a known target (yet): a plain change, the full sync covers it.
                None => core.quiet.touch(s, now),
            }
            core.set_counts();
        }
        Work::Running(emu, running) => {
            for a in core.sessions.on_running(&emu, running, now) {
                session_action(a, engine, core, inputs).await;
            }
            if !running {
                if core.session.as_ref().is_some_and(|s| s.emulator == emu) {
                    core.session = None;
                    core.model.emit(Event::SessionChanged(None));
                }
                core.scheduler.on_trigger(Trigger::EmulatorExit(emu), now);
            }
        }
        Work::Request(r, tx) => {
            let refresh = remote::changes_setup(&r);
            let resp = request(r, engine, core, inputs).await;
            if refresh {
                // A `status` arriving before the refresh must not get the old report.
                core.report = None;
                core.refresh = true;
            }
            let _ = tx.send(resp);
        }
        Work::Play(call, tx) => {
            let resp = play(call, engine, core, inputs).await;
            let _ = tx.send(resp);
        }
    }
}

/// Launcher mode for a client or the tray. While it runs the service keeps answering
/// `status`, `pause` and `resume`, and a suspend is acknowledged at once (the session keeps
/// its own lease renewals); only a stop that runs out of time ends it early. The process
/// poller's edges for this emulator during the session are dropped: the session already
/// took the lease and uploaded.
async fn play(
    call: PlayCall,
    engine: &mut Engine,
    core: &mut Core,
    inputs: &mut Inputs,
) -> Response {
    let prompt = ChannelPrompt {
        io: call.io,
        timeout: PROMPT_TIMEOUT,
    };
    let args = remote::os_args(&call.args);
    let launcher = core.launcher.clone();
    let name = engine
        .game_names()
        .get(&(call.emulator.clone(), call.game.to_lowercase()))
        .cloned();
    let session = Session {
        game: call.game.clone(),
        name,
        emulator: call.emulator.clone(),
        player: call.player.clone(),
        device: engine.local_config().device.name.clone(),
        started_ms: now_ms(),
    };
    core.session = Some(session.clone());
    core.model.emit(Event::SessionChanged(Some(session)));
    let out = drive_play(
        engine.play(
            &call.player,
            &call.emulator,
            &call.game,
            &args,
            &prompt,
            launcher.as_ref(),
        ),
        core,
        inputs,
    )
    .await;
    core.session = None;
    core.model.emit(Event::SessionChanged(None));
    core.work
        .retain(|w| !matches!(w, Work::Running(e, _) if *e == call.emulator));
    core.refresh = true;
    match out {
        Some(Ok(report)) => Response::Play(report),
        Some(Err(e)) => Response::Error(error_body(&e)),
        None => stopped(),
    }
}

/// [`drive`] for a play session: suspends are acknowledged at once, and only a stop's
/// deadline ends the call.
async fn drive_play<F: Future>(fut: F, core: &mut Core, inputs: &mut Inputs) -> Option<F::Output> {
    tokio::pin!(fut);
    loop {
        if let Some(ack) = core.suspend.take() {
            core.suspended = true;
            core.cancel_at = None;
            let _ = ack.send(());
        }
        let left = core
            .cancel_at
            .filter(|_| core.stopping)
            .map(|t| Duration::from_millis(t.saturating_sub(now_ms())));
        tokio::select! {
            biased;
            out = &mut fut => {
                drain_progress(core, inputs);
                return Some(out);
            }
            Some(c) = inputs.cmd.recv() => core.command(c),
            Some(s) = inputs.watch.recv() => core.work.push_back(Work::Change(s)),
            Some((e, r)) = inputs.poll.recv() => core.work.push_back(Work::Running(e, r)),
            Some(p) = inputs.progress.recv() => core.model.progress(&p),
            Some(c) = inputs.connection.recv() => core.connection(c),
            () = sleep_for(left) => {
                tracing::warn!("stopped a play session at its next step (daemon stop)");
                return None;
            }
        }
    }
}

async fn session_action(
    a: SessionAction,
    engine: &mut Engine,
    core: &mut Core,
    inputs: &mut Inputs,
) {
    let now = now_ms();
    match a {
        SessionAction::CheckRemote(emu) => {
            match drive(engine.remote_newer(&emu), core, inputs).await {
                Some(Ok(list)) => {
                    for n in list {
                        core.model.emit(Event::RemoteNewer(n));
                    }
                }
                Some(Err(e)) => tracing::info!("remote check skipped ({})", e.code()),
                None => {}
            }
        }
        SessionAction::TakeLease(s) => {
            core.release_after.insert(s);
            let Some(t) = core.target(&s).cloned() else {
                return;
            };
            if core.session.is_none() {
                let session = Session {
                    game: t.game.clone(),
                    name: None,
                    emulator: t.emulator.clone(),
                    player: t.player_name.clone(),
                    device: engine.local_config().device.name.clone(),
                    started_ms: now,
                };
                core.session = Some(session.clone());
                core.model.emit(Event::SessionChanged(Some(session)));
            }
            take_lease(&t, engine, core, inputs).await;
        }
        SessionAction::UploadAfterQuiet(s) => {
            core.dirty.insert(s);
            core.quiet.touch(s, now);
        }
        SessionAction::ReleaseLease(s) => {
            core.release_after.insert(s);
        }
    }
}

/// Non-interactive: never a takeover. Held elsewhere → `LeaseWarning`; upload anyway at exit.
async fn take_lease(t: &WatchTarget, engine: &mut Engine, core: &mut Core, inputs: &mut Inputs) {
    match drive(
        engine.lease_acquire(t.stream, t.player, false),
        core,
        inputs,
    )
    .await
    {
        Some(Ok((Acquire::Ours(h), _))) => {
            core.foreign.remove(&t.stream);
            core.leases.insert(t.stream, h);
            if core.renew_at.is_none() {
                core.renew_at = Some(now_ms().saturating_add(ms(core.cfg.renew)));
            }
        }
        Some(Ok((
            Acquire::HeldElsewhere {
                device,
                device_name,
                renewed_at_ms,
                stale,
            },
            _,
        ))) => {
            if !stale {
                core.foreign.insert(t.stream);
            }
            core.model.emit(Event::LeaseWarning(LeaseInfo {
                player: t.player_name.clone(),
                emulator: t.emulator.clone(),
                game: t.game.clone(),
                device_id: device.to_string(),
                device: device_name,
                renewed_at_ms,
                live: !stale,
                ours: false,
            }));
        }
        Some(Err(e)) => tracing::warn!("lease not taken ({}); uploading anyway", e.code()),
        None => {}
    }
    core.set_counts();
}

async fn renew_all(engine: &mut Engine, core: &mut Core, inputs: &mut Inputs) {
    core.renew_at = None;
    // Running sessions without a lease (lost while asleep, or never taken): try again.
    let missing: Vec<WatchTarget> = core
        .release_after
        .iter()
        .filter(|s| !core.leases.contains_key(s) && !core.foreign.contains(s))
        .filter_map(|s| core.target(s).cloned())
        .filter(|t| core.sessions.is_running(&t.emulator))
        .collect();
    for t in &missing {
        take_lease(t, engine, core, inputs).await;
    }
    let streams: Vec<StreamId> = core.leases.keys().copied().collect();
    for s in streams {
        let Some(mut h) = core.leases.remove(&s) else {
            continue;
        };
        if let Some(Err(e)) = drive(engine.lease_renew(&mut h), core, inputs).await {
            tracing::warn!("lease renewal failed ({}); retrying", e.code());
        }
        core.leases.insert(s, h);
    }
    if !core.leases.is_empty() {
        core.renew_at = Some(now_ms().saturating_add(ms(core.cfg.renew)));
    }
}

/// Releases every lease we hold, within `release` (suspend, stop). The network may be gone:
/// a lease left behind goes stale after its TTL.
async fn release_all(engine: &mut Engine, core: &mut Core, inputs: &mut Inputs) {
    let budget = core.cfg.release;
    let leases = std::mem::take(&mut core.leases);
    core.renew_at = None;
    if leases.is_empty() {
        return;
    }
    let saved = core.cancel_at.take();
    core.cancel_at = Some(now_ms().saturating_add(ms(budget)));
    for (_, h) in leases {
        if let Some(Err(e)) = drive(engine.lease_release(h), core, inputs).await {
            tracing::warn!("lease release failed ({}); it goes stale", e.code());
        }
    }
    core.cancel_at = saved;
}

async fn request(
    r: Request,
    engine: &mut Engine,
    core: &mut Core,
    inputs: &mut Inputs,
) -> Response {
    let out = match r {
        Request::Status => drive(engine.status(), core, inputs).await.map(|r| {
            r.map(|report| {
                core.report = Some(report.clone());
                Response::Status(core.with_daemon(report))
            })
        }),
        Request::Sync { dry_run: _ } => run_sync(engine, core, inputs, true)
            .await
            .map(|r| r.map(Response::Sync)),
        // Answered in `Core::command`.
        Request::Pause { .. }
        | Request::Resume
        | Request::Subscribe
        | Request::Shutdown
        | Request::Play { .. }
        | Request::Answer(_) => return bad_request("answered elsewhere"),
        other => match drive(remote::run(engine, other), core, inputs).await {
            Some(Some(r)) => Some(r),
            Some(None) => return bad_request("not an engine request"),
            None => None,
        },
    };
    match out {
        Some(Ok(r)) => r,
        Some(Err(e)) => Response::Error(error_body(&e)),
        None => stopped(),
    }
}
