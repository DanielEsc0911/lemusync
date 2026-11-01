//! The service's status model and event bus (architecture §Engine internals, `events`):
//! keeps the raw [`StatusFacts`], turns them into a [`Status`] by the spec's priority and fans
//! every [`Event`] out on a `broadcast` channel. A `StatusChanged` goes out only when the
//! status really changed.

use std::collections::BTreeMap;

use lemusync_api::{ErrorKind, Event, Status, StatusFacts, TransferPhase, TransferProgress};
use lemusync_core::schedule::SyncOutcome;
use lemusync_store::ErrorClass;
use tokio::sync::{broadcast, watch};

use crate::EngineError;

/// Events a slow subscriber may lag behind before it misses some (it then gets `Lagged`).
pub const EVENT_BUFFER: usize = 256;

pub(crate) struct StatusModel {
    facts: StatusFacts,
    /// Live transfers by `(player, emulator, game)`: phase and bytes total.
    transfers: BTreeMap<(String, String, String), (TransferPhase, u64)>,
    status: watch::Sender<Status>,
    events: broadcast::Sender<Event>,
}

impl StatusModel {
    pub fn new() -> Self {
        let facts = StatusFacts::default();
        let (status, _) = watch::channel(Status::from_facts(&facts));
        let (events, _) = broadcast::channel(EVENT_BUFFER);
        Self {
            facts,
            transfers: BTreeMap::new(),
            status,
            events,
        }
    }

    pub fn status(&self) -> Status {
        self.status.borrow().clone()
    }

    pub fn watch(&self) -> watch::Receiver<Status> {
        self.status.subscribe()
    }

    pub fn sender(&self) -> broadcast::Sender<Event> {
        self.events.clone()
    }

    /// Sends `e` to every subscriber (none listening is fine).
    pub fn emit(&self, e: Event) {
        let _ = self.events.send(e);
    }

    /// Changes the facts; a different status is published and sent as `StatusChanged`.
    pub fn update(&mut self, f: impl FnOnce(&mut StatusFacts)) {
        f(&mut self.facts);
        let next = Status::from_facts(&self.facts);
        if *self.status.borrow() != next {
            self.status.send_replace(next.clone());
            self.emit(Event::StatusChanged(next));
        }
    }

    /// One `TransferProgress` from the engine: forwarded, and counted into uploads,
    /// downloads, bytes and the syncing/scanning facts.
    pub fn progress(&mut self, p: &TransferProgress) {
        self.emit(Event::TransferProgress(p.clone()));
        let key = (p.player.clone(), p.emulator.clone(), p.game.clone());
        self.transfers.insert(key, (p.phase, p.bytes_total));
        let count = |phase| {
            u32::try_from(
                self.transfers
                    .values()
                    .filter(|(ph, _)| *ph == phase)
                    .count(),
            )
            .unwrap_or(u32::MAX)
        };
        let (uploads, downloads) = (
            count(TransferPhase::Uploading),
            count(TransferPhase::Downloading),
        );
        let bytes = self
            .transfers
            .values()
            .filter(|(ph, _)| *ph != TransferPhase::Checking)
            .map(|(_, b)| *b)
            .sum();
        self.update(|f| {
            f.uploads = uploads;
            f.downloads = downloads;
            f.bytes = bytes;
            f.syncing = uploads + downloads > 0;
        });
    }

    /// A sync (or check) starts: `scanning` until a transfer begins.
    pub fn sync_started(&mut self) {
        self.transfers.clear();
        self.update(|f| f.scanning = true);
    }

    /// The sync ended: no transfer is in flight any more.
    pub fn sync_ended(&mut self) {
        self.transfers.clear();
        self.update(|f| {
            f.scanning = false;
            f.syncing = false;
            f.uploads = 0;
            f.downloads = 0;
            f.bytes = 0;
        });
    }
}

/// The scheduler outcome of a failed sync.
#[must_use]
pub fn outcome(e: &EngineError) -> SyncOutcome {
    match e {
        EngineError::Store(s) => match s.class() {
            ErrorClass::Transient => SyncOutcome::Transient,
            ErrorClass::Auth => SyncOutcome::Auth,
            _ => SyncOutcome::Other,
        },
        // Waiting for the desktop's keystore is not an error: retry with backoff.
        EngineError::SecretsUnavailable(_) => SyncOutcome::Transient,
        _ => SyncOutcome::Other,
    }
}

/// The blocking `error` kind a failed sync shows, if any. Transient trouble is `offline`, not
/// an error; one-off failures (a busy folder, an injected fault) show nothing.
#[must_use]
pub fn error_kind(e: &EngineError) -> Option<ErrorKind> {
    match e {
        EngineError::Store(s) => match s.class() {
            ErrorClass::Auth => Some(ErrorKind::Auth),
            ErrorClass::Quota => Some(ErrorKind::Quota),
            _ => None,
        },
        EngineError::NotGranted(_) => Some(ErrorKind::Permission),
        EngineError::NotSetUp(_) => Some(ErrorKind::NotSetUp),
        EngineError::Config(_)
        | EngineError::NeedsConfirmation
        | EngineError::Unlock
        | EngineError::Secrets(_) => Some(ErrorKind::Config),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lemusync_api::StatusState;
    use lemusync_store::StoreError;

    fn progress(phase: TransferPhase, game: &str) -> TransferProgress {
        TransferProgress {
            player: "A".into(),
            emulator: "cemu".into(),
            game: game.into(),
            phase,
            files_done: 0,
            files_total: 1,
            bytes_done: 0,
            bytes_total: 10,
        }
    }

    fn states(rx: &mut broadcast::Receiver<Event>) -> Vec<StatusState> {
        let mut out = Vec::new();
        while let Ok(e) = rx.try_recv() {
            if let Event::StatusChanged(s) = e {
                out.push(s.state);
            }
        }
        out
    }

    #[test]
    fn a_sync_goes_scanning_syncing_synced_and_repeats_nothing() {
        let mut m = StatusModel::new();
        let mut rx = m.sender().subscribe();
        m.sync_started();
        m.progress(&progress(TransferPhase::Checking, "g"));
        m.progress(&progress(TransferPhase::Uploading, "g"));
        m.progress(&progress(TransferPhase::Uploading, "h"));
        assert_eq!((m.status().uploads, m.status().bytes), (2, 20));
        m.sync_ended();
        m.sync_ended();
        assert_eq!(
            states(&mut rx),
            [
                StatusState::Scanning,
                StatusState::Syncing,
                StatusState::Syncing, // the counts changed
                StatusState::Synced
            ]
        );
    }

    #[test]
    fn errors_map_to_outcomes_and_kinds() {
        let store = |c| EngineError::Store(StoreError::new(c, "x"));
        assert_eq!(
            outcome(&store(ErrorClass::Transient)),
            SyncOutcome::Transient
        );
        assert_eq!(outcome(&store(ErrorClass::Auth)), SyncOutcome::Auth);
        assert_eq!(outcome(&EngineError::Busy("x".into())), SyncOutcome::Other);
        // Waiting for the desktop keystore: retried with backoff, never a blocking error.
        let waiting = EngineError::SecretsUnavailable("no_owner".into());
        assert_eq!(outcome(&waiting), SyncOutcome::Transient);
        assert_eq!(error_kind(&waiting), None);
        assert_eq!(error_kind(&store(ErrorClass::Transient)), None);
        assert_eq!(
            error_kind(&store(ErrorClass::Quota)),
            Some(ErrorKind::Quota)
        );
        assert_eq!(
            error_kind(&EngineError::NotSetUp("repo")),
            Some(ErrorKind::NotSetUp)
        );
    }
}
