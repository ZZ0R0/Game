# Guide de démarrage - Milestone 2 Provider System

## 📋 Table des matières

1. [Vue d'ensemble](#vue-densemble)
2. [Ce qui existe dans le projet](#ce-qui-existe-dans-le-projet)
3. [Comment utiliser le système](#comment-utiliser-le-système)
4. [Exemples concrets](#exemples-concrets)
5. [Tests et validation](#tests-et-validation)

---

## Vue d'ensemble

Le **Milestone 2** ajoute un système de **providers** au moteur voxel. Un provider est une source de données voxel qui peut être :
- **Stockée en mémoire** (GridStoreProvider)
- **Générée procéduralement** (PlanetProvider, AsteroidProvider)
- **Modifiable** (via DeltaStore)

### Pourquoi des providers ?

Avant le Milestone 2, le système utilisait directement `ChunkManager` et `TerrainGenerator`. Le nouveau système apporte :

✅ **Modularité** : Changer facilement de source de données  
✅ **Composition** : Combiner base procédurale + modifications  
✅ **LOD** : Support natif des niveaux de détail  
✅ **Persistance** : Sauvegarder/charger les modifications  
✅ **Performance** : Optimisations spécifiques par type

---

## Ce qui existe dans le projet

### Fichiers créés

```
crates/voxel_engine/
│
├── MILESTONE_2_GUIDE.md          ← Documentation détaillée (600 lignes)
├── MILESTONE_2_STATUS.md         ← Status d'implémentation
├── MARCHE_A_SUIVRE.md            ← Ce fichier
│
├── src/
│   ├── providers.rs              ← Implémentation complète (1626 lignes)
│   └── lib.rs                    ← Exports mis à jour
│
└── examples/
    └── milestone2_providers.rs   ← Exemples fonctionnels
```

### Types implémentés

#### 1. **VoxelProvider (trait)**
Interface commune pour tous les providers.

```rust
pub trait VoxelProvider: Send + Sync {
    fn read_range(&self, min: IVec3, max: IVec3, lod: u32) 
        -> Result<VoxelData, ProviderError>;
    fn write_voxel(&mut self, pos: IVec3, value: VoxelValue) 
        -> Result<(), ProviderError>;
    fn provider_name(&self) -> &str;
    fn is_writable(&self) -> bool;
}
```

#### 2. **GridStoreProvider**
Stockage en mémoire basé sur chunks (comme ChunkManager mais avec l'interface provider).

**Utilisation** : Régions éditables, bases de joueurs, structures

#### 3. **PlanetProvider**
Génération procédurale de planètes sphériques avec noise et biomes.

**Utilisation** : Planètes, lunes, corps célestes

#### 4. **AsteroidProvider**
Génération d'astéroïdes avec ridge noise (arêtes vives).

**Utilisation** : Ceintures d'astéroïdes, rochers spatiaux

#### 5. **DeltaStore**
Système de stockage sparse pour modifications.

**Utilisation** : Overlay pour rendre les providers procéduraux éditables

#### 6. **ProviderWithEdits<P>**
Wrapper qui combine un provider de base + DeltaStore.

**Utilisation** : Planète éditable, terrain modifiable

---

## Comment utiliser le système

### Étape 1 : Importer les types

```rust
use voxel_engine::*;
use glam::{IVec3, Vec3};
use std::sync::Arc;
```

### Étape 2 : Choisir votre cas d'usage

#### Cas A : Région éditable simple

```rust
// Créer un store
let mut store = GridStoreProvider::new(GridStoreConfig::default());

// Écrire des voxels
store.write_voxel(IVec3::new(0, 0, 0), VoxelValue::Block(STONE)).unwrap();

// Lire une région
let data = store.read_range(
    IVec3::new(-10, -10, -10),
    IVec3::new(10, 10, 10),
    0 // LOD 0
).unwrap();

// Obtenir les chunks modifiés pour le meshing
let dirty = store.take_dirty_chunks();
for chunk_pos in dirty {
    // Regenerer le mesh de ce chunk
}
```

#### Cas B : Planète procédurale (lecture seule)

```rust
// Configuration
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

// Lire une région
let data = planet.read_range(
    IVec3::new(-32, 0, -32),
    IVec3::new(32, 64, 32),
    0
).unwrap();

// Utiliser les données pour le meshing
```

#### Cas C : Planète éditable (avec DeltaStore)

```rust
// 1. Créer la planète de base
let planet = Arc::new(PlanetProvider::new(config));

// 2. Envelopper avec support d'édition
let mut editable = ProviderWithEdits::new(
    planet,
    GCConfig {
        max_delta_chunks: 1000,
        eviction_policy: EvictionPolicy::LRU,
        auto_flush: false,
        flush_interval: 60.0,
    }
);

// 3. Lire (combine base + modifications)
let data = editable.read_range(min, max, 0).unwrap();

// 4. Modifier
editable.write_voxel(
    IVec3::new(10, 10, 10),
    VoxelValue::Block(AIR)
).unwrap();

// 5. Sauvegarder les modifications
let delta = editable.delta();
let mut delta_lock = delta.write().unwrap();
delta_lock.flush_to_disk("world_edits.delta").unwrap();

// 6. Plus tard : recharger
let loaded_delta = DeltaStore::load_from_disk("world_edits.delta").unwrap();
let restored = ProviderWithEdits {
    base: planet,
    delta: Arc::new(RwLock::new(loaded_delta)),
};
```

### Étape 3 : Intégration avec le meshing

```rust
// Pour mesher une région :
let data = provider.read_range(chunk_min, chunk_max, 0)?;

// Convertir VoxelData en format pour le mesher
for z in 0..data.size.z {
    for y in 0..data.size.y {
        for x in 0..data.size.x {
            if let Some(voxel) = data.get(x, y, z) {
                match voxel {
                    VoxelValue::Block(block_id) => {
                        // Utiliser avec mesh_chunk
                    }
                    VoxelValue::Density(density, material) => {
                        // Utiliser avec marching cubes (futur)
                    }
                }
            }
        }
    }
}
```

---

## Exemples concrets

### Exemple 1 : Créer un champ d'astéroïdes

```rust
fn create_asteroid_field() -> Vec<AsteroidProvider> {
    let mut asteroids = Vec::new();
    
    for i in 0..100 {
        let config = AsteroidConfig {
            seed: i,
            size: 10.0 + (i as f32 * 0.5),
            center: Vec3::new(
                (i as f32 * 100.0).sin() * 1000.0,
                (i as f32 * 50.0).cos() * 500.0,
                i as f32 * 100.0,
            ),
            density_threshold: 0.6,
            noise_mode: NoiseMode::Ridge,
            noise_params: NoiseParams::default(),
        };
        
        asteroids.push(AsteroidProvider::new(config));
    }
    
    asteroids
}

// Utilisation
let asteroids = create_asteroid_field();

// Pour chaque astéroïde proche du joueur
for asteroid in &asteroids {
    let player_chunk = world_to_chunk_pos(player_pos);
    let data = asteroid.read_range(
        player_chunk * 32 - IVec3::splat(32),
        player_chunk * 32 + IVec3::splat(32),
        1 // LOD 1 pour astéroïdes lointains
    )?;
    
    // Mesher et afficher
}
```

### Exemple 2 : Planète avec océans et biomes

```rust
fn create_earth_like_planet() -> PlanetProvider {
    PlanetConfig {
        seed: 12345,
        radius: 6371.0, // Rayon de la Terre en km = blocs
        center: Vec3::ZERO,
        
        // Plusieurs couches de noise pour terrain réaliste
        noise_stack: vec![
            // Continents
            NoiseLayer {
                frequency: 0.0001,
                amplitude: 2000.0,
                octaves: 6,
                lacunarity: 2.0,
                persistence: 0.5,
            },
            // Montagnes
            NoiseLayer {
                frequency: 0.001,
                amplitude: 500.0,
                octaves: 4,
                lacunarity: 2.5,
                persistence: 0.4,
            },
            // Détails
            NoiseLayer {
                frequency: 0.01,
                amplitude: 50.0,
                octaves: 3,
                lacunarity: 2.0,
                persistence: 0.5,
            },
        ],
        
        // Biomes par latitude
        biome_bands: vec![
            BiomeBand { lat_min: -1.0, lat_max: -0.8, biome: BiomeType::Ice },
            BiomeBand { lat_min: -0.8, lat_max: -0.3, biome: BiomeType::Tundra },
            BiomeBand { lat_min: -0.3, lat_max: -0.1, biome: BiomeType::Temperate },
            BiomeBand { lat_min: -0.1, lat_max: 0.1, biome: BiomeType::Tropical },
            BiomeBand { lat_min: 0.1, lat_max: 0.3, biome: BiomeType::Temperate },
            BiomeBand { lat_min: 0.3, lat_max: 0.8, biome: BiomeType::Tundra },
            BiomeBand { lat_min: 0.8, lat_max: 1.0, biome: BiomeType::Ice },
        ],
        
        sea_level: 6371.0 - 100.0, // Niveau de la mer
    }
}
```

### Exemple 3 : Brush pour creuser/construire

```rust
fn dig_sphere(provider: &mut impl VoxelProvider, center: IVec3, radius: f32) {
    let brush = Brush {
        shape: BrushShape::Sphere,
        size: radius,
        value: VoxelValue::Block(AIR),
    };
    
    provider.write_brush(center, &brush).unwrap();
}

fn build_pillar(provider: &mut impl VoxelProvider, base: IVec3, height: i32) {
    for y in 0..height {
        provider.write_voxel(
            base + IVec3::new(0, y, 0),
            VoxelValue::Block(STONE)
        ).unwrap();
    }
}
```

---

## Tests et validation

### Lancer les tests

```bash
# Tous les tests du provider system
cargo test -p voxel_engine providers

# Tests d'acceptance spécifiques
cargo test -p voxel_engine acceptance_

# Test de performance
cargo test -p voxel_engine acceptance_read_64_cubed_perf --release
```

### Résultats attendus

```
running 15 tests
test providers::tests::test_voxel_data ... ok
test providers::tests::test_brush_sphere ... ok
test providers::tests::test_grid_store ... ok
test providers::tests::test_planet_deterministic ... ok
test providers::tests::test_planet_read_range ... ok
test providers::tests::test_asteroid_generation ... ok
test providers::tests::test_delta_store ... ok
test providers::tests::test_provider_with_edits ... ok
test providers::tests::test_delta_save_load ... ok
test providers::tests::test_gc_lru ... ok
test providers::tests::test_lod_sampling ... ok
test providers::tests::acceptance_deterministic_reads ... ok
test providers::tests::acceptance_edits_override_base ... ok
test providers::tests::acceptance_survive_reload ... ok
test providers::tests::acceptance_read_64_cubed_perf ... ok

test result: ok. 15 passed; 0 failed
```

### Lancer l'exemple

```bash
cargo run --example milestone2_providers -p voxel_engine
```

Cet exemple montre :
- GridStore basique
- Génération de planète
- Génération d'astéroïde
- Planète avec éditions + delta store

---

## Checklist d'intégration

Pour intégrer le provider system dans votre jeu :

### Phase 1 : Migration du code existant

- [ ] Remplacer `ChunkManager` par `GridStoreProvider`
- [ ] Remplacer `TerrainGenerator` par `PlanetProvider`
- [ ] Adapter la boucle de meshing pour utiliser `VoxelProvider::read_range`

### Phase 2 : Ajout des fonctionnalités

- [ ] Implémenter l'édition de terrain (creuser/construire)
- [ ] Ajouter le système de sauvegarde (DeltaStore → disque)
- [ ] Implémenter le LOD pour chunks lointains

### Phase 3 : Optimisations

- [ ] Mettre en place le GC pour le DeltaStore
- [ ] Ajouter un cache de chunks lus
- [ ] Implémenter le streaming asynchrone

---

## Questions fréquentes

### Q : Puis-je utiliser plusieurs providers en même temps ?

**R :** Oui ! Vous pouvez avoir plusieurs providers actifs :

```rust
let planet = Arc::new(PlanetProvider::new(planet_config));
let asteroids: Vec<_> = /* ... */;
let station = GridStoreProvider::new(grid_config);

// Lire depuis différentes sources selon la position
```

### Q : Comment gérer le streaming de chunks ?

**R :** Utilisez `read_range` avec différents LOD selon la distance :

```rust
let distance = (chunk_pos - player_chunk).length();
let lod = if distance < 5 { 0 }
          else if distance < 10 { 1 }
          else { 2 };

let data = provider.read_range(min, max, lod)?;
```

### Q : Les providers sont-ils thread-safe ?

**R :** Oui ! Tous les providers implémentent `Send + Sync`. Vous pouvez les utiliser depuis plusieurs threads.

### Q : Comment migrer depuis Milestone 1 ?

**R :** Remplacez :

```rust
// Avant
let mut volume = CelestialVolume::new(generator, transform);

// Après
let planet = Arc::new(PlanetProvider::new(config));
let provider = ProviderWithEdits::new(planet, gc_config);
```

---

## Prochaines étapes recommandées

1. **Lire** `MILESTONE_2_GUIDE.md` pour les détails techniques
2. **Exécuter** `cargo run --example milestone2_providers -p voxel_engine`
3. **Tester** `cargo test -p voxel_engine providers`
4. **Intégrer** dans votre jeu en suivant la checklist ci-dessus

---

**Milestone 2 est complet et prêt à l'emploi !** 🎉

Pour toute question, référez-vous à :
- `MILESTONE_2_GUIDE.md` : Documentation détaillée
- `MILESTONE_2_STATUS.md` : Status d'implémentation
- `src/providers.rs` : Code source commenté
- `examples/milestone2_providers.rs` : Exemples pratiques
