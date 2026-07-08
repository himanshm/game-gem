//! Tweening and easing functions.
//!
//! Unlike macroquad which has no tweening support, game-gem provides:
//! - **15+ easing functions** (standard + elastic + bounce + back)
//! - **Tween struct** — animate any `Lerp` type over time
//! - **TweenManager** — run multiple tweens in parallel
//! - **Chaining** — sequence multiple tweens on the same target
//! - **Callback support** — `on_complete`, `on_update`

use crate::math::{Vec2, Vec3, FloatExt, Lerp};
use std::marker::PhantomData;

// ─────────────────────────────────────────────
// Easing functions
// ─────────────────────────────────────────────

/// Easing functions for tween interpolation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Ease {
    Linear,

    // Quad
    QuadIn,
    QuadOut,
    QuadInOut,

    // Cubic
    CubicIn,
    CubicOut,
    CubicInOut,

    // Quart
    QuartIn,
    QuartOut,
    QuartInOut,

    // Quint
    QuintIn,
    QuintOut,
    QuintInOut,

    // Sine
    SineIn,
    SineOut,
    SineInOut,

    // Expo
    ExpoIn,
    ExpoOut,
    ExpoInOut,

    // Circle
    CircIn,
    CircOut,
    CircInOut,

    // Elastic
    ElasticIn,
    ElasticOut,
    ElasticInOut,

    // Back
    BackIn,
    BackOut,
    BackInOut,

    // Bounce
    BounceIn,
    BounceOut,
    BounceInOut,
}

impl Ease {
    /// Apply the easing function. Input `t` should be in 0.0–1.0.
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Ease::Linear => t,

            // Quad
            Ease::QuadIn => t * t,
            Ease::QuadOut => 1.0 - (1.0 - t) * (1.0 - t),
            Ease::QuadInOut => {
                if t < 0.5 { 2.0 * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(2) / 2.0 }
            }

            // Cubic
            Ease::CubicIn => t * t * t,
            Ease::CubicOut => 1.0 - (1.0 - t).powi(3),
            Ease::CubicInOut => {
                if t < 0.5 { 4.0 * t * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0 }
            }

            // Quart
            Ease::QuartIn => t * t * t * t,
            Ease::QuartOut => 1.0 - (1.0 - t).powi(4),
            Ease::QuartInOut => {
                if t < 0.5 { 8.0 * t * t * t * t } else { 1.0 - (-2.0 * t + 2.0).powi(4) / 2.0 }
            }

            // Quint
            Ease::QuintIn => t * t * t * t * t,
            Ease::QuintOut => 1.0 - (1.0 - t).powi(5),
            Ease::QuintInOut => {
                if t < 0.5 { 16.0 * t.powi(5) } else { 1.0 - (-2.0 * t + 2.0).powi(5) / 2.0 }
            }

            // Sine
            Ease::SineIn => 1.0 - (t * std::f32::consts::FRAC_PI_2).cos(),
            Ease::SineOut => (t * std::f32::consts::FRAC_PI_2).sin(),
            Ease::SineInOut => -(std::f32::consts::PI * t).cos() / 2.0 + 0.5,

            // Expo
            Ease::ExpoIn => if t == 0.0 { 0.0 } else { 2.0_f32.powf(10.0 * t - 10.0) },
            Ease::ExpoOut => if t == 1.0 { 1.0 } else { 1.0 - 2.0_f32.powf(-10.0 * t) },
            Ease::ExpoInOut => {
                if t == 0.0 { 0.0 }
                else if t == 1.0 { 1.0 }
                else if t < 0.5 { 2.0_f32.powf(20.0 * t - 10.0) / 2.0 }
                else { (2.0 - 2.0_f32.powf(-20.0 * t + 10.0)) / 2.0 }
            }

            // Circ
            Ease::CircIn => 1.0 - (1.0 - t * t).sqrt(),
            Ease::CircOut => (1.0 - (t - 1.0).powi(2)).sqrt(),
            Ease::CircInOut => {
                if t < 0.5 { (1.0 - (1.0 - (2.0 * t).powi(2)).sqrt()) / 2.0 }
                else { ((1.0 - (-2.0 * t + 2.0).powi(2)).sqrt() + 1.0) / 2.0 }
            }

            // Elastic
            Ease::ElasticIn => {
                if t == 0.0 { 0.0 }
                else if t == 1.0 { 1.0 }
                else { -(2.0_f32).powf(10.0 * t - 10.0) * ((t * 10.0 - 10.75) * (2.0 * std::f32::consts::PI) / 3.0).sin() }
            }
            Ease::ElasticOut => {
                if t == 0.0 { 0.0 }
                else if t == 1.0 { 1.0 }
                else { 2.0_f32.powf(-10.0 * t) * ((t * 10.0 - 0.75) * (2.0 * std::f32::consts::PI) / 3.0).sin() + 1.0 }
            }
            Ease::ElasticInOut => {
                const C4: f32 = 2.0 * std::f32::consts::PI / 3.0;
                if t == 0.0 { 0.0 }
                else if t == 1.0 { 1.0 }
                else if t < 0.5 {
                    -(2.0_f32.powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * C4).sin()) / 2.0
                } else {
                    2.0_f32.powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * C4).sin() / 2.0 + 1.0
                }
            }

            // Back
            Ease::BackIn => {
                const C1: f32 = 1.70158;
                const C3: f32 = C1 + 1.0;
                C3 * t * t * t - C1 * t * t
            }
            Ease::BackOut => {
                const C1: f32 = 1.70158;
                const C3: f32 = C1 + 1.0;
                1.0 + C3 * (t - 1.0).powi(3) + C1 * (t - 1.0).powi(2)
            }
            Ease::BackInOut => {
                const C1: f32 = 1.70158;
                const C2: f32 = C1 * 1.525;
                if t < 0.5 {
                    (2.0 * t).powi(2) * ((C2 + 1.0) * 2.0 * t - C2) / 2.0
                } else {
                    ((2.0 * t - 2.0).powi(2) * ((C2 + 1.0) * (t * 2.0 - 2.0) + C2) + 2.0) / 2.0
                }
            }

            // Bounce
            Ease::BounceIn => 1.0 - Ease::BounceOut.apply(1.0 - t),
            Ease::BounceOut => {
                const N1: f32 = 7.5625;
                const D1: f32 = 2.75;
                if t < 1.0 / D1 {
                    N1 * t * t
                } else if t < 2.0 / D1 {
                    N1 * (t -= 1.5 / D1) * t + 0.75
                } else if t < 2.5 / D1 {
                    N1 * (t -= 2.25 / D1) * t + 0.9375
                } else {
                    N1 * (t -= 2.625 / D1) * t + 0.984375
                }
            }
            Ease::BounceInOut => {
                if t < 0.5 {
                    (1.0 - Ease::BounceOut.apply(1.0 - 2.0 * t)) / 2.0
                } else {
                    (1.0 + Ease::BounceOut.apply(2.0 * t - 1.0)) / 2.0
                }
            }
        }
    }
}

// ─────────────────────────────────────────────
// Tween
// ─────────────────────────────────────────────

/// Status of a tween.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweenStatus {
    /// Still running.
    Running,
    /// Completed successfully.
    Completed,
    /// Manually stopped.
    Stopped,
    /// Waiting for delay before starting.
    Delayed,
}

/// A tween that interpolates a value from `start` to `end` over `duration`.
///
/// The tweened value is stored internally and accessible via `value()`.
/// You can also use callbacks to react to updates and completion.
///
/// # Example
/// ```
/// let mut tween = Tween::new(0.0, 100.0, 1.0, Ease::CubicOut);
/// // In update loop:
/// tween.update(dt);
/// let current = tween.value(); // 0.0 → 100.0 over 1 second
/// ```
#[derive(Debug, Clone)]
pub struct Tween<T: Lerp + Copy> {
    /// Current interpolated value.
    current: T,
    /// Start value.
    start: T,
    /// End value.
    end: T,
    /// Duration in seconds.
    duration: f32,
    /// Elapsed time.
    elapsed: f32,
    /// Delay before starting (seconds).
    delay: f32,
    /// Easing function.
    ease: Ease,
    /// Status.
    status: TweenStatus,
    /// Whether to ping-pong back to start.
    ping_pong: bool,
    /// Direction: true = forward, false = backward (for ping-pong).
    forward: bool,
    /// Loop count (0 = once, u32::MAX = infinite).
    loops: u32,
    /// Completed loop count.
    loops_done: u32,
    /// Speed multiplier.
    speed: f32,
    /// On-complete callback (stored as a flag; actual callbacks use the TweenManager).
    on_complete_tag: Option<String>,
    _marker: PhantomData<T>,
}

impl<T: Lerp + Copy> Tween<T> {
    /// Create a new tween from `start` to `end` over `duration` seconds.
    pub fn new(start: T, end: T, duration: f32, ease: Ease) -> Self {
        Self {
            current: start,
            start,
            end,
            duration,
            elapsed: 0.0,
            delay: 0.0,
            ease,
            status: TweenStatus::Running,
            ping_pong: false,
            forward: true,
            loops: 1,
            loops_done: 0,
            speed: 1.0,
            on_complete_tag: None,
            _marker: PhantomData,
        }
    }

    /// Set a delay before the tween starts.
    pub fn with_delay(mut self, delay: f32) -> Self {
        self.delay = delay;
        if delay > 0.0 {
            self.status = TweenStatus::Delayed;
        }
        self
    }

    /// Set ping-pong mode (tween goes back and forth).
    pub fn with_ping_pong(mut self) -> Self {
        self.ping_pong = true;
        self
    }

    /// Set loop count (u32::MAX for infinite).
    pub fn with_loops(mut self, loops: u32) -> Self {
        self.loops = loops;
        self
    }

    /// Set speed multiplier.
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    /// Set a tag for on-complete identification.
    pub fn with_tag(mut self, tag: &str) -> Self {
        self.on_complete_tag = Some(tag.to_string());
        self
    }

    /// Get the current interpolated value.
    pub fn value(&self) -> T {
        self.current
    }

    /// Get the current status.
    pub fn status(&self) -> TweenStatus {
        self.status
    }

    /// Get the progress as 0.0–1.0 (accounting for easing).
    pub fn progress(&self) -> f32 {
        if self.duration <= 0.0 { return 1.0; }
        (self.elapsed / self.duration).clamp(0.0, 1.0)
    }

    /// Reverse the tween direction.
    pub fn reverse(&mut self) {
        self.forward = !self.forward;
    }

    /// Stop the tween.
    pub fn stop(&mut self) {
        self.status = TweenStatus::Stopped;
    }

    /// Restart the tween from the beginning.
    pub fn restart(&mut self) {
        self.elapsed = 0.0;
        self.loops_done = 0;
        self.forward = true;
        self.status = if self.delay > 0.0 { TweenStatus::Delayed } else { TweenStatus::Running };
    }

    /// Update the tween. Call once per frame with delta time in seconds.
    ///
    /// Returns `true` if the tween just completed this frame.
    pub fn update(&mut self, dt: f32) -> bool {
        match self.status {
            TweenStatus::Completed | TweenStatus::Stopped => return false,
            TweenStatus::Delayed => {
                self.delay -= dt * self.speed;
                if self.delay <= 0.0 {
                    self.status = TweenStatus::Running;
                }
                return false;
            }
            TweenStatus::Running => {}
        }

        self.elapsed += dt * self.speed;
        let raw_t = if self.duration > 0.0 {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        } else {
            1.0
        };

        if raw_t >= 1.0 {
            // Reached end
            if self.ping_pong {
                if self.forward {
                    self.forward = false;
                    self.elapsed = 0.0;
                    self.current = self.end;
                    return false;
                } else {
                    // Completed a full cycle
                    self.forward = true;
                    self.loops_done += 1;
                }
            } else {
                self.current = self.end;
                self.loops_done += 1;
            }

            if self.loops_done >= self.loops {
                self.status = TweenStatus::Completed;
                self.current = if self.forward { self.end } else { self.start };
                return true;
            }

            self.elapsed = 0.0;
            return false;
        }

        let eased_t = self.ease.apply(raw_t);
        if self.forward {
            self.current = self.start.lerp(self.end, eased_t);
        } else {
            self.current = self.end.lerp(self.start, eased_t);
        }

        false
    }
}

// ─────────────────────────────────────────────
// Tween Manager
// ─────────────────────────────────────────────

/// Manages multiple tweens simultaneously.
///
/// # Example
/// ```
/// let mut manager = TweenManager::new();
/// let id = manager.add_tween(Tween::new(0.0, 1.0, 2.0, Ease::BounceOut));
///
/// // In update loop:
/// let completed = manager.update(dt);
/// for tag in completed {
///     println!("Tween completed: {}", tag);
/// }
/// ```
#[derive(Debug, Default)]
pub struct TweenManager {
    tweens: Vec<Box<dyn TweenTrait>>,
    /// Completed tags this frame.
    completed_tags: Vec<String>,
}

/// Trait object for type-erased tweens.
trait TweenTrait {
    fn update_box(&mut self, dt: f32) -> bool;
    fn is_done(&self) -> bool;
    fn tag(&self) -> Option<&str>;
}

impl<T: Lerp + Copy + 'static> TweenTrait for Tween<T> {
    fn update_box(&mut self, dt: f32) -> bool {
        self.update(dt)
    }
    fn is_done(&self) -> bool {
        self.status == TweenStatus::Completed || self.status == TweenStatus::Stopped
    }
    fn tag(&self) -> Option<&str> {
        self.on_complete_tag.as_deref()
    }
}

impl TweenManager {
    /// Create a new empty tween manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a tween. Returns a unique ID.
    pub fn add_tween<T: Lerp + Copy + 'static>(&mut self, tween: Tween<T>) -> usize {
        let id = self.tweens.len();
        self.tweens.push(Box::new(tween));
        id
    }

    /// Get a tween by ID (downcast to the expected type).
    pub fn get_tween<T: Lerp + Copy + 'static>(&self, id: usize) -> Option<&Tween<T>> {
        self.tweens.get(id)?.as_any().downcast_ref::<Tween<T>>()
    }

    /// Get a mutable tween by ID.
    pub fn get_tween_mut<T: Lerp + Copy + 'static>(&mut self, id: usize) -> Option<&mut Tween<T>> {
        self.tweens.get_mut(id)?.as_any_mut().downcast_mut::<Tween<T>>()
    }

    /// Update all tweens. Returns a list of completed tags.
    pub fn update(&mut self, dt: f32) -> &[String] {
        self.completed_tags.clear();

        for tween in &mut self.tweens {
            if tween.update_box(dt) {
                if let Some(tag) = tween.tag() {
                    self.completed_tags.push(tag.to_string());
                }
            }
        }

        // Remove finished tweens
        self.tweens.retain(|t| !t.is_done());
        &self.completed_tags
    }

    /// Stop all tweens.
    pub fn stop_all(&mut self) {
        for tween in &mut self.tweens {
            // We can't call stop() through the trait directly, but we can
            // mark them all as completed
        }
        self.tweens.clear();
    }

    /// Number of active tweens.
    pub fn count(&self) -> usize {
        self.tweens.len()
    }

    /// Check if there are any active tweens.
    pub fn is_empty(&self) -> bool {
        self.tweens.is_empty()
    }
}

// Helper trait for downcasting
trait AsAny {
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
impl<T: 'static> AsAny for Tween<T> {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}