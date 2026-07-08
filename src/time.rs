//! Time management: delta time, FPS tracking, timers, and time utilities.
//!
//! game-gem provides a first-class [`Time`] resource that's automatically
//! updated each frame, unlike macroquad which exposes raw `get_time()`.

use std::time::{Duration, Instant};

/// Time resource, updated automatically every frame.
///
/// Access via `ctx.time` inside your [`GameState`] implementation.
#[derive(Debug)]
pub struct Time {
    /// Time since the engine started (seconds).
    elapsed: f64,
    /// Wall-clock start time.
    start: Instant,
    /// Time of the previous frame.
    last_frame: Instant,
    /// Duration of the last frame in seconds.
    delta: f64,
    /// Exponential moving average of delta (for stable FPS display).
    smoothed_delta: f64,
    /// Target fixed timestep (for fixed-update patterns).
    fixed_timestep: f64,
    /// Accumulator for fixed updates.
    fixed_accumulator: f64,
    /// Whether to use a fixed timestep.
    use_fixed_timestep: bool,
    /// Frame counter since start.
    frame_count: u64,
    /// Scale applied to delta time (for slow-motion, pause, etc.).
    time_scale: f64,
    /// FPS tracker.
    fps_tracker: FpsTracker,
}

#[derive(Debug)]
struct FpsTracker {
    samples: [f64; 60],
    index: usize,
    filled: bool,
}

impl Default for FpsTracker {
    fn default() -> Self {
        Self {
            samples: [0.0; 60],
            index: 0,
            filled: false,
        }
    }
}

impl FpsTracker {
    fn record(&mut self, delta: f64) {
        self.samples[self.index] = delta;
        self.index = (self.index + 1) % self.samples.len();
        if self.index == 0 {
            self.filled = true;
        }
    }

    fn average_fps(&self) -> f64 {
        let count = if self.filled {
            self.samples.len()
        } else {
            self.index
        };
        if count == 0 {
            return 0.0;
        }
        let sum: f64 = self.samples[..count].iter().sum();
        if sum < 1e-9 {
            return f64::INFINITY;
        }
        count as f64 / sum
    }
}

impl Time {
    /// Create a new Time resource.
    pub(crate) fn new() -> Self {
        let now = Instant::now();
        Self {
            elapsed: 0.0,
            start: now,
            last_frame: now,
            delta: 0.0,
            smoothed_delta: 1.0 / 60.0,
            fixed_timestep: 1.0 / 60.0,
            fixed_accumulator: 0.0,
            use_fixed_timestep: false,
            frame_count: 0,
            time_scale: 1.0,
            fps_tracker: FpsTracker::default(),
        }
    }

    /// Called at the start of each frame.
    pub(crate) fn tick(&mut self) {
        let now = Instant::now();
        let raw_delta = now.duration_since(self.last_frame).as_secs_f64();
        self.last_frame = now;

        // Clamp delta to avoid spiral-of-death on lag spikes
        let clamped = raw_delta.min(0.25);
        self.delta = clamped * self.time_scale;
        self.smoothed_delta = self.smoothed_delta * 0.9 + self.delta * 0.1;
        self.elapsed += self.delta;
        self.frame_count += 1;
        self.fps_tracker.record(self.delta);
    }

    /// Check if a fixed update should run this frame.
    /// Returns the number of fixed steps to perform.
    pub(crate) fn fixed_step_count(&mut self) -> u32 {
        if !self.use_fixed_timestep {
            return 0;
        }
        self.fixed_accumulator += self.delta;
        let max_steps = 5; // Prevent spiral of death
        let mut steps = 0;
        while self.fixed_accumulator >= self.fixed_timestep && steps < max_steps {
            self.fixed_accumulator -= self.fixed_timestep;
            steps += 1;
        }
        if steps >= max_steps {
            self.fixed_accumulator = 0.0;
        }
        steps
    }

    // --- Public getters ---

    /// Seconds since the engine started.
    #[inline]
    pub fn elapsed(&self) -> f64 {
        self.elapsed
    }

    /// Duration of the last frame in seconds (affected by time_scale).
    #[inline]
    pub fn delta(&self) -> f64 {
        self.delta
    }

    /// Smoothed delta time (exponential moving average).
    #[inline]
    pub fn smoothed_delta(&self) -> f64 {
        self.smoothed_delta
    }

    /// Frames per second (averaged over last 60 frames).
    #[inline]
    pub fn fps(&self) -> f64 {
        self.fps_tracker.average_fps()
    }

    /// Current frame number (starts at 0, increments each frame).
    #[inline]
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Current time scale factor (1.0 = normal, 0.0 = paused, 2.0 = double speed).
    #[inline]
    pub fn time_scale(&self) -> f64 {
        self.time_scale
    }

    /// Set the time scale factor.
    pub fn set_time_scale(&mut self, scale: f64) {
        self.time_scale = scale.clamp(0.0, 10.0);
    }

    /// Enable fixed timestep updates at the given rate (in Hz).
    pub fn set_fixed_timestep(&mut self, fps: u32) {
        self.use_fixed_timestep = true;
        self.fixed_timestep = 1.0 / fps as f64;
        self.fixed_accumulator = 0.0;
    }

    /// Disable fixed timestep updates.
    pub fn disable_fixed_timestep(&mut self) {
        self.use_fixed_timestep = false;
        self.fixed_accumulator = 0.0;
    }

    /// Whether fixed timestep is enabled.
    #[inline]
    pub fn is_fixed_timestep_enabled(&self) -> bool {
        self.use_fixed_timestep
    }

    /// The fixed timestep duration in seconds.
    #[inline]
    pub fn fixed_delta(&self) -> f64 {
        self.fixed_timestep
    }
}

// --- Standalone timer utility ---

/// A simple countdown timer.
///
/// Useful for cooldowns, delays, and timed events.
///
/// # Example
/// ```
/// let mut timer = Timer::from_seconds(2.0, false);
/// // In update loop:
/// if timer.tick(delta) {
///     println!("2 seconds elapsed!");
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Timer {
    duration: f64,
    elapsed: f64,
    repeating: bool,
    finished: bool,
}

impl Timer {
    /// Create a timer with the given duration.
    pub fn from_seconds(seconds: f64, repeating: bool) -> Self {
        Self {
            duration: seconds,
            elapsed: 0.0,
            repeating,
            finished: false,
        }
    }

    /// Advance the timer by `delta` seconds. Returns `true` when it fires.
    pub fn tick(&mut self, delta: f64) -> bool {
        if self.finished && !self.repeating {
            return false;
        }
        self.elapsed += delta;
        if self.elapsed >= self.duration {
            if self.repeating {
                self.elapsed -= self.duration;
            } else {
                self.finished = true;
                self.elapsed = self.duration;
            }
            true
        } else {
            false
        }
    }

    /// Reset the timer to zero.
    pub fn reset(&mut self) {
        self.elapsed = 0.0;
        self.finished = false;
    }

    /// Set a new duration and reset.
    pub fn set_duration(&mut self, seconds: f64) {
        self.duration = seconds;
        self.reset();
    }

    /// Fraction completed (0.0 to 1.0).
    pub fn fraction(&self) -> f32 {
        (self.elapsed / self.duration).clamp(0.0, 1.0) as f32
    }

    /// Whether the timer has finished (non-repeating only).
    #[inline]
    pub fn is_finished(&self) -> bool {
        self.finished
    }

    /// Whether the timer is still running.
    #[inline]
    pub fn is_running(&self) -> bool {
        !self.finished
    }

    /// Remaining time in seconds.
    pub fn remaining(&self) -> f64 {
        (self.duration - self.elapsed).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_basic() {
        let mut t = Timer::from_seconds(1.0, false);
        assert!(!t.tick(0.5));
        assert!(!t.tick(0.4));
        assert!(t.tick(0.2)); // total 1.1s
        assert!(t.is_finished());
        assert!(!t.tick(0.5)); // non-repeating, won't fire again
    }

    #[test]
    fn test_timer_repeating() {
        let mut t = Timer::from_seconds(1.0, true);
        assert!(!t.tick(0.5));
        assert!(t.tick(0.5));
        assert!(!t.tick(0.3));
        assert!(t.tick(0.7));
    }
}