//! When the daemon syncs: the quiet period per stream and the sync scheduler with backoff.
//! No clock and no randomness: callers pass `now_ms` and `jitter`.
//! Spec: docs/specs/reliability.md (Always on: Scheduler; Network).

use std::collections::BTreeMap;

use crate::StreamId;

/// Streams changed recently; each becomes due once it has been quiet for `quiet_ms`.
#[derive(Clone, Debug)]
pub struct Quiet {
    quiet_ms: u64,
    last_touch: BTreeMap<StreamId, u64>,
}

impl Quiet {
    #[must_use]
    pub fn new(quiet_ms: u64) -> Self {
        Self {
            quiet_ms,
            last_touch: BTreeMap::new(),
        }
    }

    /// A file of `stream` changed at `now_ms`: its quiet period starts again.
    pub fn touch(&mut self, stream: StreamId, now_ms: u64) {
        self.last_touch.insert(stream, now_ms);
    }

    /// Streams quiet for `quiet_ms` since their last touch.
    #[must_use]
    pub fn due(&self, now_ms: u64) -> Vec<StreamId> {
        self.last_touch
            .iter()
            .filter(|(_, t)| t.saturating_add(self.quiet_ms) <= now_ms)
            .map(|(s, _)| *s)
            .collect()
    }

    /// `due`, and forgets those streams.
    pub fn take_due(&mut self, now_ms: u64) -> Vec<StreamId> {
        let due = self.due(now_ms);
        for s in &due {
            self.last_touch.remove(s);
        }
        due
    }

    /// The earliest time a tracked stream becomes due.
    #[must_use]
    pub fn next_deadline(&self) -> Option<u64> {
        self.last_touch
            .values()
            .map(|t| t.saturating_add(self.quiet_ms))
            .min()
    }
}

/// Why the daemon wants a sync.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Trigger {
    /// A stream went quiet after a change.
    Change(StreamId),
    /// The rescan timer (watchers can miss events).
    Timer,
    /// The machine woke up.
    Resume,
    /// The network came back.
    NetworkBack,
    /// An emulator (by id) exited.
    EmulatorExit(String),
    /// The user asked (CLI, tray). Overrides a pause and an `Auth` stop.
    Manual,
    /// The credentials changed or were just accepted (`backend add`, a passing `doctor`
    /// backend check). Ends an `Auth` stop; a pause still holds.
    Credentials,
}

/// How a whole sync ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SyncOutcome {
    Ok,
    /// Network or server trouble that may go away: retry with backoff.
    Transient,
    /// Credentials rejected: stop until `Manual`, `Credentials` or `resume` (the daemon
    /// notifies).
    Auth,
    /// Any other failure: no retry until the next trigger or rescan.
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    SyncNow,
    /// Ask again at this time (always later than the `now_ms` given).
    WaitUntil(u64),
    /// Nothing to wait for: paused until `resume`, stopped on `Auth`, or a sync is running.
    Idle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScheduleConfig {
    pub rescan_ms: u64,
    pub backoff_min_ms: u64,
    pub backoff_max_ms: u64,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            rescan_ms: 15 * 60_000,
            backoff_min_ms: 1_000,
            backoff_max_ms: 5 * 60_000,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pause {
    No,
    Until(u64),
    UntilResume,
}

/// One sync at a time. The caller loops: `next_action` → on `SyncNow` call `on_sync_started`,
/// run the sync, then `on_sync_done`; on `WaitUntil(t)` sleep until `t` or the next trigger.
/// A new scheduler wants a sync at once (daemon start).
#[derive(Clone, Debug)]
pub struct Scheduler {
    cfg: ScheduleConfig,
    pending: bool,
    manual: bool,
    /// `Some(again)` while a sync runs; `again` is set by a trigger that arrives meanwhile.
    syncing: Option<bool>,
    stopped: bool,
    paused: Pause,
    attempt: u32,
    retry_at: Option<u64>,
    last_done: Option<u64>,
}

impl Scheduler {
    #[must_use]
    pub fn new(cfg: ScheduleConfig) -> Self {
        Self {
            cfg,
            pending: true,
            manual: false,
            syncing: None,
            stopped: false,
            paused: Pause::No,
            attempt: 0,
            retry_at: None,
            last_done: None,
        }
    }

    #[allow(clippy::needless_pass_by_value)] // Triggers are owned events from the daemon loop.
    pub fn on_trigger(&mut self, t: Trigger, _now_ms: u64) {
        match t {
            Trigger::Resume | Trigger::NetworkBack => self.reset_backoff(),
            Trigger::Manual => {
                self.manual = true;
                self.stopped = false;
                self.reset_backoff();
            }
            Trigger::Credentials => {
                self.stopped = false;
                self.reset_backoff();
            }
            Trigger::Change(_) | Trigger::Timer | Trigger::EmulatorExit(_) => {}
        }
        if let Some(again) = &mut self.syncing {
            *again = true;
        } else {
            self.pending = true;
        }
    }

    /// The caller is starting the sync `next_action` asked for.
    pub fn on_sync_started(&mut self) {
        self.syncing = Some(false);
        self.pending = false;
        self.manual = false;
    }

    /// `jitter` is a random number in `[0, 1)`.
    pub fn on_sync_done(&mut self, outcome: SyncOutcome, now_ms: u64, jitter: f64) {
        let again = self.syncing.take().unwrap_or(false);
        self.last_done = Some(now_ms);
        match outcome {
            SyncOutcome::Ok | SyncOutcome::Other => {
                self.reset_backoff();
                self.pending = again;
            }
            SyncOutcome::Transient => {
                let wait = self.backoff_ms(jitter);
                self.attempt = self.attempt.saturating_add(1);
                self.retry_at = Some(now_ms.saturating_add(wait));
                self.pending = true;
            }
            SyncOutcome::Auth => {
                self.reset_backoff();
                self.stopped = true;
                self.pending = false;
            }
        }
    }

    /// Pause until `until_ms`, or until `resume` when `None`. Only `Manual` syncs meanwhile.
    pub fn pause(&mut self, until_ms: Option<u64>) {
        self.paused = until_ms.map_or(Pause::UntilResume, Pause::Until);
    }

    /// Ends a pause and an `Auth` stop, and syncs to catch up.
    pub fn resume(&mut self) {
        self.paused = Pause::No;
        self.stopped = false;
        self.pending = true;
    }

    #[must_use]
    pub fn next_action(&self, now_ms: u64) -> Action {
        if self.syncing.is_some() {
            return Action::Idle;
        }
        if self.manual {
            return Action::SyncNow;
        }
        if self.stopped {
            return Action::Idle;
        }
        match self.paused {
            Pause::UntilResume => return Action::Idle,
            Pause::Until(t) if now_ms < t => return Action::WaitUntil(t),
            _ => {}
        }
        let due = if self.pending {
            self.retry_at.unwrap_or(0)
        } else {
            self.last_done
                .map_or(0, |d| d.saturating_add(self.cfg.rescan_ms))
        };
        if due > now_ms {
            Action::WaitUntil(due)
        } else {
            Action::SyncNow
        }
    }

    fn reset_backoff(&mut self) {
        self.attempt = 0;
        self.retry_at = None;
    }

    /// Full jitter with a floor: `max(min, cap * jitter)`, where the cap doubles from `min` to `max`.
    fn backoff_ms(&self, jitter: f64) -> u64 {
        let cap = self
            .cfg
            .backoff_min_ms
            .saturating_mul(1u64.checked_shl(self.attempt.min(32)).unwrap_or(u64::MAX))
            .min(self.cfg.backoff_max_ms);
        let j = if jitter.is_finite() {
            jitter.clamp(0.0, 1.0)
        } else {
            0.0
        };
        // Exact for any cap below 2^53 ms; the product is in [0, cap].
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let jittered = (cap as f64 * j) as u64;
        jittered
            .max(self.cfg.backoff_min_ms)
            .min(cap.max(self.cfg.backoff_min_ms))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::Hash32;

    fn s(n: u8) -> StreamId {
        StreamId(Hash32([n; 32]))
    }

    fn started(sch: &mut Scheduler, now: u64) {
        assert_eq!(sch.next_action(now), Action::SyncNow);
        sch.on_sync_started();
    }

    /// A scheduler past its startup sync.
    fn settled(now: u64) -> Scheduler {
        let mut sch = Scheduler::new(ScheduleConfig::default());
        started(&mut sch, now);
        sch.on_sync_done(SyncOutcome::Ok, now, 0.5);
        sch
    }

    #[test]
    fn quiet_waits_for_the_last_touch_of_each_stream() {
        let mut q = Quiet::new(10_000);
        assert_eq!(q.next_deadline(), None);
        q.touch(s(1), 0);
        q.touch(s(2), 3_000);
        q.touch(s(1), 5_000);
        assert_eq!(q.next_deadline(), Some(13_000));
        assert_eq!(q.due(12_999), Vec::<StreamId>::new());
        assert_eq!(q.due(13_000), vec![s(2)]);
        assert_eq!(q.take_due(14_000), vec![s(2)]);
        assert_eq!(q.due(14_000), Vec::<StreamId>::new());
        assert_eq!(q.next_deadline(), Some(15_000));
        assert_eq!(q.take_due(15_000), vec![s(1)]);
        assert_eq!(q.next_deadline(), None);
    }

    #[test]
    fn a_new_scheduler_syncs_at_once_then_rescans_on_the_timer() {
        let sch = settled(1_000);
        assert_eq!(
            sch.next_action(1_000),
            Action::WaitUntil(1_000 + 15 * 60_000)
        );
        assert_eq!(sch.next_action(1_000 + 15 * 60_000), Action::SyncNow);
    }

    #[test]
    fn transient_failures_back_off_doubling_to_five_minutes() {
        for (jitter, expect) in [
            (0.0, vec![1_000u64; 12]),
            (
                0.999,
                vec![
                    999, 1_998, 3_996, 7_992, 15_984, 31_968, 63_936, 127_872, 255_744, 299_700,
                    299_700, 299_700,
                ],
            ),
        ] {
            let mut sch = Scheduler::new(ScheduleConfig::default());
            let mut now = 0;
            for want in expect {
                started(&mut sch, now);
                sch.on_sync_done(SyncOutcome::Transient, now, jitter);
                let Action::WaitUntil(t) = sch.next_action(now) else {
                    panic!("expected a wait");
                };
                // The 1 s floor wins over 999 ms.
                assert_eq!(t - now, want.max(1_000), "jitter {jitter}");
                now = t;
            }
        }
    }

    #[test]
    fn ok_resets_the_backoff() {
        let mut sch = Scheduler::new(ScheduleConfig::default());
        let mut now = 0;
        for _ in 0..5 {
            started(&mut sch, now);
            sch.on_sync_done(SyncOutcome::Transient, now, 0.999);
            if let Action::WaitUntil(t) = sch.next_action(now) {
                now = t;
            }
        }
        started(&mut sch, now);
        sch.on_sync_done(SyncOutcome::Ok, now, 0.999);
        sch.on_trigger(Trigger::Change(s(1)), now);
        started(&mut sch, now);
        sch.on_sync_done(SyncOutcome::Transient, now, 0.999);
        assert_eq!(sch.next_action(now), Action::WaitUntil(now + 1_000));
    }

    #[test]
    fn resume_and_network_back_sync_now_and_reset_the_backoff() {
        for t in [Trigger::Resume, Trigger::NetworkBack] {
            let mut sch = Scheduler::new(ScheduleConfig::default());
            for _ in 0..4 {
                started(&mut sch, 0);
                sch.on_sync_done(SyncOutcome::Transient, 0, 0.999);
                sch.on_trigger(t.clone(), 0);
            }
            assert_eq!(sch.next_action(0), Action::SyncNow);
            sch.on_sync_started();
            sch.on_sync_done(SyncOutcome::Transient, 0, 0.999);
            assert_eq!(sch.next_action(0), Action::WaitUntil(1_000));
        }
    }

    #[test]
    fn auth_stops_until_manual() {
        let mut sch = Scheduler::new(ScheduleConfig::default());
        started(&mut sch, 0);
        sch.on_sync_done(SyncOutcome::Auth, 0, 0.5);
        for t in [Trigger::Change(s(1)), Trigger::Timer, Trigger::Resume] {
            sch.on_trigger(t, 10);
            assert_eq!(sch.next_action(10_000_000), Action::Idle);
        }
        sch.on_trigger(Trigger::Manual, 20);
        assert_eq!(sch.next_action(20), Action::SyncNow);
    }

    #[test]
    fn new_credentials_end_the_auth_stop_but_not_a_pause() {
        let mut sch = Scheduler::new(ScheduleConfig::default());
        started(&mut sch, 0);
        sch.on_sync_done(SyncOutcome::Auth, 0, 0.5);
        sch.on_trigger(Trigger::Credentials, 10);
        assert_eq!(sch.next_action(10), Action::SyncNow);

        let mut sch = Scheduler::new(ScheduleConfig::default());
        started(&mut sch, 0);
        sch.on_sync_done(SyncOutcome::Auth, 0, 0.5);
        sch.pause(None);
        sch.on_trigger(Trigger::Credentials, 10);
        assert_eq!(sch.next_action(10), Action::Idle);
    }

    #[test]
    fn auth_stops_until_resume() {
        let mut sch = Scheduler::new(ScheduleConfig::default());
        started(&mut sch, 0);
        sch.on_sync_done(SyncOutcome::Auth, 0, 0.5);
        sch.resume();
        assert_eq!(sch.next_action(1), Action::SyncNow);
    }

    #[test]
    fn a_pause_until_a_time_ends_by_itself_and_only_manual_syncs_meanwhile() {
        let mut sch = settled(0);
        sch.pause(Some(60_000));
        sch.on_trigger(Trigger::Change(s(1)), 10);
        assert_eq!(sch.next_action(10), Action::WaitUntil(60_000));
        assert_eq!(sch.next_action(60_000), Action::SyncNow);

        let mut sch = settled(0);
        sch.pause(None);
        sch.on_trigger(Trigger::Change(s(1)), 10);
        assert_eq!(sch.next_action(100_000_000), Action::Idle);
        sch.on_trigger(Trigger::Manual, 20);
        assert_eq!(sch.next_action(20), Action::SyncNow);
        sch.on_sync_started();
        sch.on_sync_done(SyncOutcome::Ok, 30, 0.5);
        assert_eq!(sch.next_action(30), Action::Idle);
        sch.resume();
        assert_eq!(sch.next_action(40), Action::SyncNow);
    }

    #[test]
    fn triggers_during_a_sync_give_exactly_one_follow_up() {
        let mut sch = settled(0);
        sch.on_trigger(Trigger::Change(s(1)), 1);
        started(&mut sch, 1);
        sch.on_trigger(Trigger::Change(s(2)), 2);
        sch.on_trigger(Trigger::EmulatorExit("cemu".into()), 3);
        sch.on_trigger(Trigger::Timer, 4);
        assert_eq!(sch.next_action(5), Action::Idle);
        sch.on_sync_done(SyncOutcome::Ok, 5, 0.5);
        started(&mut sch, 5);
        sch.on_sync_done(SyncOutcome::Ok, 6, 0.5);
        assert_eq!(sch.next_action(6), Action::WaitUntil(6 + 15 * 60_000));
    }

    #[test]
    fn next_action_never_waits_for_the_past() {
        let mut seed: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut next = || {
            seed ^= seed << 13;
            seed ^= seed >> 7;
            seed ^= seed << 17;
            seed
        };
        let mut sch = Scheduler::new(ScheduleConfig::default());
        let mut now = 0u64;
        for _ in 0..20_000 {
            now += next() % 400_000;
            #[allow(clippy::cast_precision_loss)]
            let jitter = (next() % 1_000) as f64 / 1_000.0;
            match next() % 9 {
                0 => sch.on_trigger(Trigger::Change(s(1)), now),
                1 => sch.on_trigger(Trigger::Timer, now),
                2 => sch.on_trigger(Trigger::Resume, now),
                3 => sch.on_trigger(Trigger::Manual, now),
                4 => sch.pause(Some(now + next() % 1_000_000)),
                5 => sch.resume(),
                _ => {}
            }
            match sch.next_action(now) {
                Action::WaitUntil(t) => assert!(t > now),
                Action::SyncNow => {
                    sch.on_sync_started();
                    let o = match next() % 4 {
                        0 => SyncOutcome::Ok,
                        1 => SyncOutcome::Transient,
                        2 => SyncOutcome::Auth,
                        _ => SyncOutcome::Other,
                    };
                    sch.on_sync_done(o, now, jitter);
                }
                Action::Idle => {}
            }
        }
    }
}
