//! Axis-aligned rectangle with game-oriented operations.

use super::Vec2;

/// An axis-aligned rectangle defined by its top-left corner, width, and height.
///
/// Uses the screen-space convention: Y increases downward.
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Rect {
    /// X coordinate of the top-left corner.
    pub x: f32,
    /// Y coordinate of the top-left corner.
    pub y: f32,
    /// Width of the rectangle.
    pub w: f32,
    /// Height of the rectangle.
    pub h: f32,
}

impl Rect {
    /// Create a new rectangle from position and size.
    #[inline]
    pub const fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        Self { x, y, w, h }
    }

    /// Create a rectangle from min and max corner points.
    #[inline]
    pub const fn from_min_max(min: Vec2, max: Vec2) -> Self {
        Self {
            x: min.x,
            y: min.y,
            w: max.x - min.x,
            h: max.y - min.y,
        }
    }

    /// Create a rectangle centered at `center` with the given size.
    #[inline]
    pub const fn centered(center: Vec2, size: Vec2) -> Self {
        Self {
            x: center.x - size.x * 0.5,
            y: center.y - size.y * 0.5,
            w: size.x,
            h: size.y,
        }
    }

    /// Top-left corner position.
    #[inline]
    pub fn pos(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    /// Size of the rectangle.
    #[inline]
    pub fn size(self) -> Vec2 {
        Vec2::new(self.w, self.h)
    }

    /// Center point of the rectangle.
    #[inline]
    pub fn center(self) -> Vec2 {
        Vec2::new(self.x + self.w * 0.5, self.y + self.h * 0.5)
    }

    /// Bottom-right corner (exclusive).
    #[inline]
    pub fn max(self) -> Vec2 {
        Vec2::new(self.x + self.w, self.y + self.h)
    }

    /// Top-right corner.
    #[inline]
    pub fn top_right(self) -> Vec2 {
        Vec2::new(self.x + self.w, self.y)
    }

    /// Bottom-left corner.
    #[inline]
    pub fn bottom_left(self) -> Vec2 {
        Vec2::new(self.x, self.y + self.h)
    }

    /// Check if the rectangle contains a point.
    #[inline]
    pub fn contains(self, point: Vec2) -> bool {
        point.x >= self.x
            && point.x <= self.x + self.w
            && point.y >= self.y
            && point.y <= self.y + self.h
    }

    /// Check if this rectangle fully contains another.
    #[inline]
    pub fn contains_rect(self, other: Rect) -> bool {
        let self_max = self.max();
        let other_max = other.max();
        other.x >= self.x
            && other.y >= self.y
            && other_max.x <= self_max.x
            && other_max.y <= self_max.y
    }

    /// Check if two rectangles overlap (AABB intersection test).
    #[inline]
    pub fn overlaps(self, other: Rect) -> bool {
        self.x < other.x + other.w
            && self.x + self.w > other.x
            && self.y < other.y + other.h
            && self.y + self.h > other.y
    }

    /// Compute the intersection rectangle, or `None` if they don't overlap.
    #[inline]
    pub fn intersection(self, other: Rect) -> Option<Rect> {
        if !self.overlaps(other) {
            return None;
        }
        let x1 = self.x.max(other.x);
        let y1 = self.y.max(other.y);
        let x2 = (self.x + self.w).min(other.x + other.w);
        let y2 = (self.y + self.h).min(other.y + other.h);
        Some(Rect::new(x1, y1, x2 - x1, y2 - y1))
    }

    /// Compute the union rectangle that covers both.
    #[inline]
    pub fn union(self, other: Rect) -> Rect {
        let x1 = self.x.min(other.x);
        let y1 = self.y.min(other.y);
        let x2 = (self.x + self.w).max(other.x + other.w);
        let y2 = (self.y + self.h).max(other.y + other.h);
        Rect::new(x1, y1, x2 - x1, y2 - y1)
    }

    /// Inflate the rectangle by `amount` on all sides.
    #[inline]
    pub fn inflated(self, amount: f32) -> Rect {
        Rect::new(
            self.x - amount,
            self.y - amount,
            self.w + amount * 2.0,
            self.h + amount * 2.0,
        )
    }

    /// Shrink the rectangle by `amount` on all sides.
    #[inline]
    pub fn deflated(self, amount: f32) -> Rect {
        self.inflated(-amount)
    }

    /// Scale the rectangle around its center.
    #[inline]
    pub fn scaled(self, scale: Vec2) -> Rect {
        let new_size = self.size() * scale;
        let offset = (new_size - self.size()) * 0.5;
        Rect::new(self.x - offset.x, self.y - offset.y, new_size.x, new_size.y)
    }

    /// Move the rectangle so its center is at `center`.
    #[inline]
    pub fn with_center(self, center: Vec2) -> Rect {
        Rect::centered(center, self.size())
    }

    /// Translate the rectangle by an offset.
    #[inline]
    pub fn translated(self, offset: Vec2) -> Rect {
        Rect::new(self.x + offset.x, self.y + offset.y, self.w, self.h)
    }

    /// Closest point inside the rectangle to the given `point`.
    #[inline]
    pub fn closest_point(self, point: Vec2) -> Vec2 {
        Vec2::new(
            point.x.clamp(self.x, self.x + self.w),
            point.y.clamp(self.y, self.y + self.h),
        )
    }

    /// Area of the rectangle.
    #[inline]
    pub fn area(self) -> f32 {
        self.w * self.h
    }

    /// Perimeter of the rectangle.
    #[inline]
    pub fn perimeter(self) -> f32 {
        2.0 * (self.w + self.h)
    }

    /// Aspect ratio (width / height).
    #[inline]
    pub fn aspect_ratio(self) -> f32 {
        self.w / self.h
    }
}

impl std::fmt::Display for Rect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Rect({}, {}, {}, {})", self.x, self.y, self.w, self.h)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for Rect {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut s = serializer.serialize_struct("Rect", 4)?;
        s.serialize_field("x", &self.x)?;
        s.serialize_field("y", &self.y)?;
        s.serialize_field("w", &self.w)?;
        s.serialize_field("h", &self.h)?;
        s.end()
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Rect {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct RectHelper { x: f32, y: f32, w: f32, h: f32 }
        let h = RectHelper::deserialize(deserializer)?;
        Ok(Rect::new(h.x, h.y, h.w, h.h))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_point() {
        let r = Rect::new(10.0, 10.0, 100.0, 50.0);
        assert!(r.contains(Vec2::new(50.0, 30.0)));
        assert!(r.contains(Vec2::new(10.0, 10.0)));
        assert!(r.contains(Vec2::new(110.0, 60.0)));
        assert!(!r.contains(Vec2::new(5.0, 30.0)));
        assert!(!r.contains(Vec2::new(50.0, 65.0)));
    }

    #[test]
    fn test_overlaps() {
        let a = Rect::new(0.0, 0.0, 100.0, 100.0);
        let b = Rect::new(50.0, 50.0, 100.0, 100.0);
        let c = Rect::new(200.0, 200.0, 50.0, 50.0);
        assert!(a.overlaps(b));
        assert!(!a.overlaps(c));
    }

    #[test]
    fn test_intersection() {
        let a = Rect::new(0.0, 0.0, 100.0, 100.0);
        let b = Rect::new(50.0, 50.0, 100.0, 100.0);
        let i = a.intersection(b).unwrap();
        assert_eq!(i, Rect::new(50.0, 50.0, 50.0, 50.0));
    }
}