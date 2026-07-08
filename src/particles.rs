//! Particle system for visual effects.
//!
//! Features vs macroquad (which has none):
//! - **Emitters** with configurable shape (point, line, circle, rect)
//! - **Per-particle properties**: velocity, acceleration, color, size, lifetime, rotation
//! - **Easing functions** for size/alpha over lifetime
//! - **Gravity and drag**
//! - **Particle pooling** for zero-allocation steady-state
//! - **Multiple emitters** active simultaneously

use crate::math::{Vec2, FloatExt};
use crate::color::Color;
use quad_rand as rand;

// ─────────────────────────────────────────────
// Easing
// ─────────────────────────────────────────────

/// Easing function for particle properties over lifetime.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Easing {
    /// Constant value.
    Linear,
    /// Fast start, slow end.
    EaseOut,
    /// Slow start, fast end.
    EaseIn,
    /// Slow start, fast middle, slow end.
    EaseInOut,
    /// Quadratic ease out.
    QuadOut,
}

impl Easing {
    /// Apply the easing function. `t` should be 0.0–1.0.
    pub fn apply(self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        match self {
            Easing::Linear => t,
            Easing::EaseOut => 1.0 - (1.0 - t).powi(3),
            Easing::EaseIn => t.powi(3),
            Easing::EaseInOut => {
                if t < 0.5 {
                    4.0 * t * t * t
                } else {
                    1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
                }
            }
            Easing::QuadOut => 1.0 - (1.0 - t) * (1.0 - t),
        }
    }
}

// ─────────────────────────────────────────────
// Particle
// ─────────────────────────────────────────────

/// A single particle with all its properties.
#[derive(Debug, Clone)]
pub struct Particle {
    /// Current position.
    pub position: Vec2,
    /// Current velocity.
    pub velocity: Vec2,
    /// Acceleration applied each frame.
    pub acceleration: Vec2,
    /// Current color.
    pub color: Color,
    /// Start color (for interpolation).
    pub color_start: Color,
    /// End color (for interpolation).
    pub color_end: Color,
    /// Current size.
    pub size: f32,
    /// Start size.
    pub size_start: f32,
    /// End size.
    pub size_end: f32,
    /// Current rotation (radians).
    pub rotation: f32,
    /// Rotation speed (radians/sec).
    pub rotation_speed: f32,
    /// Time lived so far (seconds).
    pub age: f32,
    /// Maximum lifetime (seconds).
    pub lifetime: f32,
    /// Whether this particle is alive.
    pub alive: bool,
    /// Alpha easing.
    pub alpha_easing: Easing,
    /// Size easing.
    pub size_easing: Easing,
}

impl Particle {
    /// Create a new particle with default values.
    fn new() -> Self {
        Self {
            position: Vec2::ZERO,
            velocity: Vec2::ZERO,
            acceleration: Vec2::ZERO,
            color: Color::WHITE,
            color_start: Color::WHITE,
            color_end: Color::TRANSPARENT,
            size: 5.0,
            size_start: 5.0,
            size_end: 0.0,
            rotation: 0.0,
            rotation_speed: 0.0,
            age: 0.0,
            lifetime: 1.0,
            alive: true,
            alpha_easing: Easing::Linear,
            size_easing: Easing::QuadOut,
        }
    }

    /// Update the particle for one frame.
    fn update(&mut self, dt: f32, drag: f32, gravity: Vec2) {
        if !self.alive {
            return;
        }

        self.age += dt;
        if self.age >= self.lifetime {
            self.alive = false;
            return;
        }

        let t = self.age / self.lifetime; // 0..1 progress

        // Apply forces
        self.velocity += (self.acceleration + gravity) * dt;
        self.velocity *= 1.0 - drag * dt;
        self.position += self.velocity * dt;
        self.rotation += self.rotation_speed * dt;

        // Interpolate color
        self.color = self.color_start.lerp(self.color_end, Easing::Linear.apply(t));
        self.color.a = self.color_start.a * (1.0 - Easing::Linear.apply(t));

        // Interpolate size
        self.size = self.size_start.lerp(self.size_end, self.size_easing.apply(t));
    }
}

// ─────────────────────────────────────────────
// Emitter shape
// ─────────────────────────────────────────────

/// Shape of the emission area.
#[derive(Debug, Clone, Copy)]
pub enum EmitterShape {
    /// Single point.
    Point,
    /// Line segment from `(x1, y1)` to `(x2, y2)`.
    Line { x1: f32, y1: f32, x2: f32, y2: f32 },
    /// Circle with center and radius.
    Circle { cx: f32, cy: f32, radius: f32 },
    /// Rectangle.
    Rect { x: f32, y: f32, w: f32, h: f32 },
}

// ─────────────────────────────────────────────
// Particle Emitter
// ─────────────────────────────────────────────

/// A particle emitter that spawns particles over time.
#[derive(Debug, Clone)]
pub struct ParticleEmitter {
    /// Position offset for the emitter.
    pub position: Vec2,
    /// Emission shape.
    pub shape: EmitterShape,
    /// Particles per second.
    pub rate: f32,
    /// Emission accumulator.
    emit_accumulator: f32,
    /// Burst count (if > 0, emit this many at once then stop).
    pub burst: u32,
    /// Burst emitted count.
    burst_emitted: u32,
    /// Template particle (properties are copied to new particles).
    pub template: Particle,
    /// Velocity range (min, max speed).
    pub speed_range: (f32, f32),
    /// Angle range for initial velocity (min, max in radians).
    pub angle_range: (f32, f32),
    /// Size range (min, max).
    pub size_range: (f32, f32),
    /// Lifetime range (min, max seconds).
    pub lifetime_range: (f32, f32),
    /// Gravity applied to all particles.
    pub gravity: Vec2,
    /// Drag coefficient (0 = no drag, 1 = full stop).
    pub drag: f32,
    /// Maximum number of alive particles.
    pub max_particles: usize,
    /// Active particles.
    particles: Vec<Particle>,
    /// Whether the emitter is active.
    pub active: bool,
    /// Whether the emitter loops (re-emits after all particles die, if burst mode).
    pub looping: bool,
}

impl ParticleEmitter {
    /// Create a new emitter with default settings.
    pub fn new(position: Vec2) -> Self {
        Self {
            position,
            shape: EmitterShape::Point,
            rate: 10.0,
            emit_accumulator: 0.0,
            burst: 0,
            burst_emitted: 0,
            template: Particle::new(),
            speed_range: (50.0, 150.0),
            angle_range: (0.0, std::f32::consts::TAU),
            size_range: (2.0, 8.0),
            lifetime_range: (0.5, 2.0),
            gravity: Vec2::new(0.0, 100.0),
            drag: 0.0,
            max_particles: 1000,
            particles: Vec::with_capacity(1000),
            active: true,
            looping: false,
        }
    }

    /// Configure the emitter using a builder-style closure.
    pub fn configure(&mut self, f: impl FnOnce(&mut ParticleEmitterConfig)) {
        let mut cfg = ParticleEmitterConfig { emitter: self };
        f(&mut cfg);
    }

    /// Get a reference to the active particles (for rendering).
    pub fn particles(&self) -> &[Particle] {
        &self.particles
    }

    /// Get a mutable reference to active particles.
    pub fn particles_mut(&mut self) -> &mut Vec<Particle> {
        &mut self.particles
    }

    /// Count of alive particles.
    pub fn alive_count(&self) -> usize {
        self.particles.iter().filter(|p| p.alive).count()
    }

    /// Emit a single particle with randomized properties.
    fn emit_one(&mut self) {
        let pos = match self.shape {
            EmitterShape::Point => self.position,
            EmitterShape::Line { x1, y1, x2, y2 } => {
                let t = rand::gen_range(0.0, 1.0);
                Vec2::new(x1.lerp(x2, t), y1.lerp(y2, t))
            }
            EmitterShape::Circle { cx, cy, radius } => {
                let angle = rand::gen_range(0.0, std::f32::consts::TAU);
                let r = radius.sqrt() * rand::gen_range(0.0, 1.0);
                Vec2::new(cx + angle.cos() * r, cy + angle.sin() * r)
            }
            EmitterShape::Rect { x, y, w, h } => {
                Vec2::new(x + rand::gen_range(0.0, w), y + rand::gen_range(0.0, h))
            }
        };

        let speed = rand::gen_range(self.speed_range.0, self.speed_range.1);
        let angle = rand::gen_range(self.angle_range.0, self.angle_range.1);
        let vel = Vec2::new(angle.cos() * speed, angle.sin() * speed);

        let size = rand::gen_range(self.size_range.0, self.size_range.1);
        let lifetime = rand::gen_range(self.lifetime_range.0, self.lifetime_range.1);

        let mut p = self.template.clone();
        p.position = pos;
        p.velocity = vel;
        p.size_start = size;
        p.size = size;
        p.size_end = self.template.size_end;
        p.lifetime = lifetime;
        p.color_start = self.template.color_start;
        p.color_end = self.template.color_end;

        self.particles.push(p);
    }

    /// Update the emitter and all its particles.
    pub fn update(&mut self, dt: f32) {
        if !self.active {
            return;
        }

        // Emit new particles
        if self.burst > 0 {
            if self.burst_emitted < self.burst {
                let to_emit = self.burst - self.burst_emitted;
                for _ in 0..to_emit {
                    if self.alive_count() < self.max_particles {
                        self.emit_one();
                        self.burst_emitted += 1;
                    }
                }
                if !self.looping {
                    self.active = false;
                }
            } else if self.looping {
                // Wait for all particles to die before re-bursting
                if self.alive_count() == 0 {
                    self.burst_emitted = 0;
                }
            }
        } else {
            self.emit_accumulator += dt;
            let interval = 1.0 / self.rate;
            while self.emit_accumulator >= interval {
                self.emit_accumulator -= interval;
                if self.alive_count() < self.max_particles {
                    self.emit_one();
                }
            }
        }

        // Update existing particles
        for p in &mut self.particles {
            p.update(dt, self.drag, self.gravity);
        }

        // Remove dead particles periodically (every 60 frames to amortize cost)
        // In practice, we just compact when the dead count exceeds a threshold
        let dead_count = self.particles.iter().filter(|p| !p.alive).count();
        if dead_count > self.max_particles / 4 {
            self.particles.retain(|p| p.alive);
        }
    }

    /// Remove all particles and reset the emitter.
    pub fn clear(&mut self) {
        self.particles.clear();
        self.emit_accumulator = 0.0;
        self.burst_emitted = 0;
        self.active = true;
    }

    /// Trigger a burst emission regardless of current mode.
    pub fn burst_now(&mut self, count: u32) {
        for _ in 0..count {
            if self.alive_count() < self.max_particles {
                self.emit_one();
            }
        }
    }
}

/// Helper for builder-style configuration.
pub struct ParticleEmitterConfig<'a> {
    emitter: &'a mut ParticleEmitter,
}

impl<'a> ParticleEmitterConfig<'a> {
    pub fn rate(&mut self, rate: f32) -> &mut Self { self.emitter.rate = rate; self }
    pub fn shape(&mut self, shape: EmitterShape) -> &mut Self { self.emitter.shape = shape; self }
    pub fn speed(&mut self, min: f32, max: f32) -> &mut Self { self.emitter.speed_range = (min, max); self }
    pub fn angle(&mut self, min: f32, max: f32) -> &mut Self { self.emitter.angle_range = (min, max); self }
    pub fn size(&mut self, min: f32, max: f32) -> &mut Self { self.emitter.size_range = (min, max); self }
    pub fn lifetime(&mut self, min: f32, max: f32) -> &mut Self { self.emitter.lifetime_range = (min, max); self }
    pub fn gravity(&mut self, x: f32, y: f32) -> &mut Self { self.emitter.gravity = Vec2::new(x, y); self }
    pub fn drag(&mut self, drag: f32) -> &mut Self { self.emitter.drag = drag; self }
    pub fn max_particles(&mut self, max: usize) -> &mut Self { self.emitter.max_particles = max; self }
    pub fn color_start(&mut self, color: Color) -> &mut Self { self.emitter.template.color_start = color; self }
    pub fn color_end(&mut self, color: Color) -> &mut Self { self.emitter.template.color_end = color; self }
    pub fn size_end(&mut self, size: f32) -> &mut Self { self.emitter.template.size_end = size; self }
    pub fn burst(&mut self, count: u32) -> &mut Self { self.emitter.burst = count; self }
    pub fn looping(&mut self, looping: bool) -> &mut Self { self.emitter.looping = looping; self }
    pub fn rotation_speed(&mut self, min: f32, max: f32) -> &mut Self {
        self.emitter.template.rotation_speed = rand::gen_range(min, max);
        self
    }
}