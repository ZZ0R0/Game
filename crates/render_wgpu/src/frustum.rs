//! Frustum culling system for chunk-based rendering
//!
//! Extracts frustum planes from VP matrix and tests AABBs against them

use glam::{Mat4, Vec3, Vec4};

/// Axis-Aligned Bounding Box
#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Create AABB from chunk position (chunk coordinates)
    pub fn from_chunk_pos(chunk_x: i32, chunk_y: i32, chunk_z: i32, chunk_size: f32) -> Self {
        let min = Vec3::new(
            chunk_x as f32 * chunk_size,
            chunk_y as f32 * chunk_size,
            chunk_z as f32 * chunk_size,
        );
        let max = min + Vec3::splat(chunk_size);
        Self { min, max }
    }

    /// Get center point
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Get half-extents (radius)
    pub fn half_extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }
}

/// A plane in 3D space (ax + by + cz + d = 0)
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub normal: Vec3,  // (a, b, c) - normalized
    pub distance: f32, // d
}

impl Plane {
    pub fn new(normal: Vec3, distance: f32) -> Self {
        Self { normal, distance }
    }

    /// Create plane from Vec4 (x, y, z, w) and normalize
    pub fn from_vec4(v: Vec4) -> Self {
        let length = v.truncate().length();
        Self {
            normal: v.truncate() / length,
            distance: v.w / length,
        }
    }

    /// Distance from plane to point (positive = in front)
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.distance
    }
}

/// View frustum with 6 planes (left, right, bottom, top, near, far)
#[derive(Debug, Clone)]
pub struct Frustum {
    planes: [Plane; 6],
}

impl Frustum {
    /// Extract frustum planes from view-projection matrix
    /// Using Gribb-Hartmann method
    pub fn from_matrix(vp: Mat4) -> Self {
        // Extract rows from matrix
        let row0 = vp.row(0);
        let row1 = vp.row(1);
        let row2 = vp.row(2);
        let row3 = vp.row(3);

        // Extract planes (Gribb-Hartmann method)
        let left = Plane::from_vec4(row3 + row0); // w + x
        let right = Plane::from_vec4(row3 - row0); // w - x
        let bottom = Plane::from_vec4(row3 + row1); // w + y
        let top = Plane::from_vec4(row3 - row1); // w - y
        let near = Plane::from_vec4(row3 + row2); // w + z
        let far = Plane::from_vec4(row3 - row2); // w - z

        Self {
            planes: [left, right, bottom, top, near, far],
        }
    }

    /// Test if AABB is inside or intersects frustum (conservative test)
    /// Returns true if visible (inside or intersecting)
    pub fn test_aabb(&self, aabb: &AABB) -> bool {
        let center = aabb.center();
        let extents = aabb.half_extents();

        // Test against all 6 planes
        for plane in &self.planes {
            // Calculate radius of AABB projected onto plane normal
            let radius = extents.x * plane.normal.x.abs()
                + extents.y * plane.normal.y.abs()
                + extents.z * plane.normal.z.abs();

            let distance = plane.distance_to_point(center);

            // If center is more than radius behind plane, AABB is outside
            if distance < -radius {
                return false;
            }
        }

        // AABB is inside or intersecting frustum
        true
    }

    /// Test multiple AABBs and return indices of visible ones
    pub fn cull_aabbs(&self, aabbs: &[AABB]) -> Vec<usize> {
        aabbs
            .iter()
            .enumerate()
            .filter(|(_, aabb)| self.test_aabb(aabb))
            .map(|(i, _)| i)
            .collect()
    }
}
