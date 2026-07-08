//! # game-gem Math
//!
//! A comprehensive 2D/3D math library built on `glam` with game-specific extensions.
//!
//! Re-exports all necessary `glam` types and adds:
//! - [`Rect`] — axis-aligned rectangle with intersection/containment helpers
//! - [`Transform`] — 2D transform (position, rotation, scale) with hierarchy support
//! - [`Lerp`] trait — generic linear interpolation
//! - [`Angle`] — type-safe angle wrapper (radians, degrees)

mod rect;
mod transform;
mod lerp;
mod angle;

pub use glam::{Vec2, Vec3, Vec4, Mat4, Quat};
pub use rect::Rect;
pub use transform::Transform;
pub use lerp::Lerp;
pub use angle::Angle;

// --- Convenience constructors & extension methods ---

/// Shorthand: `vec2(x, y)`
#[inline(always)]
pub const fn vec2(x: f32, y: f32) -> Vec2 {
    Vec2::new(x, y)
}

/// Shorthand: `vec3(x, y, z)`
#[inline(always)]
pub const fn vec3(x: f32, y: f32, z: f32) -> Vec3 {
    Vec3::new(x, y, z)
}

/// Shorthand: `ivec2(x, y)`
#[inline(always)]
pub const fn ivec2(x: i32, y: i32) -> glam::IVec2 {
    glam::IVec2::new(x, y)
}

/// Common constants.
pub mod consts {
    use super::*;

    /// (0, 0)
    pub const ZERO: Vec2 = Vec2::new(0.0, 0.0);
    /// (1, 0)
    pub const RIGHT: Vec2 = Vec2::new(1.0, 0.0);
    /// (-1, 0)
    pub const LEFT: Vec2 = Vec2::new(-1.0, 0.0);
    /// (0, -1) — screen-space down (Y-down convention)
    pub const UP: Vec2 = Vec2::new(0.0, -1.0);
    /// (0, 1) — screen-space down (Y-down convention)
    pub const DOWN: Vec2 = Vec2::new(0.0, 1.0);
    /// (1, 1) normalized
    pub const ONE: Vec2 = Vec2::new(1.0, 1.0);

    /// Useful approximation of π.
    pub const PI: f32 = std::f32::consts::PI;
    /// π / 2
    pub const HALF_PI: f32 = PI / 2.0;
    /// 2π
    pub const TAU: f32 = PI * 2.0;
}

/// Extension trait for `Vec2` with game-oriented helpers.
pub trait Vec2Ext {
    /// Euclidean distance to another point.
    fn distance_to(self, other: Vec2) -> f32;
    /// Squared Euclidean distance (avoids sqrt).
    fn distance_squared_to(self, other: Vec2) -> f32;
    /// Angle from `self` to `other` in radians.
    fn angle_to(self, other: Vec2) -> f32;
    /// Rotate this vector by `angle` radians.
    fn rotated(self, angle: f32) -> Vec2;
    /// Move toward `target` by at most `max_delta`.
    fn move_toward(self, target: Vec2, max_delta: f32) -> Vec2;
    /// Reflect this vector off a surface with the given `normal`.
    fn reflect(self, normal: Vec2) -> Vec2;
    /// Project `self` onto `onto`.
    fn projected_onto(self, onto: Vec2) -> Vec2;
    /// Perpendicular vector (rotated 90° clockwise in Y-down).
    fn perp(self) -> Vec2;
    /// Convert to integer pixel coordinates.
    fn to_ivec(self) -> glam::IVec2;
}

impl Vec2Ext for Vec2 {
    #[inline]
    fn distance_to(self, other: Vec2) -> f32 {
        (self - other).length()
    }

    #[inline]
    fn distance_squared_to(self, other: Vec2) -> f32 {
        (self - other).length_squared()
    }

    #[inline]
    fn angle_to(self, other: Vec2) -> f32 {
        (other - self).angle_from(Vec2::X)
    }

    #[inline]
    fn rotated(self, angle: f32) -> Vec2 {
        let (sin, cos) = angle.sin_cos();
        Vec2::new(
            self.x * cos - self.y * sin,
            self.x * sin + self.y * cos,
        )
    }

    #[inline]
    fn move_toward(self, target: Vec2, max_delta: f32) -> Vec2 {
        let diff = target - self;
        let dist = diff.length();
        if dist <= max_delta || dist < 1e-6 {
            target
        } else {
            self + diff * (max_delta / dist)
        }
    }

    #[inline]
    fn reflect(self, normal: Vec2) -> Vec2 {
        self - 2.0 * self.dot(normal) * normal
    }

    #[inline]
    fn projected_onto(self, onto: Vec2) -> Vec2 {
        onto * (self.dot(onto) / onto.length_squared())
    }

    #[inline]
    fn perp(self) -> Vec2 {
        Vec2::new(self.y, -self.x)
    }

    #[inline]
    fn to_ivec(self) -> glam::IVec2 {
        glam::IVec2::new(self.x as i32, self.y as i32)
    }
}

/// Extension trait for `f32` with game-oriented helpers.
pub trait FloatExt {
    /// Linear interpolation between `self` and `other` by `t` (clamped 0..1).
    fn lerp(self, other: f32, t: f32) -> f32;
    /// Clamp value between `min` and `max`.
    fn clamp(self, min: f32, max: f32) -> f32;
    /// Map value from `in_min..=in_max` to `out_min..=out_max`.
    fn map_range(self, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32;
    /// Deadzone filter — returns 0.0 if within `deadzone` of zero.
    fn deadzone(self, deadzone: f32) -> f32;
    /// Snap to nearest increment of `step`.
    fn snap(self, step: f32) -> f32;
}

impl FloatExt for f32 {
    #[inline]
    fn lerp(self, other: f32, t: f32) -> f32 {
        self + (other - self) * t.clamp(0.0, 1.0)
    }

    #[inline]
    fn clamp(self, min: f32, max: f32) -> f32 {
        Self::clamp(self, min, max)
    }

    #[inline]
    fn map_range(self, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
        (self - in_min) / (in_max - in_min) * (out_max - out_min) + out_min
    }

    #[inline]
    fn deadzone(self, deadzone: f32) -> f32 {
        if self.abs() < deadzone { 0.0 } else { self }
    }

    #[inline]
    fn snap(self, step: f32) -> f32 {
        (self / step).round() * step
    }
}