//! Milestone 3 Example: Meshing Pipeline
//! 
//! Demonstrates both block meshing (greedy quad merge) and density meshing (marching cubes)

use voxel_engine::*;
use glam::IVec3;
use std::time::Instant;

fn main() {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║   Milestone 3: Meshing Pipeline Demonstration       ║");
    println!("╚══════════════════════════════════════════════════════╝\n");
    
    test_blocks_meshing();
    println!();
    test_density_meshing();
    println!();
    test_separated_meshing();
    println!();
    test_unified_output();
}

/// Test 1: Blocks meshing with greedy quad merge
fn test_blocks_meshing() {
    println!("📦 Test 1: Blocks Meshing (Greedy Quad Merge)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let mut chunk = Chunk::new(IVec3::ZERO);
    
    // Create a solid cube (10x10x10)
    println!("Creating 10x10x10 stone cube...");
    for x in 10..20 {
        for y in 10..20 {
            for z in 10..20 {
                chunk.set(x, y, z, STONE);
            }
        }
    }
    
    // Add some transparent blocks
    for i in 12..18 {
        chunk.set(i, 15, 15, GLASS);
    }
    
    let atlas = TextureAtlas::new_16x16();
    let manager = ChunkManager::new();
    
    // Greedy meshing with AO
    println!("Running greedy meshing with AO...");
    let start = Instant::now();
    let mesh = greedy_mesh_chunk(&chunk, Some(&manager), &atlas);
    let elapsed = start.elapsed();
    
    println!("\n✅ Results:");
    println!("   • Vertices: {}", mesh.positions.len());
    println!("   • Triangles: {}", mesh.indices.len() / 3);
    println!("   • UV coords: {}", mesh.uvs.len());
    println!("   • AO values: {}", mesh.ao.len());
    
    let stats = mesh.stats();
    println!("\n📊 Statistics:");
    println!("   • Vertex count: {}", stats.vertex_count);
    println!("   • Triangle count: {}", stats.triangle_count);
    println!("   • Memory usage: {} bytes ({:.2} KB)", stats.memory_bytes, stats.memory_bytes as f32 / 1024.0);
    println!("   • Meshing time: {:.3}ms", elapsed.as_secs_f64() * 1000.0);
    
    let aabb = mesh.calculate_aabb();
    println!("\n📐 Bounding Box:");
    println!("   • Min: ({:.1}, {:.1}, {:.1})", aabb.min.x, aabb.min.y, aabb.min.z);
    println!("   • Max: ({:.1}, {:.1}, {:.1})", aabb.max.x, aabb.max.y, aabb.max.z);
    println!("   • Size: ({:.1}, {:.1}, {:.1})", 
        aabb.size().x, aabb.size().y, aabb.size().z);
}

/// Test 2: Density meshing with marching cubes
fn test_density_meshing() {
    println!("🌍 Test 2: Density Meshing (Marching Cubes)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let mut schema = DensitySchema::new(IVec3::ZERO);
    
    // Create a sphere using signed distance field
    println!("Generating sphere with radius 10...");
    let center = 16.0;
    let radius = 10.0;
    
    for x in 0..32 {
        for y in 0..32 {
            for z in 0..32 {
                let dx = x as f32 - center;
                let dy = y as f32 - center;
                let dz = z as f32 - center;
                let dist = (dx*dx + dy*dy + dz*dz).sqrt();
                
                // Density falloff from center
                let density = if dist < radius {
                    (255.0 - (dist / radius * 127.0)) as u8
                } else {
                    0
                };
                
                schema.set_local(x, y, z, density, MAT_STONE);
            }
        }
    }
    
    // Configure marching cubes
    let config = DensityMeshConfig {
        iso_level: 128.0,
        vertex_snapping: true,
        snap_tolerance: 0.001,
        calculate_normals: true,
        material_blending: MaterialBlendMode::Nearest,
    };
    
    println!("Running marching cubes...");
    println!("Config:");
    println!("   • Iso level: {}", config.iso_level);
    println!("   • Vertex snapping: {}", config.vertex_snapping);
    println!("   • Snap tolerance: {}", config.snap_tolerance);
    println!("   • Calculate normals: {}", config.calculate_normals);
    
    let start = Instant::now();
    let mesh = marching_cubes(&schema, &config);
    let elapsed = start.elapsed();
    
    println!("\n✅ Results:");
    println!("   • Vertices: {}", mesh.positions.len());
    println!("   • Normals: {}", mesh.normals.len());
    println!("   • Materials: {}", mesh.materials.len());
    println!("   • Triangles: {}", mesh.indices.len() / 3);
    println!("   • Meshing time: {:.3}ms", elapsed.as_secs_f64() * 1000.0);
    
    // Calculate approximate memory
    let mem = mesh.positions.len() * 12 + mesh.normals.len() * 12 
             + mesh.materials.len() + mesh.indices.len() * 4;
    println!("   • Memory: {} bytes ({:.2} KB)", mem, mem as f32 / 1024.0);
}

/// Test 3: Separated meshing (opaque vs transparent)
fn test_separated_meshing() {
    println!("🎨 Test 3: Separated Meshing (Opaque/Transparent)");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let mut chunk = Chunk::new(IVec3::ZERO);
    
    // Create mixed geometry
    println!("Creating mixed geometry...");
    
    // Solid floor
    for x in 0..32 {
        for z in 0..32 {
            chunk.set(x, 0, z, STONE);
        }
    }
    
    // Glass walls
    for y in 1..10 {
        for x in 5..27 {
            chunk.set(x, y, 5, GLASS);
            chunk.set(x, y, 26, GLASS);
        }
        for z in 5..27 {
            chunk.set(5, y, z, GLASS);
            chunk.set(26, y, z, GLASS);
        }
    }
    
    // Water inside
    for x in 10..22 {
        for z in 10..22 {
            for y in 1..5 {
                chunk.set(x, y, z, WATER);
            }
        }
    }
    
    let atlas = TextureAtlas::new_16x16();
    let manager = ChunkManager::new();
    
    println!("Running separated meshing...");
    let start = Instant::now();
    let separated = greedy_mesh_chunk_separated(&chunk, Some(&manager), &atlas);
    let elapsed = start.elapsed();
    
    println!("\n✅ Opaque Mesh:");
    println!("   • Vertices: {}", separated.opaque.positions.len());
    println!("   • Triangles: {}", separated.opaque.indices.len() / 3);
    
    println!("\n✅ Transparent Mesh:");
    println!("   • Vertices: {}", separated.transparent.positions.len());
    println!("   • Triangles: {}", separated.transparent.indices.len() / 3);
    
    let total_tris = (separated.opaque.indices.len() + separated.transparent.indices.len()) / 3;
    println!("\n📊 Total:");
    println!("   • Total triangles: {}", total_tris);
    println!("   • Meshing time: {:.3}ms", elapsed.as_secs_f64() * 1000.0);
    
    let opaque_pct = (separated.opaque.indices.len() as f32 / (total_tris * 3) as f32) * 100.0;
    let transp_pct = 100.0 - opaque_pct;
    println!("   • Opaque: {:.1}%", opaque_pct);
    println!("   • Transparent: {:.1}%", transp_pct);
}

/// Test 4: Unified output
fn test_unified_output() {
    println!("🔧 Test 4: Unified Mesh Output");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let mut chunk = Chunk::new(IVec3::ZERO);
    
    // Simple cube
    for x in 10..20 {
        for y in 10..20 {
            for z in 10..20 {
                chunk.set(x, y, z, STONE);
            }
        }
    }
    
    let atlas = TextureAtlas::new_16x16();
    let manager = ChunkManager::new();
    
    let mesh = greedy_mesh_chunk(&chunk, Some(&manager), &atlas);
    
    println!("Converting to unified output...");
    let output = MeshBuildOutput::from_mesh_data(mesh);
    
    println!("\n✅ Unified Output:");
    println!("   • Positions: {}", output.positions.len());
    println!("   • UVs: {}", output.uvs.len());
    println!("   • Normals: {}", output.normals.len());
    println!("   • AO: {}", output.ao.len());
    println!("   • Indices: {}", output.indices.len());
    
    println!("\n📊 Statistics:");
    println!("   • Vertex count: {}", output.stats.vertex_count);
    println!("   • Triangle count: {}", output.stats.triangle_count);
    println!("   • Memory: {} bytes", output.stats.memory_bytes);
    
    println!("\n📐 AABB:");
    println!("   • Min: ({:.1}, {:.1}, {:.1})", output.aabb.min.x, output.aabb.min.y, output.aabb.min.z);
    println!("   • Max: ({:.1}, {:.1}, {:.1})", output.aabb.max.x, output.aabb.max.y, output.aabb.max.z);
    println!("   • Center: ({:.1}, {:.1}, {:.1})", 
        output.aabb.center().x, output.aabb.center().y, output.aabb.center().z);
    
    println!("\n🎯 Submeshes:");
    for (i, submesh) in output.submeshes.iter().enumerate() {
        println!("   • Submesh {}: {:?}, {} indices (start: {})", 
            i, submesh.material_type, submesh.index_count, submesh.start_index);
    }
}
