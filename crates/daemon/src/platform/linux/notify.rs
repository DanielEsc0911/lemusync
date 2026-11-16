//! Freedesktop notifications via `notify-rust` (zbus, tokio). Each shown notice gets a
//! thread that waits for `ActionInvoked` or the close, then forwards a click as
//! `(NoticeId, action id)`. The desktop's `__closed` is not forwarded.

use std::sync::atomic::{AtomicU64, Ordering};

use notify_rust::{Notification, Timeout, Urgency};
use tokio::runtime::Handle;
use tokio::sync::mpsc;

use crate::platform::{Notice, NoticeId, NoticeKind, Notifier};

struct LinuxNotifier {
    next: AtomicU64,
    tx: mpsc::Sender<(NoticeId, String)>,
    rt: Handle,
}

impl Notifier for LinuxNotifier {
    fn notify(&self, n: Notice) -> NoticeId {
        let id = NoticeId(self.next.fetch_add(1, Ordering::Relaxed) + 1);
        let tx = self.tx.clone();
        let rt = self.rt.clone();
        let spawned = std::thread::Builder::new()
            .name("lemusync-notice".into())
            .spawn(move || {
                // zbus spawns its tasks on this runtime; `wait_for_action` blocks this thread only.
                let _enter = rt.enter();
                let mut note = Notification::new();
                note.appname("LemuSync")
                    .summary(&n.title)
                    .body(&n.body)
                    .icon(crate::platform::APP_ICON_NAME)
                    .urgency(match n.kind {
                        NoticeKind::Attention => Urgency::Critical,
                        NoticeKind::Error => Urgency::Normal,
                        NoticeKind::Info => Urgency::Low,
                    })
                    .timeout(Timeout::Default);
                for (action, label) in &n.actions {
                    note.action(action, label);
                }
                let Ok(handle) = note.show() else { return };
                handle.wait_for_action(|action| {
                    if action != "__closed" {
                        let action = action.to_owned();
                        rt.spawn(async move { tx.send((id, action)).await });
                    }
                });
            });
        drop(spawned); // Best effort: no thread, no notification.
        id
    }
}

/// Start the notifier. Must be called inside a tokio runtime.
#[must_use]
pub fn start_notifier(tx: mpsc::Sender<(NoticeId, String)>) -> Box<dyn Notifier> {
    Box::new(LinuxNotifier {
        next: AtomicU64::new(0),
        tx,
        rt: Handle::current(),
    })
}
