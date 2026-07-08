//! Camera / viewport system with multiple camera support.
//!
//! Advantages over macroquad's single global camera:
//! - **Multiple cameras** — render different layers with different cameras
//! - **Smooth follow** — built-in lerp-based following with deadzone
//! - **Camera shake** — one-liner screen shake
//! - **Zoom limits** — clamp min/max zoom
//! - **Viewport splitting** — split-screen support

use crate::math::{Vec2, Rect, Mat4, Transform, FloatExt};

/// A 2D camera that controls the view transform.
///
/// By default, the camera is at the center of the screen looking "down"
/// the Y axis (screen-space: +Y = down).
#[derive(Debug, Clone)]
pub struct Camera {
    /// World-space position (the "eye" point — center of the screen).
    pub position: Vec2,
    /// Rotation in radians.
    pub rotation: f32,
    /// Zoom level (1.0 = default, >1 = zoom in, <1 = zoom out).
    pub zoom: f32,
    /// Minimum zoom level.
    pub min_zoom: f32,
    /// Maximum zoom level.
    pub max_zoom: f32,
    /// Target position for smooth following.
    follow_target: Option<Vec2>,
    /// Follow lerp speed (0 = no movement, 1 = instant snap).
    follow_lerp: f32,
    /// Deadzone around the target before the camera starts moving.
    follow_deadzone: f32,
    /// Active shake.
    shake: CameraShake,
    /// The viewport rect on screen (for split-screen).
    viewport: Option<Rect>,
    /// Render layer mask (which layers this camera renders).
    layers: u32,
    /// Z-order (cameras with lower z render first).
    z_order: i32,
}

/// Camera shake state.
#[derive(Debug, Clone, Default)]
struct CameraShake {
    intensity: f32,
    duration: f32,
    remaining: f32,
    offset: Vec2,
}

impl Camera {
    /// Create a new camera at the given position.
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            rotation: 0.0,
            zoom: 1.0,
            min_zoom: 0.01,
            max_zoom: 100.0,
            follow_target: None,
            follow_lerp: 0.1,
            follow_deadzone: 0.0,
            shake: CameraShake::default(),
            viewport: None,
            layers: u32::MAX, // All layers
            z_order: 0,
        }
    }

    /// Create a camera centered at the origin.
    pub fn centered() -> Self {
        Self::new(0.0, 0.0)
    }

    /// Set the zoom level (clamped to min/max).
    pub fn set_zoom(&mut self, zoom: f32) {
        self.zoom = zoom.clamp(self.min_zoom, self.max_zoom);
    }

    /// Set zoom limits.
    pub fn set_zoom_limits(&mut self, min: f32, max: f32) {
        self.min_zoom = min.min(max);
        self.max_zoom = max.max(min);
        self.zoom = self.zoom.clamp(self.min_zoom, self.max_zoom);
    }

    /// Make the camera follow a target position with smooth lerp.
    pub fn follow(&mut self, target: Vec2, lerp_speed: f32) {
        self.follow_target = Some(target);
        self.follow_lerp = lerp_speed;
    }

    /// Set the deadzone for follow (camera won't move while target is within this distance).
    pub fn set_follow_deadzone(&mut self, deadzone: f32) {
        self.follow_deadzone = deadzone;
    }

    /// Stop following.
    pub fn stop_following(&mut self) {
        self.follow_target = None;
    }

    /// Trigger screen shake.
    ///
    /// - `intensity` — maximum pixel offset in each direction
    /// - `duration` — how long the shake lasts (seconds)
    pub fn shake(&mut self, intensity: f32, duration: f32) {
        self.shake.intensity = intensity;
        self.shake.duration = duration;
        self.shake.remaining = duration;
    }

    /// Set the viewport rectangle (for split-screen).
    ///
    /// Coordinates are in screen pixels (0,0 = top-left).
    pub fn set_viewport(&mut self, viewport: Rect) {
        self.viewport = Some(viewport);
    }

    /// Clear the viewport (render to full screen).
    pub fn clear_viewport(&mut self) {
        self.viewport = None;
    }

    /// Set which layers this camera renders (bitmask).
    pub fn set_layers(&mut self, layers: u32) {
        self.layers = layers;
    }

    /// Set the z-order (lower renders first).
    pub fn set_z_order(&mut self, z: i32) {
        self.z_order = z;
    }

    /// Update the camera (call once per frame).
    ///
    /// Handles smooth following and shake.
    pub fn update(&mut self, dt: f32) {
        // Smooth follow
        if let Some(target) = self.follow_target {
            let diff = target - self.position;
            let dist = diff.length();
            if dist > self.follow_deadzone {
                self.position = self.position.move_toward(target, dist * self.follow_lerp);
            }
        }

        // Shake
        if self.shake.remaining > 0.0 {
            self.shake.remaining -= dt;
            let t = self.shake.remaining / self.shake.duration;
            let current_intensity = self.shake.intensity * t;
            let angle = quad_rand::gen_range(0.0, std::f32::consts::TAU);
            self.shake.offset = Vec2::new(
                angle.cos() * current_intensity,
                angle.sin() * current_intensity,
            );
        } else {
            self.shake.offset = Vec2::ZERO;
        }
    }

    /// Get the view-projection matrix for this camera.
    ///
    /// This transforms world coordinates to screen coordinates.
    pub fn view_matrix(&self, screen_size: Vec2) -> Mat4 {
        let mut view = Mat4::IDENTITY;

        // Translate so camera position is at screen center
        let offset = self.position + self.shake.offset;
        view = Mat4::from_translation(Vec3::new(
            -offset.x,
            -offset.y,
            0.0,
        ));

        // Rotate
        if self.rotation.abs() > 1e-6 {
            view = Mat4::from_rotation_z(-self.rotation) * view;
        }

        // Scale (zoom)
        if (self.zoom - 1.0).abs() > 1e-6 {
            let scale = Mat4::from_scale(Vec3::new(self.zoom, self.zoom, 1.0));
            let to_center = Mat4::from_translation(Vec3::new(
                screen_size.x * 0.5,
                screen_size.y * 0.5,
                0.0,
            ));
            let from_center = Mat4::from_translation(Vec3::new(
                -screen_size.x * 0.5,
                -screen_size.y * 0.5,
                0.0,
            ));
            view = to_center * scale * from_center * view;
        }

        view
    }

    /// Convert screen coordinates to world coordinates.
    pub fn screen_to_world(&self, screen_pos: Vec2, screen_size: Vec2) -> Vec2 {
        let center = screen_size * 0.5;
        let mut world = (screen_pos - center) / self.zoom;

        // Un-rotate
        if self.rotation.abs() > 1e-6 {
            world = world.rotated(-self.rotation);
        }

        world + self.position + self.shake.offset
    }

    /// Convert world coordinates to screen coordinates.
    pub fn world_to_screen(&self, world_pos: Vec2, screen_size: Vec2) -> Vec2 {
        let center = screen_size * 0.5;
        let relative = world_pos - self.position - self.shake.offset;

        let rotated = if self.rotation.abs() > 1e-6 {
            relative.rotated(self.rotation)
        } else {
            relative
        };

        rotated * self.zoom + center
    }

    /// Get the visible world-space rectangle (accounting for zoom and position).
    pub fn visible_rect(&self, screen_size: Vec2) -> Rect {
        let half_extents = screen_size / (2.0 * self.zoom);
        Rect::centered(self.position, half_extents * 2.0)
    }
}