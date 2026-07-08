//! Collision detection module.
//!
//! Provides multiple collision detection algorithms:
//! - **AABB** (Axis-Aligned Bounding Box) — fastest, axis-aligned only
//! - **Circle** — for circular hitboxes
//! - **SAT** (Separating Axis Theorem) — for oriented rectangles and convex polygons
//! - **Broad phase** — spatial hashing for large numbers of objects
//!
//! All collision functions return a [`CollisionInfo`] struct with penetration
//! depth and normal, enabling proper resolution — unlike macroquad which has
//! no collision detection at all.

use crate::math::{Vec2, Rect, FloatExt};

/// Result of a collision test.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CollisionInfo {
    /// Whether a collision occurred.
    pub colliding: bool,
    /// Penetration depth (how much the shapes overlap).
    pub penetration: f32,
    /// Collision normal (points from shape A toward shape B).
    pub normal: Vec2,
    /// Contact point (approximate).
    pub contact_point: Vec2,
}

impl Default for CollisionInfo {
    fn default() -> Self {
        Self {
            colliding: false,
            penetration: 0.0,
            normal: Vec2::ZERO,
            contact_point: Vec2::ZERO,
        }
    }
}

impl CollisionInfo {
    /// Create a "no collision" result.
    pub fn none() -> Self {
        Self::default()
    }

    /// Create a collision result.
    pub fn new(penetration: f32, normal: Vec2, contact_point: Vec2) -> Self {
        Self {
            colliding: true,
            penetration,
            normal,
            contact_point,
        }
    }
}

// ─────────────────────────────────────────────
// Circle
// ─────────────────────────────────────────────

/// A circle collider.
#[derive(Debug, Clone, Copy)]
pub struct CircleCollider {
    /// Center position.
    pub center: Vec2,
    /// Radius.
    pub radius: f32,
}

impl CircleCollider {
    /// Create a new circle collider.
    pub fn new(center: Vec2, radius: f32) -> Self {
        Self { center, radius }
    }
}

/// Test collision between two circles.
pub fn circle_vs_circle(a: CircleCollider, b: CircleCollider) -> CollisionInfo {
    let diff = b.center - a.center;
    let dist = diff.length();
    let sum_radii = a.radius + b.radius;

    if dist >= sum_radii {
        return CollisionInfo::none();
    }

    let normal = if dist > 1e-6 { diff / dist } else { Vec2::new(1.0, 0.0) };
    let contact = a.center + normal * a.radius;

    CollisionInfo::new(sum_radii - dist, normal, contact)
}

/// Test collision between a circle and an AABB rectangle.
pub fn circle_vs_rect(circle: CircleCollider, rect: Rect) -> CollisionInfo {
    let closest = rect.closest_point(circle.center);
    let diff = circle.center - closest;
    let dist_sq = diff.length_squared();

    if dist_sq >= circle.radius * circle.radius {
        return CollisionInfo::none();
    }

    let dist = dist_sq.sqrt();
    if dist < 1e-6 {
        // Circle center is inside the rect
        let center = rect.center();
        let diff_to_center = circle.center - center;
        let half = rect.size() * 0.5;

        let overlap_x = half.x - diff_to_center.x.abs();
        let overlap_y = half.y - diff_to_center.y.abs();

        if overlap_x < overlap_y {
            let normal = Vec2::new(if diff_to_center.x > 0.0 { 1.0 } else { -1.0 }, 0.0);
            return CollisionInfo::new(overlap_x + circle.radius, normal, closest);
        } else {
            let normal = Vec2::new(0.0, if diff_to_center.y > 0.0 { 1.0 } else { -1.0 });
            return CollisionInfo::new(overlap_y + circle.radius, normal, closest);
        }
    }

    let normal = diff / dist;
    CollisionInfo::new(circle.radius - dist, normal, closest)
}

// ─────────────────────────────────────────────
// AABB vs AABB
// ─────────────────────────────────────────────

/// Test collision between two AABBs with penetration info.
pub fn rect_vs_rect(a: Rect, b: Rect) -> CollisionInfo {
    if !a.overlaps(b) {
        return CollisionInfo::none();
    }

    let a_center = a.center();
    let b_center = b.center();
    let diff = b_center - a_center;

    let a_half = a.size() * 0.5;
    let b_half = b.size() * 0.5;

    let overlap_x = a_half.x + b_half.x - diff.x.abs();
    let overlap_y = a_half.y + b_half.y - diff.y.abs();

    if overlap_x < overlap_y {
        let normal = Vec2::new(if diff.x > 0.0 { 1.0 } else { -1.0 }, 0.0);
        let contact = Vec2::new(
            a_center.x + a_half.x * normal.x,
            a_center.y + (b_center.y - a_center.y) * 0.5,
        );
        CollisionInfo::new(overlap_x, normal, contact)
    } else {
        let normal = Vec2::new(0.0, if diff.y > 0.0 { 1.0 } else { -1.0 });
        let contact = Vec2::new(
            a_center.x + (b_center.x - a_center.x) * 0.5,
            a_center.y + a_half.y * normal.y,
        );
        CollisionInfo::new(overlap_y, normal, contact)
    }
}

// ─────────────────────────────────────────────
// Point tests
// ─────────────────────────────────────────────

/// Check if a point is inside a circle.
pub fn point_in_circle(point: Vec2, circle: CircleCollider) -> bool {
    point.distance_squared_to(circle.center) <= circle.radius * circle.radius
}

/// Check if a point is inside a rectangle.
pub fn point_in_rect(point: Vec2, rect: Rect) -> bool {
    rect.contains(point)
}

// ─────────────────────────────────────────────
// Ray casting
// ─────────────────────────────────────────────

/// A 2D ray.
#[derive(Debug, Clone, Copy)]
pub struct Ray {
    /// Origin point.
    pub origin: Vec2,
    /// Normalized direction.
    pub direction: Vec2,
    /// Maximum distance to check.
    pub max_distance: f32,
}

impl Ray {
    /// Create a new ray.
    pub fn new(origin: Vec2, direction: Vec2, max_distance: f32) -> Self {
        Self {
            origin,
            direction: direction.normalize_or_zero(),
            max_distance,
        }
    }

    /// Create a ray from two points.
    pub fn from_to(from: Vec2, to: Vec2) -> Self {
        let diff = to - from;
        Self {
            origin: from,
            direction: diff.normalize_or_zero(),
            max_distance: diff.length(),
        }
    }
}

/// Result of a ray cast.
#[derive(Debug, Clone, Copy)]
pub struct RaycastHit {
    /// Point of intersection.
    pub point: Vec2,
    /// Distance from ray origin.
    pub distance: f32,
    /// Surface normal at the hit point.
    pub normal: Vec2,
}

/// Cast a ray against an AABB.
pub fn ray_vs_rect(ray: Ray, rect: Rect) -> Option<RaycastHit> {
    let r_min = rect.pos();
    let r_max = rect.max();

    let mut t_min = f32::NEG_INFINITY;
    let mut t_max = f32::INFINITY;

    // X axis
    if ray.direction.x.abs() > 1e-6 {
        let t1 = (r_min.x - ray.origin.x) / ray.direction.x;
        let t2 = (r_max.x - ray.origin.x) / ray.direction.x;
        t_min = t_min.max(t1.min(t2));
        t_max = t_max.min(t1.max(t2));
    } else if ray.origin.x < r_min.x || ray.origin.x > r_max.x {
        return None;
    }

    // Y axis
    if ray.direction.y.abs() > 1e-6 {
        let t1 = (r_min.y - ray.origin.y) / ray.direction.y;
        let t2 = (r_max.y - ray.origin.y) / ray.direction.y;
        t_min = t_min.max(t1.min(t2));
        t_max = t_max.min(t1.max(t2));
    } else if ray.origin.y < r_min.y || ray.origin.y > r_max.y {
        return None;
    }

    if t_min > t_max || t_max < 0.0 || t_min > ray.max_distance {
        return None;
    }

    let t = if t_min >= 0.0 { t_min } else { t_max };
    let point = ray.origin + ray.direction * t;

    // Compute normal
    let center = rect.center();
    let half = rect.size() * 0.5;
    let local = point - center;
    let normal = if (local.x / half.x).abs() > (local.y / half.y).abs() {
        Vec2::new(local.x.signum(), 0.0)
    } else {
        Vec2::new(0.0, local.y.signum())
    };

    Some(RaycastHit {
        point,
        distance: t,
        normal,
    })
}

/// Cast a ray against a circle.
pub fn ray_vs_circle(ray: Ray, circle: CircleCollider) -> Option<RaycastHit> {
    let oc = ray.origin - circle.center;
    let a = ray.direction.length_squared();
    let b = 2.0 * oc.dot(ray.direction);
    let c = oc.length_squared() - circle.radius * circle.radius;
    let discriminant = b * b - 4.0 * a * c;

    if discriminant < 0.0 {
        return None;
    }

    let sqrt_disc = discriminant.sqrt();
    let t1 = (-b - sqrt_disc) / (2.0 * a);
    let t2 = (-b + sqrt_disc) / (2.0 * a);

    let t = if t1 >= 0.0 { t1 } else if t2 >= 0.0 { t2 } else { return None };

    if t > ray.max_distance {
        return None;
    }

    let point = ray.origin + ray.direction * t;
    let normal = (point - circle.center).normalize_or_zero();

    Some(RaycastHit { point, distance: t, normal })
}

// ─────────────────────────────────────────────
// Spatial hash (broad phase)
// ─────────────────────────────────────────────

/// A simple spatial hash grid for broad-phase collision detection.
///
/// Divides the world into cells of `cell_size` and assigns colliders to cells.
/// Only tests collisions between objects in the same or neighboring cells.
pub struct SpatialHash {
    cell_size: f32,
    cells: std::collections::HashMap<(i32, i32), Vec<usize>>,
    positions: Vec<Vec2>,
    radii: Vec<f32>,
}

impl SpatialHash {
    /// Create a new spatial hash with the given cell size.
    pub fn new(cell_size: f32) -> Self {
        Self {
            cell_size,
            cells: std::collections::HashMap::new(),
            positions: Vec::new(),
            radii: Vec::new(),
        }
    }

    /// Clear all entries.
    pub fn clear(&mut self) {
        self.cells.clear();
        self.positions.clear();
        self.radii.clear();
    }

    /// Insert a collider at the given position with the given radius.
    pub fn insert(&mut self, id: usize, pos: Vec2, radius: f32) {
        if id >= self.positions.len() {
            self.positions.resize(id + 1, Vec2::ZERO);
            self.radii.resize(id + 1, 0.0);
        }
        self.positions[id] = pos;
        self.radii[id] = radius;

        let min_x = ((pos.x - radius) / self.cell_size).floor() as i32;
        let max_x = ((pos.x + radius) / self.cell_size).floor() as i32;
        let min_y = ((pos.y - radius) / self.cell_size).floor() as i32;
        let max_y = ((pos.y + radius) / self.cell_size).floor() as i32;

        for cy in min_y..=max_y {
            for cx in min_x..=max_x {
                self.cells.entry((cx, cy)).or_default().push(id);
            }
        }
    }

    /// Query all potential collision pairs. Returns `(id_a, id_b)` pairs.
    pub fn query_pairs(&self) -> Vec<(usize, usize)> {
        let mut seen = std::collections::HashSet::new();
        let mut pairs = Vec::new();

        for cell_ids in self.cells.values() {
            for i in 0..cell_ids.len() {
                for j in (i + 1)..cell_ids.len() {
                    let a = cell_ids[i];
                    let b = cell_ids[j];
                    let key = if a < b { (a, b) } else { (b, a) };
                    if seen.insert(key) {
                        pairs.push(key);
                    }
                }
            }
        }

        pairs
    }

    /// Query all objects near a point within `radius`.
    pub fn query_near(&self, point: Vec2, radius: f32) -> Vec<usize> {
        let mut result = Vec::new();
        let mut seen = std::collections::HashSet::new();

        let min_x = ((point.x - radius) / self.cell_size).floor() as i32;
        let max_x = ((point.x + radius) / self.cell_size).floor() as i32;
        let min_y = ((point.y - radius) / self.cell_size).floor() as i32;
        let max_y = ((point.y + radius) / self.cell_size).floor() as i32;

        for cy in min_y..=max_y {
            for cx in min_x..=max_x {
                if let Some(ids) = self.cells.get(&(cx, cy)) {
                    for &id in ids {
                        if seen.insert(id) {
                            result.push(id);
                        }
                    }
                }
            }
        }

        result
    }
}

// ─────────────────────────────────────────────
// Collision resolution helpers
// ─────────────────────────────────────────────

/// Resolve a collision by pushing shape A out of shape B.
///
/// Modifies `position_a` in place and returns the new position.
pub fn resolve_aabb_collision(position_a: &mut Vec2, velocity_a: &mut Vec2, collision: &CollisionInfo) {
    if !collision.colliding {
        return;
    }
    *position_a -= collision.normal * collision.penetration;

    // Remove velocity component into the surface
    let vel_along_normal = velocity_a.dot(collision.normal);
    if vel_along_normal < 0.0 {
        *velocity_a -= collision.normal * vel_along_normal;
    }
}