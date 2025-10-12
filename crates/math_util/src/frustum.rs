//! Frustum culling for efficient visibility determination
//!
//! Provides frustum extraction from view-projection matrices and intersection
//! tests for AABBs and spheres. Used for culling chunks and models that are
//! outside the camera's field of view.

use glam::{Mat4, Vec3, Vec4};

/// A plane in 3D space defined by normal and distance from origin
#[derive(Debug, Clone, Copy)]
pub struct Plane {
    /// Normal vector pointing to the positive half-space
    pub normal: Vec3,
    /// Signed distance from origin along normal
    pub distance: f32,
}

impl Plane {
    /// Create a plane from a Vec4 (a, b, c, d) where ax + by + cz + d = 0
    pub fn from_vec4(v: Vec4) -> Self {
        let normal = Vec3::new(v.x, v.y, v.z);
        let length = normal.length();

        if length > 0.0 {
            Self {
                normal: normal / length,
                distance: v.w / length,
            }
        } else {
            Self {
                normal: Vec3::Y,
                distance: 0.0,
            }
        }
    }

    /// Calculate signed distance from point to plane
    /// Positive = in front of plane, Negative = behind plane
    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.distance
    }

    /// Test if point is in front of (or on) the plane
    pub fn is_in_front(&self, point: Vec3) -> bool {
        self.distance_to_point(point) >= 0.0
    }
}

/// View frustum defined by 6 planes (left, right, bottom, top, near, far)
#[derive(Debug, Clone)]
pub struct Frustum {
    /// The six planes of the frustum
    /// Order: [left, right, bottom, top, near, far]
    pub planes: [Plane; 6],
}

impl Frustum {
    /// Extract frustum planes from a view-projection matrix
    /// Uses the Gribb-Hartmann method for plane extraction
    pub fn from_matrix(view_proj: Mat4) -> Self {
        let m = view_proj.to_cols_array();

        // Extract planes using Gribb-Hartmann method
        // Each plane is extracted by adding/subtracting rows of the matrix
        let left = Plane::from_vec4(Vec4::new(
            m[3] + m[0],
            m[7] + m[4],
            m[11] + m[8],
            m[15] + m[12],
        ));

        let right = Plane::from_vec4(Vec4::new(
            m[3] - m[0],
            m[7] - m[4],
            m[11] - m[8],
            m[15] - m[12],
        ));

        let bottom = Plane::from_vec4(Vec4::new(
            m[3] + m[1],
            m[7] + m[5],
            m[11] + m[9],
            m[15] + m[13],
        ));

        let top = Plane::from_vec4(Vec4::new(
            m[3] - m[1],
            m[7] - m[5],
            m[11] - m[9],
            m[15] - m[13],
        ));

        let near = Plane::from_vec4(Vec4::new(
            m[3] + m[2],
            m[7] + m[6],
            m[11] + m[10],
            m[15] + m[14],
        ));

        let far = Plane::from_vec4(Vec4::new(
            m[3] - m[2],
            m[7] - m[6],
            m[11] - m[10],
            m[15] - m[14],
        ));

        Self {
            planes: [left, right, bottom, top, near, far],
        }
    }

    /// Test if an axis-aligned bounding box (AABB) intersects the frustum
    /// Returns true if the AABB is fully or partially inside the frustum
    pub fn intersects_aabb(&self, min: Vec3, max: Vec3) -> bool {
        for plane in &self.planes {
            // Get the positive vertex (corner closest to plane's positive side)
            let positive_vertex = Vec3::new(
                if plane.normal.x >= 0.0 { max.x } else { min.x },
                if plane.normal.y >= 0.0 { max.y } else { min.y },
                if plane.normal.z >= 0.0 { max.z } else { min.z },
            );

            // If the positive vertex is behind the plane, the entire AABB is outside
            if plane.distance_to_point(positive_vertex) < 0.0 {
                return false; // Outside this plane
            }
        }

        // AABB intersects or is inside all planes
        true
    }

    /// Test if a sphere intersects the frustum (faster than AABB for simple objects)
    /// Returns true if the sphere is fully or partially inside the frustum
    pub fn intersects_sphere(&self, center: Vec3, radius: f32) -> bool {
        for plane in &self.planes {
            if plane.distance_to_point(center) < -radius {
                return false; // Sphere is entirely behind this plane
            }
        }

        true
    }

    /// Test if a point is inside the frustum
    pub fn contains_point(&self, point: Vec3) -> bool {
        for plane in &self.planes {
            if !plane.is_in_front(point) {
                return false;
            }
        }
        true
    }
}

/// Axis-aligned bounding box helper
#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl AABB {
    /// Create a new AABB
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Create an AABB from center and half-extents
    pub fn from_center_extents(center: Vec3, half_extents: Vec3) -> Self {
        Self {
            min: center - half_extents,
            max: center + half_extents,
        }
    }

    /// Get the center of the AABB
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Get the half-extents (half the size along each axis)
    pub fn half_extents(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }

    /// Get all 8 corners of the AABB
    pub fn corners(&self) -> [Vec3; 8] {
        [
            Vec3::new(self.min.x, self.min.y, self.min.z),
            Vec3::new(self.max.x, self.min.y, self.min.z),
            Vec3::new(self.min.x, self.max.y, self.min.z),
            Vec3::new(self.max.x, self.max.y, self.min.z),
            Vec3::new(self.min.x, self.min.y, self.max.z),
            Vec3::new(self.max.x, self.min.y, self.max.z),
            Vec3::new(self.min.x, self.max.y, self.max.z),
            Vec3::new(self.max.x, self.max.y, self.max.z),
        ]
    }

    /// Transform AABB by a matrix (returns axis-aligned bounds of transformed box)
    pub fn transform(&self, matrix: Mat4) -> Self {
        let corners = self.corners();
        let transformed: Vec<_> = corners
            .iter()
            .map(|&corner| matrix.transform_point3(corner))
            .collect();

        let min = transformed
            .iter()
            .fold(Vec3::splat(f32::MAX), |acc: Vec3, &v: &Vec3| acc.min(v));
        let max = transformed
            .iter()
            .fold(Vec3::splat(f32::MIN), |acc: Vec3, &v: &Vec3| acc.max(v));

        Self { min, max }
    }

    /// Test if this AABB intersects the frustum
    pub fn intersects_frustum(&self, frustum: &Frustum) -> bool {
        frustum.intersects_aabb(self.min, self.max)
    }
}
