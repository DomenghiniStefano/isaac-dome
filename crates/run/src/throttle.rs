//! When a burst of log changes is acted on. Pure: it is handed the time,
//! it never reads a clock, so the rule is tested without waiting for one.

use std::time::{Duration, Instant};

/// Acts at once on a change, then at most once per `period` — and **never drops one**: a change
/// that arrives inside the period is kept, and acted on when the period ends.
///
/// The watcher it replaces acted on the first change and threw away every one in the next two
/// seconds. The last lines of a burst — a death, an ending, the game closing — then stayed
/// unread until something else touched the file, and were lost for good when that touch was
/// a relaunch.
#[derive(Debug, Clone)]
pub struct Throttle {
    period: Duration,
    last: Option<Instant>,
    pending: bool,
}

impl Throttle {
    pub fn new(period: Duration) -> Throttle {
        Throttle {
            period,
            last: None,
            pending: false,
        }
    }

    /// A change arrived at `now`. `true` when it is to be acted on now; otherwise it is kept.
    pub fn event(&mut self, now: Instant) -> bool {
        if self.quiet_since_last(now) {
            self.act(now);
            true
        } else {
            self.pending = true;
            false
        }
    }

    /// Time passed at `now` with no new change. `true` when the kept change is due now.
    pub fn tick(&mut self, now: Instant) -> bool {
        if self.pending && self.quiet_since_last(now) {
            self.act(now);
            true
        } else {
            false
        }
    }

    /// How long until the kept change is due; `None` when nothing is kept.
    pub fn wait(&self, now: Instant) -> Option<Duration> {
        let last = self.last.filter(|_| self.pending)?;
        Some((last + self.period).saturating_duration_since(now))
    }

    fn quiet_since_last(&self, now: Instant) -> bool {
        self.last
            .is_none_or(|t| now.saturating_duration_since(t) >= self.period)
    }

    fn act(&mut self, now: Instant) {
        self.last = Some(now);
        self.pending = false;
    }
}
