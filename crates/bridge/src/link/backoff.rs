//! Reconnect delays: 1 s doubling to 30 s (B3: reconnect never spins).

use std::time::Duration;

/// The longest wait between two connection attempts.
const MAX_SECS: u64 = 30;

/// How long to wait after `attempt` failed attempts in a row (0 -> 1 s, 1 -> 2 s, ... 30 s).
pub(crate) fn delay(attempt: u32) -> Duration {
    let secs = 1_u64.checked_shl(attempt).unwrap_or(MAX_SECS).min(MAX_SECS);
    Duration::from_secs(secs)
}

#[cfg(test)]
mod tests {
    use super::delay;
    use std::time::Duration;

    #[test]
    fn doubles_from_one_second_and_caps_at_thirty() {
        let secs: Vec<u64> = (0..9).map(|a| delay(a).as_secs()).collect();
        assert_eq!(secs, [1, 2, 4, 8, 16, 30, 30, 30, 30]);
        assert_eq!(delay(63), Duration::from_secs(30));
        assert_eq!(delay(64), Duration::from_secs(30));
        assert_eq!(delay(u32::MAX), Duration::from_secs(30));
    }
}
