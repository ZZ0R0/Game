# Milestone 2 — Providers (Data Sources)

## Vue d'ensemble

Le Milestone 2 introduit un système de **providers** modulaires pour les sources de données voxel. Ces providers permettent de séparer la logique de génération/stockage de données de la logique de gestion de volumes.

## Objectifs

✅ **Pluggable data sources** : Interface commune pour différentes sources de voxels  
✅ **LOD support** : Lecture à différents niveaux de détail  
✅ **Edit overlay** : Modifications superposées aux données de base  
✅ **Deterministic generation** : Même seed = mêmes données  
✅ **Performance** : Lecture rapide (64³ region < X ms à LOD0)

## Architecture

```
VoxelProvider (trait)
├── GridStoreProvider (chunk map storage)
│   ├── Palette management
│   ├── Dirty tracking
│   └── Read/write operations
│
├── PlanetProvider (procedural sphere)
│   ├── Seed-based generation
│   ├── Radius + noise stack
│   ├── Biome bands
│   └── LOD sampling
│
├── AsteroidProvider (procedural rock)
│   ├── Per-rock seed
│   ├── Size + density threshold
│   └── Ridge noise mode
│
└── DeltaStore (sparse overlay)
    ├── Sparse chunk overlay
    ├── GC strategy
    └── Disk-flush policy
```

## 1. VoxelProvider (trait)

Le trait de base pour tous les providers :

```rust
pub trait VoxelProvider: Send + Sync {
    /// Read voxel data in a range with LOD sampling
    /// 
    /// - `min`, `max`: World coordinates (inclusive)
    /// - `lod`: Level of detail (0 = full resolution, 1+ = lower resolution)
    /// - Returns: VoxelData or error
    fn read_range(&self, min: IVec3, max: IVec3, lod: u32) -> Result<VoxelData, ProviderError>;
    
    /// Write a single voxel (or brush pattern)
    /// 
    /// - `pos`: World position
    /// - `value`: Voxel value to write
    /// - Returns: Success or error
    fn write_voxel(&mut self, pos: IVec3, value: VoxelValue) -> Result<(), ProviderError>;
    
    /// Write a brush (multiple voxels)
    fn write_brush(&mut self, center: IVec3, brush: &Brush) -> Result<(), ProviderError>;
    
    /// Get provider name for debugging
    fn provider_name(&self) -> &str;
    
    /// Check if provider supports writes
    fn is_writable(&self) -> bool;
}
```

## 2. GridStoreProvider

Stockage en mémoire basé sur chunks avec palette.

### Caractéristiques

- **Chunk map** : HashMap de chunks 32³
- **Palette compression** : Réduit la mémoire pour chunks uniformes
- **Dirty tracking** : Marque les régions modifiées
- **Thread-safe** : RwLock pour accès concurrent

### Structure de données

```rust
pub struct GridStoreProvider {
    /// Chunks stockés en mémoire
    chunks: Arc<RwLock<HashMap<IVec3, ChunkData>>>,
    
    /// Régions modifiées
    dirty_regions: Arc<RwLock<HashSet<IVec3>>>,
    
    /// Configuration
    config: GridStoreConfig,
}

pub struct ChunkData {
    /// Données de voxels (avec palette)
    blocks: Box<[BlockId; CHUNK_VOLUME]>,
    
    /// Palette locale (compression)
    palette: Vec<BlockId>,
    
    /// Version (pour invalidation de cache)
    version: u32,
}
```

### Gestion de palette

- **Compaction** : Si palette > 256 entrées, compacter
- **Stratégie** : LRU pour conserver les blocs les plus utilisés
- **Uniforme** : Chunks uniformes = 1 entrée de palette

### API

```rust
impl GridStoreProvider {
    pub fn new(config: GridStoreConfig) -> Self;
    
    /// Insérer un chunk
    pub fn insert_chunk(&mut self, pos: IVec3, data: ChunkData);
    
    /// Obtenir les chunks dirty
    pub fn take_dirty_chunks(&mut self) -> Vec<IVec3>;
    
    /// Compacter les palettes
    pub fn compact_palettes(&mut self);
}
```

## 3. PlanetProvider

Génération procédurale de planètes sphériques.

### Paramètres

```rust
pub struct PlanetConfig {
    /// Seed pour reproductibilité
    pub seed: u64,
    
    /// Rayon de la planète (en blocs)
    pub radius: f32,
    
    /// Centre de la planète
    pub center: Vec3,
    
    /// Stack de noise (plusieurs octaves)
    pub noise_stack: Vec<NoiseLayer>,
    
    /// Bandes de biomes (latitude)
    pub biome_bands: Vec<BiomeBand>,
    
    /// Niveau de la mer
    pub sea_level: f32,
}

pub struct NoiseLayer {
    pub frequency: f32,
    pub amplitude: f32,
    pub octaves: u32,
    pub lacunarity: f32,
    pub persistence: f32,
}

pub struct BiomeBand {
    /// Latitude min/max (0.0 = équateur, ±1.0 = pôles)
    pub lat_min: f32,
    pub lat_max: f32,
    
    /// Type de biome
    pub biome: BiomeType,
}
```

### Fonctionnement

```rust
impl PlanetProvider {
    /// Calculer la distance signée à la surface
    /// Négatif = à l'intérieur, Positif = à l'extérieur
    fn signed_distance(&self, pos: Vec3) -> f32;
    
    /// Échantillonner le matériau à une position
    fn sample_material(&self, pos: Vec3, distance: f32) -> MaterialId;
    
    /// Lire avec LOD
    /// LOD0 = 1 bloc/voxel, LOD1 = 2 blocs/voxel, etc.
    fn read_range_lod(&self, min: IVec3, max: IVec3, lod: u32) -> VoxelData;
}
```

### Support d'éditions

Les modifications sont appliquées via un **DeltaStore** superposé :

```rust
pub struct PlanetProviderWithEdits {
    /// Provider de base (immuable)
    base: Arc<PlanetProvider>,
    
    /// Overlay de modifications
    delta: Arc<RwLock<DeltaStore>>,
}

impl VoxelProvider for PlanetProviderWithEdits {
    fn read_range(&self, min: IVec3, max: IVec3, lod: u32) -> Result<VoxelData, ProviderError> {
        // 1. Lire le champ de base
        let mut data = self.base.read_range(min, max, lod)?;
        
        // 2. Appliquer les deltas
        self.delta.read().unwrap().apply_to(&mut data, min, max);
        
        Ok(data)
    }
    
    fn write_voxel(&mut self, pos: IVec3, value: VoxelValue) -> Result<(), ProviderError> {
        let mut delta = self.delta.write().unwrap();
        delta.set_voxel(pos, value);
        Ok(())
    }
}
```

## 4. AsteroidProvider

Génération d'astéroïdes avec noise ridge.

### Paramètres

```rust
pub struct AsteroidConfig {
    /// Seed unique par astéroïde
    pub seed: u64,
    
    /// Taille approximative (rayon)
    pub size: f32,
    
    /// Centre
    pub center: Vec3,
    
    /// Seuil de densité (0.0-1.0)
    pub density_threshold: f32,
    
    /// Mode noise (ridge pour créneaux)
    pub noise_mode: NoiseMode,
    
    /// Paramètres de noise
    pub noise_params: NoiseParams,
}

pub enum NoiseMode {
    /// Perlin standard
    Standard,
    
    /// Ridge noise (abs() pour créneaux)
    Ridge,
    
    /// Billowy (carrés)
    Billowy,
}
```

### Fonctionnement

```rust
impl AsteroidProvider {
    /// Calculer la densité à une position
    /// Ridge noise crée des arêtes vives
    fn density_at(&self, pos: Vec3) -> f32;
    
    /// Déterminer si un voxel est solide
    fn is_solid(&self, pos: Vec3) -> bool {
        self.density_at(pos) > self.config.density_threshold
    }
}
```

## 5. DeltaStore

Stockage sparse des modifications.

### Structure

```rust
pub struct DeltaStore {
    /// Chunks modifiés (sparse)
    deltas: HashMap<IVec3, DeltaChunk>,
    
    /// Configuration GC
    gc_config: GCConfig,
    
    /// Statistiques
    stats: DeltaStats,
}

pub struct DeltaChunk {
    /// Voxels modifiés dans ce chunk
    /// Key = index local (0..32767)
    /// Value = nouvelle valeur
    modifications: HashMap<usize, VoxelValue>,
    
    /// Timestamp de dernière modification
    last_modified: Instant,
    
    /// Dirty flag
    dirty: bool,
}

pub struct GCConfig {
    /// Nombre max de chunks delta
    max_delta_chunks: usize,
    
    /// Politique d'éviction
    eviction_policy: EvictionPolicy,
    
    /// Flush vers disque automatique
    auto_flush: bool,
    
    /// Intervalle de flush (en secondes)
    flush_interval: f32,
}

pub enum EvictionPolicy {
    /// LRU (Least Recently Used)
    LRU,
    
    /// Éviter les chunks proche du joueur
    SpatialLRU { player_pos: Vec3, radius: f32 },
    
    /// Jamais évincé (mémoire infinie)
    Never,
}
```

### API

```rust
impl DeltaStore {
    pub fn new(config: GCConfig) -> Self;
    
    /// Ajouter une modification
    pub fn set_voxel(&mut self, pos: IVec3, value: VoxelValue);
    
    /// Appliquer les deltas à des données
    pub fn apply_to(&self, data: &mut VoxelData, min: IVec3, max: IVec3);
    
    /// Garbage collection
    pub fn gc(&mut self);
    
    /// Flush vers disque
    pub fn flush_to_disk(&mut self, path: &Path) -> Result<(), io::Error>;
    
    /// Charger depuis le disque
    pub fn load_from_disk(path: &Path) -> Result<Self, io::Error>;
    
    /// Statistiques
    pub fn stats(&self) -> &DeltaStats;
}

pub struct DeltaStats {
    pub total_deltas: usize,
    pub memory_usage_bytes: usize,
    pub dirty_chunks: usize,
}
```

### Stratégie de GC

1. **Vérifier la limite** : Si `deltas.len() > max_delta_chunks`
2. **Trier par politique** : LRU ou SpatialLRU
3. **Évincir les plus anciens** : Flush vers disque si `auto_flush`
4. **Nettoyer** : Libérer la mémoire

### Flush disque

Format binaire simple :

```
Header:
- Magic: "DLTA" (4 bytes)
- Version: u32
- Chunk count: u32

For each chunk:
- Chunk pos: IVec3 (12 bytes)
- Modification count: u32
- For each modification:
  - Local index: u16
  - VoxelValue: depends on type (BlockId = u16, Density = u8+u8)
```

## Acceptance Criteria

### ✅ Deterministic reads

```rust
let planet1 = PlanetProvider::new(config.clone());
let planet2 = PlanetProvider::new(config.clone());

let data1 = planet1.read_range(min, max, 0).unwrap();
let data2 = planet2.read_range(min, max, 0).unwrap();

assert_eq!(data1, data2); // Même seed = mêmes données
```

### ✅ Edits override base field

```rust
let mut provider = PlanetProviderWithEdits::new(planet);

// Lire avant
let before = provider.read_range(pos, pos, 0).unwrap();

// Modifier
provider.write_voxel(pos, VoxelValue::Block(AIR)).unwrap();

// Lire après
let after = provider.read_range(pos, pos, 0).unwrap();

assert_ne!(before, after); // Modifié
```

### ✅ Survive reload

```rust
// Sauvegarder
provider.delta().flush_to_disk("world.delta").unwrap();

// Recharger
let delta = DeltaStore::load_from_disk("world.delta").unwrap();
let reloaded = PlanetProviderWithEdits::with_delta(planet, delta);

// Vérifier
let data = reloaded.read_range(pos, pos, 0).unwrap();
assert_eq!(data, expected); // Modifications préservées
```

### ✅ Benchmark: Read 64³ region

```rust
let start = Instant::now();
let data = provider.read_range(
    IVec3::ZERO,
    IVec3::splat(64),
    0 // LOD 0
).unwrap();
let elapsed = start.elapsed();

assert!(elapsed.as_millis() < 50); // < 50ms pour 64³ voxels
```

## Exemples d'utilisation

### Créer une planète avec éditions

```rust
use voxel_engine::providers::*;

// Configuration de planète
let config = PlanetConfig {
    seed: 42,
    radius: 1000.0,
    center: Vec3::ZERO,
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

// Créer le provider
let planet = Arc::new(PlanetProvider::new(config));
let mut provider = PlanetProviderWithEdits::new(planet);

// Lire une région
let data = provider.read_range(
    IVec3::new(-32, -32, -32),
    IVec3::new(32, 32, 32),
    0
).unwrap();

// Modifier (creuser un tunnel)
for x in -10..=10 {
    for z in -10..=10 {
        let pos = IVec3::new(x, 0, z);
        provider.write_voxel(pos, VoxelValue::Block(AIR)).unwrap();
    }
}

// Sauvegarder
provider.delta().flush_to_disk("my_planet.delta").unwrap();
```

### Créer un champ d'astéroïdes

```rust
let mut asteroids = Vec::new();

for i in 0..100 {
    let config = AsteroidConfig {
        seed: i,
        size: rand::gen_range(10.0..50.0),
        center: random_position_in_belt(),
        density_threshold: 0.6,
        noise_mode: NoiseMode::Ridge,
        noise_params: NoiseParams::default(),
    };
    
    asteroids.push(AsteroidProvider::new(config));
}

// Lire les voxels autour du joueur
for asteroid in &asteroids {
    let data = asteroid.read_range(player_pos - 100, player_pos + 100, 1)?;
    // ... mesh generation ...
}
```

## Tests

Lancer les tests :

```bash
cargo test -p voxel_engine providers
```

Tests spécifiques :

```bash
# Tests de déterminisme
cargo test -p voxel_engine test_deterministic_planet

# Tests de performance
cargo test -p voxel_engine test_read_perf --release

# Tests de persistance
cargo test -p voxel_engine test_delta_save_load
```

## Performance cibles

| Opération | Temps cible | Taille |
|-----------|-------------|--------|
| Read 64³ LOD0 | < 50 ms | 262,144 voxels |
| Read 64³ LOD1 | < 10 ms | 32,768 voxels |
| Write voxel | < 1 ms | 1 voxel |
| Write brush (10³) | < 10 ms | 1,000 voxels |
| GC pass | < 5 ms | 1,000 chunks |
| Flush to disk | < 100 ms | 10 MB |

## Migration depuis Milestone 1

### Avant (Milestone 1)

```rust
let generator = Arc::new(TerrainGenerator::new(config));
let mut volume = CelestialVolume::new(generator, transform);
```

### Après (Milestone 2)

```rust
let planet = Arc::new(PlanetProvider::new(planet_config));
let provider = PlanetProviderWithEdits::new(planet);
let mut volume = CelestialVolume::with_provider(provider, transform);
```

## Prochaines étapes (Milestone 3+)

- **Milestone 3** : Meshing optimisé avec LOD
- **Milestone 4** : Système de chunks asynchrones
- **Milestone 5** : Networking et synchronisation multi-joueurs
- **Milestone 6** : Compression avancée (RLE, palette globale)

---

**Status** : 🚧 En développement  
**Version** : 0.2.0  
**Date** : Octobre 2025
