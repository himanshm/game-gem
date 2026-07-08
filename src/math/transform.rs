//! 2D Transform with hierarchy support.

use super::{Vec2, Angle};

/// A 2D transform representing position, rotation, and scale.
///
/// Supports parent-child hierarchies so that child transforms inherit
/// their parent's world-space transform.
#[derive(Clone, Debug)]
pub struct Transform {
    /// Local position relative to parent.
    pub position: Vec2,
    /// Local rotation in radians.
    pub rotation: Angle,
    /// Local scale (independent X/Y).
    pub scale: Vec2,
    /// Optional parent reference index (used by [`SceneNode`]).
    pub parent: Option<usize>,
    /// Cached world-space matrix (invalidated when local values change).
    world_matrix: glam::Mat4,
    /// Whether the world matrix needs recomputation.
    dirty: bool,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vec2::ZERO,
            rotation: Angle::zero(),
            scale: Vec2::ONE,
            parent: None,
            world_matrix: glam::Mat4::IDENTITY,
            dirty: false,
        }
    }
}

impl Transform {
    /// Create a transform with the given position.
    pub fn at(x: f32, y: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            ..Default::default()
        }
    }

    /// Create a transform with position, rotation (degrees), and uniform scale.
    pub fn new(x: f32, y: f32, rotation_degrees: f32, scale: f32) -> Self {
        Self {
            position: Vec2::new(x, y),
            rotation: Angle::from_degrees(rotation_degrees),
            scale: Vec2::splat(scale),
            ..Default::default()
        }
    }

    /// Set position and mark dirty.
    pub fn set_position(&mut self, pos: Vec2) {
        self.position = pos;
        self.dirty = true;
    }

    /// Set rotation in degrees and mark dirty.
    pub fn set_rotation_degrees(&mut self, degrees: f32) {
        self.rotation = Angle::from_degrees(degrees);
        self.dirty = true;
    }

    /// Set uniform scale and mark dirty.
    pub fn set_scale(&mut self, s: f32) {
        self.scale = Vec2::splat(s);
        self.dirty = true;
    }

    /// Set non-uniform scale and mark dirty.
    pub fn set_scale_vec(&mut self, s: Vec2) {
        self.scale = s;
        self.dirty = true;
    }

    /// Translate by an offset.
    pub fn translate(&mut self, offset: Vec2) {
        self.position += offset;
        self.dirty = true;
    }

    /// Rotate by an angle (radians).
    pub fn rotate(&mut self, angle: Angle) {
        self.rotation += angle;
        self.dirty = true;
    }

    /// Get the local-space 4×4 matrix (TR × S).
    pub fn local_matrix(&self) -> glam::Mat4 {
        let translation = glam::Mat4::from_translation(glam::Vec3::new(
            self.position.x,
            self.position.y,
            0.0,
        ));
        let rotation = glam::Mat4::from_rotation_z(self.rotation.as_radians());
        let scale = glam::Mat4::from_scale(glam::Vec3::new(self.scale.x, self.scale.y, 1.0));
        translation * rotation * scale
    }

    /// Get the world-space matrix, recomputing if dirty.
    ///
    /// If a parent index is set, the parent's world matrix should be passed.
    pub fn world_matrix(&mut self, parent_world: Option<glam::Mat4>) -> glam::Mat4 {
        if self.dirty || parent_world.is_some() {
            let local = self.local_matrix();
            self.world_matrix = match parent_world {
                Some(pw) => pw * local,
                None => local,
            };
            self.dirty = false;
        }
        self.world_matrix
    }

    /// Invalidate the world matrix, forcing recomputation next frame.
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Extract the world-space position from the world matrix.
    pub fn world_position(&self) -> Vec2 {
        Vec2::new(self.world_matrix.w_axis.x, self.world_matrix.w_axis.y)
    }

    /// Transform a local point to world space.
    pub fn transform_point(&self, local_point: Vec2) -> Vec2 {
        let transformed = self.world_matrix
            * glam::Vec4::new(local_point.x, local_point.y, 0.0, 1.0);
        Vec2::new(transformed.x, transformed.y)
    }

    /// Inverse-transform a world point to local space.
    pub fn inverse_transform_point(&self, world_point: Vec2) -> Vec2 {
        if let Some(inv) = self.world_matrix.inverse() {
            let transformed = inv * glam::Vec4::new(world_point.x, world_point.y, 0.0, 1.0);
            Vec2::new(transformed.x, transformed.y)
        } else {
            world_point // Degenerate transform, return as-is
        }
    }

    /// Linearly interpolate between two transforms.
    pub fn lerp_to(&self, other: &Transform, t: f32) -> Transform {
        use super::FloatExt;
        Transform {
            position: self.position.lerp(other.position, t),
            rotation: Angle::from_radians(
                self.rotation.as_radians().lerp(other.rotation.as_radians(), t),
            ),
            scale: Vec2::new(
                self.scale.x.lerp(other.scale.x, t),
                self.scale.y.lerp(other.scale.y, t),
            ),
            parent: self.parent,
            world_matrix: glam::Mat4::IDENTITY,
            dirty: true,
        }
    }

    /// Look at a target point (sets rotation to face the target).
    pub fn look_at(&mut self, target: Vec2) {
        let diff = target - self.position;
        self.rotation = Angle::from_radians(diff.y.atan2(diff.x));
        self.dirty = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_transform() {
        let t = Transform::default();
        assert_eq!(t.position, Vec2::ZERO);
        assert_eq!(t.rotation.as_radians(), 0.0);
        assert_eq!(t.scale, Vec2::ONE);
    }

    #[test]
    fn test_local_matrix_identity() {
        let t = Transform::default();
        let m = t.local_matrix();
        assert!((m - glam::Mat4::IDENTITY).abs_diff_eq(glam::Mat4::IDENTITY, 1e-6));
    }

    #[test]
    fn test_translate() {
        let mut t = Transform::default();
        t.translate(Vec2::new(10.0, 20.0));
        assert_eq!(t.position, Vec2::new(10.0, 20.0));
    }
}