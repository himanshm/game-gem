//! Sprite animation system.
//!
//! Features:
//! - **Sprite sheets** with configurable frames, rows, columns
//! - **Animation state machine** — idle, run, jump, etc.
//! - **Blending between animations**
//! - **Events** at specific keyframes (footstep sounds, spawn effects)
//! - **Speed control** and **ping-pong** playback

use crate::math::{Vec2, Rect, FloatExt};
use crate::time::Timer;

// ─────────────────────────────────────────────
// Animation data
// ─────────────────────────────────────────────

/// A single animation defined from a sprite sheet.
#[derive(Debug, Clone)]
pub struct Animation {
    /// Human-readable name (e.g., "idle", "run", "attack").
    pub name: String,
    /// Region of the sprite sheet: (x, y, width, height) per frame in pixels.
    pub frame_size: Vec2,
    /// Frames in this animation (indices into the sprite sheet row).
    pub frames: Vec<AnimationFrame>,
    /// Playback speed (1.0 = normal, 2.0 = double speed).
    pub speed: f32,
    /// Whether to loop.
    pub looping: bool,
    /// Whether to ping-pong (play forward then backward).
    pub ping_pong: bool,
    /// Offset from the sprite's position (for attack effects, etc.).
    pub offset: Vec2,
}

/// A single frame in an animation.
#[derive(Debug, Clone)]
pub struct AnimationFrame {
    /// Column index in the sprite sheet.
    pub column: u32,
    /// Row index in the sprite sheet.
    pub row: u32,
    /// Duration of this specific frame in seconds.
    /// If 0.0, uses the animation's default timing.
    pub duration: f32,
    /// Optional event to trigger when this frame is first shown.
    pub event: Option<String>,
}

impl Animation {
    /// Create a simple animation with evenly-spaced frames in a single row.
    ///
    /// - `name` — animation name
    /// - `row` — sprite sheet row
    /// - `start_col` — first column (inclusive)
    /// - `end_col` — last column (inclusive)
    /// - `frame_duration` — seconds per frame
    pub fn from_row(
        name: &str,
        row: u32,
        start_col: u32,
        end_col: u32,
        frame_duration: f32,
    ) -> Self {
        let frames = (start_col..=end_col)
            .map(|col| AnimationFrame {
                column: col,
                row,
                duration: frame_duration,
                event: None,
            })
            .collect();

        Self {
            name: name.to_string(),
            frame_size: Vec2::ZERO,
            frames,
            speed: 1.0,
            looping: true,
            ping_pong: false,
            offset: Vec2::ZERO,
        }
    }

    /// Builder: set frame size.
    pub fn with_frame_size(mut self, w: f32, h: f32) -> Self {
        self.frame_size = Vec2::new(w, h);
        self
    }

    /// Builder: set speed.
    pub fn with_speed(mut self, speed: f32) -> Self {
        self.speed = speed;
        self
    }

    /// Builder: set looping.
    pub fn with_looping(mut self, looping: bool) -> Self {
        self.looping = looping;
        self
    }

    /// Builder: set ping-pong.
    pub fn with_ping_pong(mut self) -> Self {
        self.ping_pong = true;
        self
    }

    /// Builder: set offset.
    pub fn with_offset(mut self, x: f32, y: f32) -> Self {
        self.offset = Vec2::new(x, y);
        self
    }

    /// Add an event to a specific frame.
    pub fn with_frame_event(mut self, frame_index: usize, event: &str) -> Self {
        if frame_index < self.frames.len() {
            self.frames[frame_index].event = Some(event.to_string());
        }
        self
    }

    /// Total duration of the animation in seconds.
    pub fn total_duration(&self) -> f32 {
        self.frames.iter().map(|f| f.duration).sum::<f32>() / self.speed
    }

    /// Number of frames.
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }
}

// ─────────────────────────────────────────────
// Animation Player
// ─────────────────────────────────────────────

/// Playback state for an animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnimationState {
    /// Currently playing.
    Playing,
    /// Animation finished (non-looping).
    Finished,
    /// Manually paused.
    Paused,
}

/// An animation player that controls playback of one animation at a time.
///
/// # Example
/// ```
/// let mut player = AnimationPlayer::new();
/// player.play(&idle_anim);
/// player.play(&run_anim);
///
/// // In update:
/// let events = player.update(dt);
/// for event in events {
///     if event == "footstep" { play_sound("step.wav"); }
/// }
/// ```
#[derive(Debug)]
pub struct AnimationPlayer {
    /// Current animation (reference).
    current: Option<Animation>,
    /// Index of the current frame.
    frame_index: usize,
    /// Timer for the current frame.
    frame_timer: f32,
    /// Playback state.
    state: AnimationState,
    /// Whether playing forward (false = playing backward for ping-pong).
    forward: bool,
    /// Events triggered this frame.
    pending_events: Vec<String>,
}

impl Default for AnimationPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl AnimationPlayer {
    /// Create a new animation player.
    pub fn new() -> Self {
        Self {
            current: None,
            frame_index: 0,
            frame_timer: 0.0,
            state: AnimationState::Finished,
            forward: true,
            pending_events: Vec::new(),
        }
    }

    /// Start playing an animation.
    ///
    /// If `restart` is true, always restarts from frame 0.
    /// If false, only restarts if it's a different animation.
    pub fn play(&mut self, anim: &Animation) {
        let is_same = self.current.as_ref().map(|c| c.name == anim.name).unwrap_or(false);
        if !is_same || self.state == AnimationState::Finished {
            self.current = Some(anim.clone());
            self.frame_index = 0;
            self.frame_timer = 0.0;
            self.state = AnimationState::Playing;
            self.forward = true;
            self.pending_events.clear();
        }
    }

    /// Force-restart the current animation.
    pub fn restart(&mut self) {
        self.frame_index = 0;
        self.frame_timer = 0.0;
        self.state = AnimationState::Playing;
        self.forward = true;
    }

    /// Pause the animation.
    pub fn pause(&mut self) {
        if self.state == AnimationState::Playing {
            self.state = AnimationState::Paused;
        }
    }

    /// Resume from pause.
    pub fn resume(&mut self) {
        if self.state == AnimationState::Paused {
            self.state = AnimationState::Playing;
        }
    }

    /// Stop and reset.
    pub fn stop(&mut self) {
        self.current = None;
        self.frame_index = 0;
        self.frame_timer = 0.0;
        self.state = AnimationState::Finished;
    }

    /// Get the current animation name, if any.
    pub fn current_name(&self) -> Option<&str> {
        self.current.as_ref().map(|a| a.name.as_str())
    }

    /// Get the current frame's source rectangle (UV region in the sprite sheet).
    pub fn current_frame_rect(&self) -> Option<(u32, u32, f32, f32)> {
        let anim = self.current.as_ref()?;
        let frame = anim.frames.get(self.frame_index)?;
        Some((frame.column, frame.row, anim.frame_size.x, anim.frame_size.y))
    }

    /// Get the current frame index.
    pub fn frame_index(&self) -> usize {
        self.frame_index
    }

    /// Get the playback state.
    pub fn state(&self) -> AnimationState {
        self.state
    }

    /// Get the animation offset.
    pub fn offset(&self) -> Vec2 {
        self.current.as_ref().map(|a| a.offset).unwrap_or(Vec2::ZERO)
    }

    /// Update the animation player. Returns events triggered this frame.
    pub fn update(&mut self, dt: f32) -> &[String] {
        self.pending_events.clear();

        if self.state != AnimationState::Playing {
            return &self.pending_events;
        }

        let anim = match &self.current {
            Some(a) => a,
            None => return &self.pending_events,
        };

        if anim.frames.is_empty() {
            self.state = AnimationState::Finished;
            return &self.pending_events;
        }

        let frame = &anim.frames[self.frame_index];
        let effective_duration = if frame.duration > 0.0 {
            frame.duration / anim.speed
        } else {
            0.1 / anim.speed
        };

        self.frame_timer += dt;

        if self.frame_timer >= effective_duration {
            self.frame_timer -= effective_duration;

            // Fire frame event
            if let Some(event) = &frame.event {
                self.pending_events.push(event.clone());
            }

            // Advance frame
            if anim.ping_pong {
                if self.forward {
                    if self.frame_index + 1 >= anim.frames.len() {
                        self.forward = false;
                        self.frame_index = self.frame_index.saturating_sub(1);
                    } else {
                        self.frame_index += 1;
                    }
                } else {
                    if self.frame_index == 0 {
                        if anim.looping {
                            self.forward = true;
                            self.frame_index = 1;
                        } else {
                            self.state = AnimationState::Finished;
                        }
                    } else {
                        self.frame_index -= 1;
                    }
                }
            } else {
                if self.frame_index + 1 >= anim.frames.len() {
                    if anim.looping {
                        self.frame_index = 0;
                    } else {
                        self.state = AnimationState::Finished;
                    }
                } else {
                    self.frame_index += 1;
                }
            }
        }

        &self.pending_events
    }
}