//! DDA 3D Voxel Raycast
//! 
//! High-performance ray-grid traversal using Digital Differential Analyzer
//! 
//! References:
//! - "A Fast Voxel Traversal Algorithm for Ray Tracing" by John Amanatides & Andrew Woo
//! - https://www.shadertoy.com/view/4dX3zl

use glam::{Vec3, IVec3};
use crate::chunk::BlockId;

/// Result of a voxel raycast
#[derive(Debug, Clone, Copy)]
pub struct RaycastHit {
    /// World position of the hit voxel
    pub position: IVec3,
    
    /// Block ID at the hit position
    pub block_id: BlockId,
    
    /// Normal of the hit face (which cube face was hit)
    /// One of: (±1, 0, 0), (0, ±1, 0), (0, 0, ±1)
    pub normal: IVec3,
    
    /// Distance from ray origin to hit point
    pub distance: f32,
    
    /// Position of the adjacent air block (for block placement)
    pub adjacent_position: IVec3,
}

/// Voxel traversal algorithm using DDA
/// 
/// # Arguments
/// * `origin` - Ray start position (world coordinates)
/// * `direction` - Ray direction (must be normalized)
/// * `max_distance` - Maximum ray distance
/// * `check_solid` - Callback function to check if a voxel is solid
/// 
/// # Returns
/// `Some(RaycastHit)` if a solid voxel is hit, `None` otherwise
pub fn raycast_dda<F>(
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    check_solid: F,
) -> Option<RaycastHit>
where
    F: Fn(IVec3) -> Option<BlockId>,
{
    // Current voxel position
    let mut voxel = origin.floor().as_ivec3();
    
    // Direction signs
    let step = IVec3::new(
        if direction.x > 0.0 { 1 } else { -1 },
        if direction.y > 0.0 { 1 } else { -1 },
        if direction.z > 0.0 { 1 } else { -1 },
    );
    
    // Distance along ray to cross one voxel boundary in each axis
    let delta = Vec3::new(
        if direction.x.abs() < 1e-10 { f32::INFINITY } else { (1.0 / direction.x).abs() },
        if direction.y.abs() < 1e-10 { f32::INFINITY } else { (1.0 / direction.y).abs() },
        if direction.z.abs() < 1e-10 { f32::INFINITY } else { (1.0 / direction.z).abs() },
    );
    
    // Distance to next voxel boundary
    let mut t_max = Vec3::new(
        if direction.x > 0.0 {
            (voxel.x as f32 + 1.0 - origin.x) / direction.x
        } else if direction.x < 0.0 {
            (origin.x - voxel.x as f32) / -direction.x
        } else {
            f32::INFINITY
        },
        if direction.y > 0.0 {
            (voxel.y as f32 + 1.0 - origin.y) / direction.y
        } else if direction.y < 0.0 {
            (origin.y - voxel.y as f32) / -direction.y
        } else {
            f32::INFINITY
        },
        if direction.z > 0.0 {
            (voxel.z as f32 + 1.0 - origin.z) / direction.z
        } else if direction.z < 0.0 {
            (origin.z - voxel.z as f32) / -direction.z
        } else {
            f32::INFINITY
        },
    );
    
    // Track the face normal of the last step
    let mut normal = IVec3::ZERO;
    let mut distance = 0.0;
    
    // DDA traversal
    for _ in 0..256 {  // Safety limit: max 256 voxels
        // Check if current voxel is solid
        if let Some(block_id) = check_solid(voxel) {
            if block_id != crate::chunk::AIR {
                // Calculate adjacent position (for block placement)
                let adjacent_position = voxel - normal;
                
                return Some(RaycastHit {
                    position: voxel,
                    block_id,
                    normal,
                    distance,
                    adjacent_position,
                });
            }
        }
        
        // Check max distance
        if distance > max_distance {
            break;
        }
        
        // Step to next voxel
        if t_max.x < t_max.y {
            if t_max.x < t_max.z {
                // Step in X
                voxel.x += step.x;
                distance = t_max.x;
                t_max.x += delta.x;
                normal = IVec3::new(-step.x, 0, 0);
            } else {
                // Step in Z
                voxel.z += step.z;
                distance = t_max.z;
                t_max.z += delta.z;
                normal = IVec3::new(0, 0, -step.z);
            }
        } else if t_max.y < t_max.z {
            // Step in Y
            voxel.y += step.y;
            distance = t_max.y;
            t_max.y += delta.y;
            normal = IVec3::new(0, -step.y, 0);
        } else {
            // Step in Z
            voxel.z += step.z;
            distance = t_max.z;
            t_max.z += delta.z;
            normal = IVec3::new(0, 0, -step.z);
        }
    }
    
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_raycast_hit() {
        // Simple test: ray hits a solid block at (5, 0, 0)
        let origin = Vec3::new(0.0, 0.0, 0.0);
        let direction = Vec3::new(1.0, 0.0, 0.0).normalize();
        
        let check_solid = |pos: IVec3| -> Option<BlockId> {
            if pos == IVec3::new(5, 0, 0) {
                Some(1) // Solid block
            } else {
                Some(0) // Air
            }
        };
        
        let hit = raycast_dda(origin, direction, 10.0, check_solid);
        assert!(hit.is_some());
        
        let hit = hit.unwrap();
        assert_eq!(hit.position, IVec3::new(5, 0, 0));
        assert_eq!(hit.normal, IVec3::new(-1, 0, 0)); // Hit from -X side
    }
    
    #[test]
    fn test_raycast_miss() {
        let origin = Vec3::new(0.0, 0.0, 0.0);
        let direction = Vec3::new(1.0, 0.0, 0.0).normalize();
        
        // No solid blocks
        let check_solid = |_pos: IVec3| -> Option<BlockId> {
            Some(0) // All air
        };
        
        let hit = raycast_dda(origin, direction, 10.0, check_solid);
        assert!(hit.is_none());
    }
    
    #[test]
    fn test_raycast_diagonal() {
        let origin = Vec3::new(0.5, 0.5, 0.5);
        let direction = Vec3::new(1.0, 1.0, 1.0).normalize();
        
        let check_solid = |pos: IVec3| -> Option<BlockId> {
            if pos == IVec3::new(3, 3, 3) {
                Some(1)
            } else {
                Some(0)
            }
        };
        
        let hit = raycast_dda(origin, direction, 10.0, check_solid);
        assert!(hit.is_some());
        assert_eq!(hit.unwrap().position, IVec3::new(3, 3, 3));
    }
}
