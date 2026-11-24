//! Watch-mode process detection: polls which emulators run and reports each start and stop
//! (sync-model §Watch mode). The poll is the daemon's only periodic wake (reliability §Budgets).

use std::collections::BTreeSet;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;

use lemusync_api::RemoteNewer;
use lemusync_core::heads::{Graph, heads};

use crate::{Engine, EngineError, ProcessProbe};

/// Default poll interval (V-DAEMON-2 measures its idle cost).
pub const DEFAULT_POLL: Duration = Duration::from_secs(3);

/// Calls `ProcessProbe::running_set` every `poll` and sends `(emulator, running)` only when it
/// changes. Everything starts as "not running", so emulators already open are reported on the
/// first poll. Stops when dropped or when the receiver is gone.
pub struct ProcessPoller {
    task: JoinHandle<()>,
}

impl ProcessPoller {
    /// Must be called inside a tokio runtime.
    #[must_use]
    pub fn start(
        probe: Arc<dyn ProcessProbe>,
        names_by_emulator: Vec<(String, Vec<String>)>,
        poll: Duration,
        tx: Sender<(String, bool)>,
    ) -> Self {
        let names = Arc::new(names_by_emulator);
        let task = tokio::spawn(async move {
            let mut was: BTreeSet<String> = BTreeSet::new();
            loop {
                let (p, n) = (Arc::clone(&probe), Arc::clone(&names));
                // The process walk blocks (a `/proc` read on Linux).
                let Ok(now) = tokio::task::spawn_blocking(move || p.running_set(&n)).await else {
                    return;
                };
                for id in now.symmetric_difference(&was) {
                    if tx.send((id.clone(), now.contains(id))).await.is_err() {
                        return;
                    }
                }
                was = now;
                tokio::time::sleep(poll).await;
            }
        });
        Self { task }
    }
}

impl Drop for ProcessPoller {
    fn drop(&mut self) {
        self.task.abort();
    }
}

impl Engine {
    /// Watch mode at emulator start (`CheckRemote`): the streams of `emulator` whose newest
    /// head another device wrote and this device hasn't restored yet. Reads the version list
    /// only and restores nothing (I3): the update waits for the emulator to close.
    ///
    /// # Errors
    /// As `watch_plan`; storage errors while listing versions.
    pub async fn remote_newer(&mut self, emulator: &str) -> Result<Vec<RemoteNewer>, EngineError> {
        let targets = self.watch_plan().await?;
        let names = self.game_names();
        let me = self.device_id();
        let mut warnings = Vec::new();
        let mut out = Vec::new();
        for t in targets.into_iter().filter(|t| t.emulator == emulator) {
            let versions = self.fetch_graph(&t.stream, &mut warnings).await?;
            let graph: Graph = versions
                .iter()
                .map(|(id, v)| (*id, v.parents.clone()))
                .collect();
            let base = self.state.stream(&t.stream)?.and_then(|r| r.base);
            let Some(newest) = heads(&graph)
                .iter()
                .filter_map(|h| versions.get(h))
                .max_by_key(|v| (v.created_at, v.id))
            else {
                continue;
            };
            if Some(newest.id) == base || newest.device == me {
                continue;
            }
            let device = self
                .repo_ref()?
                .device_name(&newest.device)
                .await
                .unwrap_or_else(|| newest.device.to_string());
            out.push(RemoteNewer {
                name: crate::games::name_in(&names, &t.emulator, &t.game),
                player: t.player_name,
                emulator: t.emulator,
                game: t.game,
                device,
            });
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FakeProbe;
    use tokio::time::timeout;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn the_poller_reports_start_and_stop_once_each() {
        let probe = Arc::new(FakeProbe::default());
        let (tx, mut rx) = tokio::sync::mpsc::channel(8);
        let _p = ProcessPoller::start(
            probe.clone(),
            vec![("cemu".into(), vec!["Cemu".into()])],
            Duration::from_millis(20),
            tx,
        );
        let wait = Duration::from_secs(10);
        probe.set(true);
        assert_eq!(
            timeout(wait, rx.recv()).await.ok().flatten(),
            Some(("cemu".to_owned(), true))
        );
        probe.set(false);
        assert_eq!(
            timeout(wait, rx.recv()).await.ok().flatten(),
            Some(("cemu".to_owned(), false))
        );
        // Many more polls, no change: nothing more.
        assert!(
            timeout(Duration::from_millis(300), rx.recv())
                .await
                .is_err()
        );
    }
}
