//! GPU-independent rays, axis-aligned bounds and conservative frustum queries.
//!
//! Constructors validate finite data. Queries use metres with a right-handed
//! +Y-up world; frusta use the renderer's DirectX/wgpu depth range of 0..1.
//!
//! ```
//! use kvas_engine::spatial::{Aabb, Ray};
//! use glam::Vec3;
//! let bounds = Aabb::new(-Vec3::ONE, Vec3::ONE).unwrap();
//! let ray = Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::NEG_Z).unwrap();
//! assert_eq!(bounds.ray_distance(ray, 10.0), Some(4.0));
//! assert!(bounds.contains(Vec3::ZERO));
//! ```

use glam::{Mat4, Vec3, Vec4};

/// Finite world/local ray with a unit direction.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    /// Normalizes a nonzero direction; invalid data returns `None`.
    pub fn new(origin: Vec3, direction: Vec3) -> Option<Self> {
        if !origin.is_finite() || !direction.is_finite() {
            return None;
        }
        // Rescale before length calculation to avoid overflow/underflow for
        // finite directions far outside ordinary world-coordinate magnitudes.
        let largest = direction.abs().max_element();
        if largest == 0.0 { return None; }
        Some(Self { origin, direction: (direction / largest).normalize() })
    }
    /// Ray start point.
    pub fn origin(self) -> Vec3 {
        self.origin
    }
    /// Unit direction along increasing distance.
    pub fn direction(self) -> Vec3 {
        self.direction
    }
    /// Point at a signed distance from the origin; negative distances go backward.
    pub fn point_at(self, distance: f32) -> Vec3 {
        self.origin + self.direction * distance
    }
    /// Transforms the origin and direction, then renormalizes the direction.
    /// Only affine matrices are accepted; singular transforms may collapse the ray.
    pub fn transformed(self, matrix: Mat4) -> Option<Self> {
        if !is_affine(matrix) {
            return None;
        }
        Self::new(matrix.transform_point3(self.origin), matrix.transform_vector3(self.direction))
    }
    /// Forward distance to a plane represented by a point and any nonzero normal.
    /// Parallel, backward, non-finite or degenerate intersections return `None`.
    pub fn plane_distance(self, point: Vec3, normal: Vec3) -> Option<f32> {
        if !point.is_finite() || !normal.is_finite() {
            return None;
        }
        let normal = normal.try_normalize()?;
        let denominator = self.direction.dot(normal);
        if denominator.abs() <= f32::EPSILON {
            return None;
        }
        let distance = (point - self.origin).dot(normal) / denominator;
        (distance.is_finite() && distance >= 0.0).then_some(distance)
    }

    /// First forward intersection with a finite positive-radius sphere.
    /// Origins inside/on the sphere return zero, matching AABB query semantics.
    /// Entirely backward intersections return `None` instead of a false zero hit.
    pub fn sphere_distance(self, center: Vec3, radius: f32) -> Option<f32> {
        if !center.is_finite() || !radius.is_finite() || radius <= 0.0 { return None; }
        let offset = self.origin.as_dvec3() - center.as_dvec3();
        let direction = self.direction.as_dvec3();
        let a = direction.length_squared();
        let b = offset.dot(direction);
        let c = offset.length_squared() - (radius as f64).powi(2);
        if c <= 0.0 { return Some(0.0); }
        let discriminant = b * b - a * c;
        if discriminant < 0.0 { return None; }
        let distance = (-b - discriminant.sqrt()) / a;
        (distance >= 0.0 && distance <= f32::MAX as f64).then_some(distance as f32)
    }
}

/// Finite axis-aligned closed box. Degenerate extents (planes/points) are allowed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    min: Vec3,
    max: Vec3,
}

impl Aabb {
    /// Creates ordered bounds; inverted or non-finite extents return `None`.
    pub fn new(min: Vec3, max: Vec3) -> Option<Self> {
        (min.is_finite() && max.is_finite() && min.cmple(max).all()).then_some(Self { min, max })
    }
    /// Builds bounds around at least one finite point; any invalid point rejects the set.
    pub fn from_points(points: impl IntoIterator<Item = Vec3>) -> Option<Self> {
        let mut points = points.into_iter();
        let first = points.next()?;
        let mut bounds = Self::new(first, first)?;
        for point in points {
            if !point.is_finite() {
                return None;
            }
            bounds.min = bounds.min.min(point);
            bounds.max = bounds.max.max(point);
        }
        Some(bounds)
    }
    /// Minimum corner.
    pub fn min(self) -> Vec3 {
        self.min
    }
    /// Maximum corner.
    pub fn max(self) -> Vec3 {
        self.max
    }
    /// Midpoint, computed without summing potentially large same-sign endpoints.
    pub fn center(self) -> Vec3 {
        self.min * 0.5 + self.max * 0.5
    }
    /// Full per-axis extents.
    pub fn size(self) -> Vec3 {
        self.max - self.min
    }
    /// Half of the per-axis extents.
    pub fn half_extents(self) -> Vec3 {
        self.max * 0.5 - self.min * 0.5
    }
    /// Whether a finite point lies inside or on the boundary.
    pub fn contains(self, point: Vec3) -> bool {
        point.is_finite() && point.cmpge(self.min).all() && point.cmple(self.max).all()
    }
    /// Whether two closed boxes overlap or touch.
    pub fn intersects(self, other: Self) -> bool {
        self.min.cmple(other.max).all() && self.max.cmpge(other.min).all()
    }
    /// Smallest axis-aligned box containing both inputs.
    pub fn union(self, other: Self) -> Self {
        Self { min: self.min.min(other.min), max: self.max.max(other.max) }
    }
    /// Closest point in/on the box; the caller must supply a finite point.
    pub fn closest_point(self, point: Vec3) -> Vec3 {
        point.clamp(self.min, self.max)
    }
    /// Expands every face by a non-negative finite margin; overflow returns `None`.
    pub fn expanded(self, margin: f32) -> Option<Self> {
        if !margin.is_finite() || margin < 0.0 {
            return None;
        }
        Self::new(self.min - Vec3::splat(margin), self.max + Vec3::splat(margin))
    }
    /// Eight corners in bit order: bit 0 selects max X, bit 1 max Y, bit 2 max Z.
    pub fn corners(self) -> [Vec3; 8] {
        std::array::from_fn(|i| {
            Vec3::new(
                if i & 1 == 0 { self.min.x } else { self.max.x },
                if i & 2 == 0 { self.min.y } else { self.max.y },
                if i & 4 == 0 { self.min.z } else { self.max.z },
            )
        })
    }
    /// Axis-aligned envelope after a finite affine transformation, including mirrors/shear.
    pub fn transformed(self, matrix: Mat4) -> Option<Self> {
        if !is_affine(matrix) {
            return None;
        }
        Self::from_points(self.corners().map(|point| matrix.transform_point3(point)))
    }
    /// First intersection distance within a finite non-negative range.
    /// A ray beginning inside/on the box returns zero. Parallel axes are handled
    /// without reciprocal infinities, including origins exactly on a slab face.
    pub fn ray_distance(self, ray: Ray, max_distance: f32) -> Option<f32> {
        if !max_distance.is_finite() || max_distance < 0.0 {
            return None;
        }
        let (mut enter, mut exit) = (0.0f32, max_distance);
        for axis in 0..3 {
            let direction = ray.direction[axis];
            let origin = ray.origin[axis];
            if direction == 0.0 {
                if origin < self.min[axis] || origin > self.max[axis] {
                    return None;
                }
            } else {
                let a = (self.min[axis] - origin) / direction;
                let b = (self.max[axis] - origin) / direction;
                enter = enter.max(a.min(b));
                exit = exit.min(a.max(b));
                if enter > exit {
                    return None;
                }
            }
        }
        Some(enter)
    }
}

/// Six inward-facing clip planes for a finite 0..1-depth view/projection matrix.
#[derive(Clone, Copy, Debug)]
pub struct Frustum {
    planes: [Vec4; 6],
}

impl Frustum {
    /// Extracts left/right/bottom/top/near/far planes from a wgpu view-projection matrix.
    /// Invalid or degenerate planes return `None`; infinite-far projections are unsupported.
    pub fn from_view_projection(matrix: Mat4) -> Option<Self> {
        if !matrix.is_finite() {
            return None;
        }
        let (x, y, z, w) = (matrix.row(0), matrix.row(1), matrix.row(2), matrix.row(3));
        let mut planes = [w + x, w - x, w + y, w - y, z, w - z];
        for plane in &mut planes {
            let length = plane.truncate().length();
            if !length.is_finite() || length <= f32::EPSILON {
                return None;
            }
            *plane /= length;
            if !plane.is_finite() {
                return None;
            }
        }
        Some(Self { planes })
    }
    /// Unit-normal planes in left/right/bottom/top/near/far order.
    pub fn planes(&self) -> &[Vec4; 6] {
        &self.planes
    }
    /// Whether a point lies on or inside every clip plane.
    pub fn contains_point(&self, point: Vec3) -> bool {
        point.is_finite() && self.planes.iter().all(|p| p.truncate().dot(point) + p.w >= 0.0)
    }
    /// Conservative world-box visibility test. May retain boxes outside a frustum corner.
    pub fn intersects_aabb(&self, bounds: Aabb) -> bool {
        self.planes.iter().all(|plane| {
            let normal = plane.truncate();
            let positive = Vec3::select(normal.cmpge(Vec3::ZERO), bounds.max, bounds.min);
            normal.dot(positive) + plane.w >= 0.0
        })
    }
    /// Conservative test of a transformed local box without heap allocation.
    /// Non-finite corner positions conservatively return true to avoid hiding geometry.
    pub fn intersects_transformed_aabb(&self, bounds: Aabb, model: Mat4) -> bool {
        let corners = bounds.corners().map(|point| model.transform_point3(point));
        if corners.iter().any(|p| !p.is_finite()) {
            return true;
        }
        self.planes.iter().all(|plane| corners.iter().any(|point| plane.truncate().dot(*point) + plane.w >= 0.0))
    }
}

fn is_affine(matrix: Mat4) -> bool {
    matrix.is_finite() && matrix.row(3).abs_diff_eq(Vec4::W, 1e-6)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn slab_queries_handle_parallel_inside_and_boundary_rays() {
        let bounds = Aabb::new(-Vec3::ONE, Vec3::ONE).unwrap();
        assert_eq!(bounds.ray_distance(Ray::new(Vec3::new(1.0, 0.0, 5.0), Vec3::NEG_Z).unwrap(), 10.0), Some(4.0));
        assert_eq!(bounds.ray_distance(Ray::new(Vec3::new(2.0, 0.0, 5.0), Vec3::NEG_Z).unwrap(), 10.0), None);
        assert_eq!(bounds.ray_distance(Ray::new(Vec3::ZERO, Vec3::X).unwrap(), 0.0), Some(0.0));
        assert_eq!(bounds.ray_distance(Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::NEG_Z).unwrap(), 3.0), None);
        assert!(Ray::new(Vec3::ZERO, Vec3::ZERO).is_none());
    }

    #[test]
    fn sphere_queries_reject_backward_hits_and_direction_normalization_is_stable() {
        let ray = Ray::new(Vec3::ZERO, Vec3::new(0.0, 0.0, f32::MAX)).unwrap();
        assert_eq!(ray.direction(), Vec3::Z);
        assert_eq!(ray.sphere_distance(Vec3::Z * -5.0, 1.0), None);
        assert_eq!(ray.sphere_distance(Vec3::Z * 5.0, 1.0), Some(4.0));
        assert_eq!(ray.sphere_distance(Vec3::ZERO, 1.0), Some(0.0));
        assert_eq!(Ray::new(Vec3::ZERO, Vec3::X * f32::MIN_POSITIVE).unwrap().direction(), Vec3::X);
    }
    #[test]
    fn bounds_transform_and_frustum_use_wgpu_depth() {
        let bounds = Aabb::new(Vec3::ZERO, Vec3::ONE).unwrap();
        let moved =
            bounds.transformed(Mat4::from_scale_rotation_translation(Vec3::new(-2.0, 3.0, 1.0), glam::Quat::IDENTITY, Vec3::X)).unwrap();
        assert_eq!(moved.min(), Vec3::new(-1.0, 0.0, 0.0));
        assert_eq!(moved.max(), Vec3::new(1.0, 3.0, 1.0));
        let frustum = Frustum::from_view_projection(Mat4::IDENTITY).unwrap();
        assert!(frustum.contains_point(Vec3::new(0.0, 0.0, 0.5)));
        assert!(!frustum.contains_point(Vec3::new(0.0, 0.0, -0.1)));
        assert!(frustum.intersects_aabb(bounds));
        assert!(!frustum.intersects_transformed_aabb(bounds, Mat4::from_translation(Vec3::X * 10.0)));
        assert!(Frustum::from_view_projection(Mat4::ZERO).is_none());
        assert!(Aabb::new(Vec3::ONE, Vec3::ZERO).is_none());
        assert!(bounds.expanded(-1.0).is_none());
    }
}
