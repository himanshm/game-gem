//! Generic linear interpolation trait.

/// Types that can be linearly interpolated.
pub trait Lerp: Sized {
    /// Linear interpolation from `self` toward `other` by factor `t` (clamped 0..1).
    fn lerp(self, other: Self, t: f32) -> Self;
}

impl Lerp for f32 {
    #[inline]
    fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t.clamp(0.0, 1.0)
    }
}

impl Lerp for super::Vec2 {
    #[inline]
    fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t.clamp(0.0, 1.0)
    }
}

impl Lerp for super::Vec3 {
    #[inline]
    fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t.clamp(0.0, 1.0)
    }
}

impl Lerp for super::Vec4 {
    #[inline]
    fn lerp(self, other: Self, t: f32) -> Self {
        self + (other - self) * t.clamp(0.0, 1.0)
    }
}