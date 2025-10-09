//! Example: Using the Provider system
//! 
//! This example demonstrates how to use the Milestone 2 provider system

use voxel_engine::*;
use glam::{IVec3, Vec3};
use std::sync::Arc;

fn main() {
    println!("=== Milestone 2 Provider Examples ===\n");
    
    // Example 1: GridStoreProvider
    example_grid_store();
    
    // Example 2: PlanetProvider
    example_planet();
    
    // Example 3: AsteroidProvider
    example_asteroid();
    
    // Example 4: Planet with edits (DeltaStore)
    example_planet_with_edits();
}

fn example_grid_store() {
    println!("--- Example 1: GridStoreProvider ---");
    
    let mut store = GridStoreProvider::new(GridStoreConfig::default());
    
    // Write some voxels
    store.write_voxel(IVec3::new(0, 0, 0), VoxelValue::Block(STONE)).unwrap();
    store.write_voxel(IVec3::new(1, 0, 0), VoxelValue::Block(DIRT)).unwrap();
    store.write_voxel(IVec3::new(0, 1, 0), VoxelValue::Block(GRASS)).unwrap();
    
    // Read region
    let data = store.read_range(
        IVec3::new(-5, -5, -5),
        IVec3::new(5, 5, 5),
        0
    ).unwrap();
    
    println!("Stored {} voxels", data.values.len());
    println!("Chunk count: {}", store.chunk_count());
    
    // Get dirty chunks
    let dirty = store.take_dirty_chunks();
    println!("Dirty chunks: {}", dirty.len());
    
    println!();
}

fn example_planet() {
    println!("--- Example 2: PlanetProvider ---");
    
    let config = PlanetConfig {
        seed: 42,
        radius: 1000.0,
        center: Vec3::new(0.0, 1000.0, 0.0),
        noise_stack: vec![
            NoiseLayer {
                frequency: 0.01,
                amplitude: 50.0,
                octaves: 4,
                lacunarity: 2.0,
                persistence: 0.5,
            },
        ],
        biome_bands: vec![
            BiomeBand {
                lat_min: -1.0,
                lat_max: -0.5,
                biome: BiomeType::Ice,
            },
            BiomeBand {
                lat_min: -0.5,
                lat_max: 0.5,
                biome: BiomeType::Temperate,
            },
            BiomeBand {
                lat_min: 0.5,
                lat_max: 1.0,
                biome: BiomeType::Ice,
            },
        ],
        sea_level: 950.0,
    };
    
    let planet = PlanetProvider::new(config);
    
    // Read a region at the surface
    let data = planet.read_range(
        IVec3::new(-10, 0, -10),
        IVec3::new(10, 20, 10),
        0
    ).unwrap();
    
    // Count solid vs air
    let mut solid = 0;
    let mut air = 0;
    for val in &data.values {
        if val.is_solid() {
            solid += 1;
        } else {
            air += 1;
        }
    }
    
    println!("Planet region: {} solid, {} air", solid, air);
    println!("Provider: {}", planet.provider_name());
    println!("Writable: {}", planet.is_writable());
    
    println!();
}

fn example_asteroid() {
    println!("--- Example 3: AsteroidProvider ---");
    
    let config = AsteroidConfig {
        seed: 123,
        size: 30.0,
        center: Vec3::new(1000.0, 500.0, 2000.0),
        density_threshold: 0.6,
        noise_mode: NoiseMode::Ridge,
        noise_params: NoiseParams {
            frequency: 0.05,
            octaves: 3,
            lacunarity: 2.0,
            persistence: 0.5,
        },
    };
    
    let asteroid = AsteroidProvider::new(config);
    
    // Read around the asteroid
    let data = asteroid.read_range(
        IVec3::new(980, 480, 1980),
        IVec3::new(1020, 520, 2020),
        0
    ).unwrap();
    
    let mut solid = 0;
    for val in &data.values {
        if val.is_solid() {
            solid += 1;
        }
    }
    
    println!("Asteroid region: {} solid voxels", solid);
    println!("Provider: {}", asteroid.provider_name());
    
    println!();
}

fn example_planet_with_edits() {
    println!("--- Example 4: Planet with DeltaStore ---");
    
    let planet = Arc::new(PlanetProvider::new(PlanetConfig {
        seed: 999,
        radius: 500.0,
        center: Vec3::new(0.0, 500.0, 0.0),
        ..Default::default()
    }));
    
    let mut provider = ProviderWithEdits::new(
        planet,
        GCConfig {
            max_delta_chunks: 1000,
            eviction_policy: EvictionPolicy::LRU,
            auto_flush: false,
            flush_interval: 60.0,
        }
    );
    
    // Read original
    let pos = IVec3::new(10, 10, 10);
    let before = provider.read_range(pos, pos, 0).unwrap();
    println!("Before edit: {:?}", before.get(0, 0, 0));
    
    // Make some edits (dig a tunnel)
    for x in -5..=5 {
        for z in -5..=5 {
            let edit_pos = IVec3::new(x, 10, z);
            provider.write_voxel(edit_pos, VoxelValue::Block(AIR)).unwrap();
        }
    }
    
    // Read after edit
    let after = provider.read_range(pos, pos, 0).unwrap();
    println!("After edit: {:?}", after.get(0, 0, 0));
    
    // Check delta stats
    let delta = provider.delta();
    let stats = delta.read().unwrap().stats();
    println!("Delta stats:");
    println!("  Total deltas: {}", stats.total_deltas);
    println!("  Memory usage: {} bytes", stats.memory_usage_bytes);
    println!("  Dirty chunks: {}", stats.dirty_chunks);
    
    // Optionally save to disk
    // let mut delta_mut = delta.write().unwrap();
    // delta_mut.flush_to_disk(Path::new("planet_edits.delta")).unwrap();
    
    println!();
    println!("=== All examples completed ===");
}
