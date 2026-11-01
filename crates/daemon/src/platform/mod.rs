//! Desktop presence behind small traits: tray, notifications, power (sleep and idle).
//! Spec: docs/specs/ui/status-and-notifications.md §Desktop tray and menu bar,
//! docs/specs/reliability.md §Sleep and resume. Linux implementations live in `linux/`;
//! other targets get the no-ops in `other.rs` until Phase 2 Task 11.
//! The `Fake*` types record calls for tests.

use std::sync::{Arc, Mutex, PoisonError};

use tokio::sync::mpsc;

mod icons;
pub use icons::APP_ICON_NAME;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
pub use linux::{start_notifier, start_power, start_tray};

#[cfg(not(target_os = "linux"))]
mod other;
#[cfg(not(target_os = "linux"))]
pub use other::{start_notifier, start_power, start_tray};

/// The tray icon's state. `scanning` shows as `Syncing`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayIcon {
    Synced,
    Syncing,
    Paused,
    Offline,
    Attention,
    Error,
}

impl TrayIcon {
    /// The freedesktop symbolic icon name used on Linux.
    #[must_use]
    pub fn freedesktop_name(self) -> &'static str {
        match self {
            Self::Synced => "emblem-ok-symbolic",
            Self::Syncing => "emblem-synchronizing-symbolic",
            Self::Paused => "media-playback-pause-symbolic",
            Self::Offline => "network-offline-symbolic",
            Self::Attention => "dialog-warning-symbolic",
            Self::Error => "dialog-error-symbolic",
        }
    }
}

/// Everything the tray shows, already localised (built by `view::tray_view`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayView {
    pub icon: TrayIcon,
    pub tooltip: String,
    pub status_line: String,
    pub paused: bool,
    /// `(game ref, label)` for the "Recent games ▸ Play" submenu.
    pub recent: Vec<(String, String)>,
    /// Localised menu labels.
    pub menu: TrayMenuText,
}

/// Localised labels of the fixed tray menu items.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrayMenuText {
    pub sync_now: String,
    pub pause: String,
    pub pause_1h: String,
    pub pause_until_resume: String,
    pub resume: String,
    pub recent: String,
    pub quit: String,
}

/// What the user picked in the tray menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrayAction {
    SyncNow,
    Pause1h,
    PauseUntilResume,
    Resume,
    Play(String),
    Quit,
}

pub trait Tray: Send + Sync {
    /// Show this view. Never blocks; without a tray host it does nothing.
    fn show(&self, view: &TrayView);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoticeKind {
    Attention,
    Error,
    Info,
}

/// A notification. `actions` are `(id, label)`; a click comes back as `(NoticeId, id)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    pub kind: NoticeKind,
    pub title: String,
    pub body: String,
    pub actions: Vec<(String, String)>,
}

/// Our id for a shown notification (not the desktop's).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NoticeId(pub u64);

pub trait Notifier: Send + Sync {
    /// Show the notice. Never blocks; failures are dropped (notifications are best effort).
    fn notify(&self, n: Notice) -> NoticeId;
}

/// A sleep edge from the OS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SleepEdge {
    Suspending,
    Resumed,
}

pub trait Power: Send + Sync {
    /// Block idle sleep while the guard lives (at most 10 min). A suspend the user asks for
    /// still wins (reliability §Sleep and resume).
    fn block_idle(&self, reason: &str) -> IdleGuard;
    /// The sleep edges. The first call gets the stream; later calls get a closed one.
    fn sleep_events(&self) -> mpsc::Receiver<SleepEdge>;
    /// The daemon has checkpointed after `Suspending`: let the suspend go ahead.
    fn ack_suspend(&self);
}

/// Most idle blocks are released after this long, even if the guard still lives.
pub const IDLE_BLOCK_MAX: std::time::Duration = std::time::Duration::from_mins(10);

/// Releases its idle block when dropped.
#[must_use = "the idle block is released when the guard is dropped"]
pub struct IdleGuard {
    release: Option<Box<dyn FnOnce() + Send>>,
}

impl IdleGuard {
    pub fn new(release: impl FnOnce() + Send + 'static) -> Self {
        Self {
            release: Some(Box::new(release)),
        }
    }

    /// A guard that holds nothing.
    pub fn noop() -> Self {
        Self { release: None }
    }
}

impl Drop for IdleGuard {
    fn drop(&mut self) {
        if let Some(release) = self.release.take() {
            release();
        }
    }
}

impl std::fmt::Debug for IdleGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IdleGuard").finish_non_exhaustive()
    }
}

fn lock<T>(m: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    m.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Records every view shown.
#[derive(Debug, Default, Clone)]
pub struct FakeTray {
    pub shown: Arc<Mutex<Vec<TrayView>>>,
}

impl Tray for FakeTray {
    fn show(&self, view: &TrayView) {
        lock(&self.shown).push(view.clone());
    }
}

/// Records every notice and numbers them from 1.
#[derive(Debug, Default, Clone)]
pub struct FakeNotifier {
    pub sent: Arc<Mutex<Vec<Notice>>>,
}

impl Notifier for FakeNotifier {
    fn notify(&self, n: Notice) -> NoticeId {
        let mut sent = lock(&self.sent);
        sent.push(n);
        NoticeId(sent.len() as u64)
    }
}

/// What `FakePower` saw.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PowerCall {
    BlockIdle(String),
    ReleaseIdle(String),
    AckSuspend,
}

/// Records calls; `edges` pushes sleep edges to whoever took `sleep_events`.
#[derive(Debug, Clone)]
pub struct FakePower {
    pub calls: Arc<Mutex<Vec<PowerCall>>>,
    pub edges: mpsc::Sender<SleepEdge>,
    rx: Arc<Mutex<Option<mpsc::Receiver<SleepEdge>>>>,
}

impl Default for FakePower {
    fn default() -> Self {
        let (edges, rx) = mpsc::channel(8);
        Self {
            calls: Arc::default(),
            edges,
            rx: Arc::new(Mutex::new(Some(rx))),
        }
    }
}

impl Power for FakePower {
    fn block_idle(&self, reason: &str) -> IdleGuard {
        lock(&self.calls).push(PowerCall::BlockIdle(reason.to_owned()));
        let calls = Arc::clone(&self.calls);
        let reason = reason.to_owned();
        IdleGuard::new(move || lock(&calls).push(PowerCall::ReleaseIdle(reason)))
    }

    fn sleep_events(&self) -> mpsc::Receiver<SleepEdge> {
        take_or_closed(&self.rx)
    }

    fn ack_suspend(&self) {
        lock(&self.calls).push(PowerCall::AckSuspend);
    }
}

/// The receiver on the first call, a closed one after.
fn take_or_closed<T>(slot: &Mutex<Option<mpsc::Receiver<T>>>) -> mpsc::Receiver<T> {
    lock(slot).take().unwrap_or_else(|| mpsc::channel(1).1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idle_guard_drop_releases() {
        let power = FakePower::default();
        let guard = power.block_idle("uploading");
        assert_eq!(
            *lock(&power.calls),
            vec![PowerCall::BlockIdle("uploading".into())]
        );
        drop(guard);
        assert_eq!(
            *lock(&power.calls),
            vec![
                PowerCall::BlockIdle("uploading".into()),
                PowerCall::ReleaseIdle("uploading".into())
            ]
        );
        power.ack_suspend();
        assert_eq!(lock(&power.calls).last(), Some(&PowerCall::AckSuspend));
    }

    #[tokio::test]
    async fn sleep_events_once_then_closed() {
        let power = FakePower::default();
        let mut rx = power.sleep_events();
        power.edges.send(SleepEdge::Suspending).await.ok();
        assert_eq!(rx.recv().await, Some(SleepEdge::Suspending));
        assert_eq!(power.sleep_events().recv().await, None);
    }

    #[test]
    fn fakes_record() {
        let tray = FakeTray::default();
        let view = TrayView {
            icon: TrayIcon::Synced,
            tooltip: "t".into(),
            status_line: "s".into(),
            paused: false,
            recent: vec![],
            menu: TrayMenuText::default(),
        };
        tray.show(&view);
        assert_eq!(*lock(&tray.shown), vec![view]);
        let notifier = FakeNotifier::default();
        let n = Notice {
            kind: NoticeKind::Info,
            title: "a".into(),
            body: "b".into(),
            actions: vec![],
        };
        assert_eq!(notifier.notify(n.clone()), NoticeId(1));
        assert_eq!(notifier.notify(n), NoticeId(2));
    }

    #[test]
    fn icons_are_symbolic() {
        for icon in [
            TrayIcon::Synced,
            TrayIcon::Syncing,
            TrayIcon::Paused,
            TrayIcon::Offline,
            TrayIcon::Attention,
            TrayIcon::Error,
        ] {
            assert!(icon.freedesktop_name().ends_with("-symbolic"));
        }
    }
}
