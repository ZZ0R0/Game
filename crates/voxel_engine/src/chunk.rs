#![allow(clippy::len_without_is_empty)]
use core::ops::{Index, IndexMut};

pub const CHUNK_SIZE: usize = 32;
pub const AIR: u8 = 0;
pub const SOLID: u8 = 1;

pub type Voxel = u8;

#[inline]
const fn idx(x: usize, y: usize, z: usize) -> usize {
    x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE
}

#[derive(Clone)]
pub struct Chunk {
    voxels: Box<[Voxel; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE]>,
}

impl Chunk {
    pub fn new_filled(fill: Voxel) -> Self {
        Self { voxels: Box::new([fill; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE]) }
    }

    pub fn new_empty() -> Self { Self::new_filled(AIR) }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, z: usize, v: Voxel) {
        self.voxels[idx(x, y, z)] = v;
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize, z: usize) -> Voxel {
        self.voxels[idx(x, y, z)]
    }

    pub fn fill_debug_column(&mut self) {
        // Simple test pattern: a 16×16 pillar grid on the floor.
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                if (x % 4 == 0) && (z % 4 == 0) {
                    for y in 0..8 {
                        self.set(x, y, z, SOLID);
                    }
                }
            }
        }
        // Add a flat floor
        for z in 0..CHUNK_SIZE {
            for x in 0..CHUNK_SIZE {
                self.set(x, 0, z, SOLID);
            }
        }
    }

    /// Fill chunk with a complex GPU-intensive pattern: nested spheres and spirals
    /// This creates ~20,000-25,000 visible voxels with maximum face count
    pub fn fill_gpu_stress_test(&mut self) {
        let center = CHUNK_SIZE as f32 / 2.0;
        
        for z in 0..CHUNK_SIZE {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let fx = x as f32 - center;
                    let fy = y as f32 - center;
                    let fz = z as f32 - center;
                    
                    let dist = (fx * fx + fy * fy + fz * fz).sqrt();
                    
                    // Multiple spherical shells
                    let in_sphere1 = dist < 14.0 && dist > 12.0;
                    let in_sphere2 = dist < 10.0 && dist > 8.0;
                    let in_sphere3 = dist < 6.0 && dist > 4.0;
                    
                    // Spiral pattern
                    let angle = fy.atan2(fx);
                    let spiral = ((angle + dist * 0.5).sin() * 2.0).abs() < 1.0;
                    
                    // Checkerboard inside
                    let checker = (x + y + z) % 2 == 0;
                    let in_core = dist < 3.0 && checker;
                    
                    // Pillars extending outward
                    let pillar = (x % 3 == 0 && z % 3 == 0) && y < (16 + (x * z) % 10);
                    
                    // Floor and ceiling
                    let floor_ceiling = y == 0 || y == CHUNK_SIZE - 1;
                    
                    if in_sphere1 || in_sphere2 || in_sphere3 || 
                       (spiral && dist > 5.0 && dist < 13.0) ||
                       in_core || pillar || floor_ceiling {
                        self.set(x, y, z, SOLID);
                    }
                }
            }
        }
    }
}

impl Index<(usize, usize, usize)> for Chunk {
    type Output = Voxel;
    fn index(&self, i: (usize, usize, usize)) -> &Self::Output {
        &self.voxels[idx(i.0, i.1, i.2)]
    }
}
impl IndexMut<(usize, usize, usize)> for Chunk {
    fn index_mut(&mut self, i: (usize, usize, usize)) -> &mut Self::Output {
        &mut self.voxels[idx(i.0, i.1, i.2)]
    }
}
