//! Repeating timer
use std::time::{Duration, Instant};

/// A simple timer repeating over an interval.
pub struct RepeatTimer {
    interval: Duration,
    start: Instant,
}

impl RepeatTimer {
    /// Returns a new timer with the specified `interval`.
    pub fn new(interval: Duration) -> Self {
        RepeatTimer {
            interval,
            start: Instant::now(),
        }
    }

    /// Returns `true` and resets if the timer surpasses the `interval`.
    pub fn tick(&mut self) -> bool {
        let now = Instant::now();
        if now.duration_since(self.start) >= self.interval {
            self.start = now;
            true
        } else {
            false
        }
    }
}
