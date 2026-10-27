//! Deterministic gameplay timers driven explicitly by simulation deltas.
//!
//! Timers never read wall-clock time and own no host resources, so they may be
//! stored directly in script structs. Hot reload resets unexposed script state.
//!
//! ```
//! use kvas_engine::timer::{Timer, TimerMode};
//! use std::time::Duration;
//! let mut timer = Timer::new(Duration::from_millis(100), TimerMode::Repeating);
//! timer.tick(Duration::from_millis(250));
//! assert_eq!(timer.times_finished(), 2);
//! assert_eq!(timer.elapsed(), Duration::from_millis(50));
//! assert!((timer.fraction() - 0.5).abs() < 1e-6);
//! ```

use std::time::Duration;

/// What happens when a timer reaches its duration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerMode {
    /// Completes once and remains finished until reset.
    Once,
    /// Wraps elapsed time and counts every crossed interval, including catch-up.
    Repeating,
}

/// Explicitly ticked timer with pause, reset and exact duration accounting.
/// A zero-duration timer completes once per non-paused tick, including a zero tick.
#[derive(Clone, Debug)]
pub struct Timer {
    duration: Duration,
    elapsed: Duration,
    mode: TimerMode,
    paused: bool,
    finished: bool,
    times_finished: u64,
}

impl Timer {
    /// Creates an unpaused timer at zero elapsed time.
    pub fn new(duration: Duration, mode: TimerMode) -> Self {
        Self { duration, elapsed: Duration::ZERO, mode, paused: false, finished: false, times_finished: 0 }
    }

    /// Creates a one-shot timer from finite non-negative seconds.
    /// Invalid or unrepresentable durations return `None`.
    pub fn once(seconds: f32) -> Option<Self> {
        Duration::try_from_secs_f32(seconds).ok().map(|d| Self::new(d, TimerMode::Once))
    }

    /// Creates a repeating timer from finite non-negative seconds.
    pub fn repeating(seconds: f32) -> Option<Self> {
        Duration::try_from_secs_f32(seconds).ok().map(|d| Self::new(d, TimerMode::Repeating))
    }

    /// Advances time and replaces this tick's completion count.
    /// Paused ticks clear completion pulses while preserving elapsed time and
    /// the sticky finished state of one-shot timers.
    pub fn tick(&mut self, delta: Duration) -> &mut Self {
        self.times_finished = 0;
        if self.mode == TimerMode::Repeating { self.finished = false; }
        if self.paused || (self.mode == TimerMode::Once && self.finished) {
            return self;
        }
        if self.duration.is_zero() {
            self.times_finished = 1;
            self.finished = true;
            return self;
        }
        let total = self.elapsed.as_nanos() + delta.as_nanos();
        let duration = self.duration.as_nanos();
        if total >= duration {
            self.finished = true;
            match self.mode {
                TimerMode::Once => {
                    self.elapsed = self.duration;
                    self.times_finished = 1;
                }
                TimerMode::Repeating => {
                    self.times_finished = (total / duration).min(u64::MAX as u128) as u64;
                    self.elapsed = from_nanos(total % duration);
                }
            }
        } else {
            self.elapsed = from_nanos(total);
            self.finished = false;
        }
        self
    }

    /// Ticks using a script/frame delta in seconds. Invalid deltas are ignored
    /// and clear completion edges, so a bad frame cannot poison timer state.
    pub fn tick_secs(&mut self, seconds: f32) -> &mut Self {
        match Duration::try_from_secs_f32(seconds) {
            Ok(dt) => self.tick(dt),
            Err(_) => {
                self.times_finished = 0;
                if self.mode == TimerMode::Repeating { self.finished = false; }
                self
            }
        }
    }

    /// Configured interval.
    pub fn duration(&self) -> Duration {
        self.duration
    }
    /// Elapsed part of the interval (clamped for one-shot, wrapped for repeating).
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }
    /// Time remaining until the next completion. A finished one-shot returns zero.
    pub fn remaining(&self) -> Duration {
        self.duration.saturating_sub(self.elapsed)
    }
    /// Progress in `[0, 1]`; zero-duration timers return one after their first tick.
    pub fn fraction(&self) -> f32 {
        if self.duration.is_zero() {
            return if self.finished { 1.0 } else { 0.0 };
        }
        (self.elapsed.as_secs_f64() / self.duration.as_secs_f64()).clamp(0.0, 1.0) as f32
    }
    /// Whether a one-shot is complete, or a repeating timer completed this tick.
    pub fn finished(&self) -> bool {
        self.finished
    }
    /// Whether the most recent tick crossed at least one interval.
    pub fn just_finished(&self) -> bool {
        self.times_finished != 0
    }
    /// Number of crossed intervals in the last tick, saturating at `u64::MAX`.
    pub fn times_finished(&self) -> u64 {
        self.times_finished
    }
    /// Current repetition policy.
    pub fn mode(&self) -> TimerMode {
        self.mode
    }
    /// Whether ticks are suspended.
    pub fn is_paused(&self) -> bool {
        self.paused
    }
    /// Suspends or resumes the timer without resetting it.
    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }
    /// Clears elapsed/completion state while preserving duration, mode and pause.
    pub fn reset(&mut self) {
        self.elapsed = Duration::ZERO;
        self.finished = false;
        self.times_finished = 0;
    }
    /// Replaces the interval and resets progress, preserving mode and pause.
    pub fn set_duration(&mut self, duration: Duration) {
        self.duration = duration;
        self.reset();
    }
    /// Replaces the repetition policy and resets progress.
    pub fn set_mode(&mut self, mode: TimerMode) {
        self.mode = mode;
        self.reset();
    }
}

fn from_nanos(value: u128) -> Duration {
    Duration::new((value / 1_000_000_000) as u64, (value % 1_000_000_000) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn once_edges_and_pause() {
        let mut t = Timer::once(1.0).unwrap();
        t.tick_secs(0.5);
        assert_eq!(t.fraction(), 0.5);
        t.set_paused(true);
        t.tick_secs(2.0);
        assert!(!t.finished());
        t.set_paused(false);
        t.tick_secs(2.0);
        assert!(t.just_finished());
        assert_eq!(t.remaining(), Duration::ZERO);
        t.tick_secs(1.0);
        assert!(t.finished());
        assert!(!t.just_finished());
        t.reset();
        assert!(!t.finished());
        assert_eq!(t.elapsed(), Duration::ZERO);
    }
    #[test]
    fn repeating_counts_catchup_exactly_and_handles_invalid_deltas() {
        let mut t = Timer::new(Duration::from_millis(100), TimerMode::Repeating);
        t.tick(Duration::from_millis(250));
        assert_eq!(t.times_finished(), 2);
        assert_eq!(t.elapsed(), Duration::from_millis(50));
        t.tick_secs(f32::NAN);
        assert!(!t.just_finished());
        assert_eq!(t.elapsed(), Duration::from_millis(50));
        t.tick(Duration::from_millis(50));
        assert_eq!(t.times_finished(), 1);
        assert_eq!(t.elapsed(), Duration::ZERO);
        assert!(Timer::once(-1.0).is_none());
        assert!(Timer::repeating(f32::INFINITY).is_none());
    }
    #[test]
    fn zero_and_max_durations_are_bounded() {
        let mut t = Timer::new(Duration::ZERO, TimerMode::Repeating);
        t.tick(Duration::MAX);
        assert_eq!(t.times_finished(), 1);
        t.set_duration(Duration::from_nanos(1));
        t.tick(Duration::MAX);
        assert_eq!(t.times_finished(), u64::MAX);
        t.set_duration(Duration::MAX);
        t.tick(Duration::MAX);
        assert_eq!(t.times_finished(), 1);
    }
}
