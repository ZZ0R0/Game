# Système de Delta pour la Synchronisation d'État

## Vue d'ensemble

Le système de delta fonctionne de manière récursive, en propageant les changements depuis les entités enfants jusqu'à l'entité racine (World). Cette approche permet de :
- Minimiser le trafic réseau en n'envoyant que les changements
- Optimiser les performances en évitant de recharger des objets non modifiés
- Maintenir la cohérence de l'état entre serveur et clients

## Architecture

### Hiérarchie des entités
```
World
├── Grids
│   └── Blocks
├── Players
└── CelestialBodies
```

### Composants du système

1. **Mutator Pattern**
   - Chaque entité possède son propre Mutator
   - Le Mutator accumule temporairement les changements
   - Les changements sont appliqués atomiquement via .apply()

2. **Delta Objects**
   - BlockDelta: changements d'un block
   - GridDelta: changements d'une grid + deltas des blocks
   - PlayerDelta: changements d'un player
   - CelestialDelta: changements d'un corps céleste
   - WorldDelta: agrège tous les deltas

3. **Références parentales**
   - Weak<RefCell<Parent>> pour éviter les cycles
   - Permet la propagation vers le haut

## Flux de Propagation

### 1. Modification d'un Block
```rust
// Modification via Mutator
block.mutate()
    .set_integrity(50.0)
    .add_component("power", PowerStorage::new(100.0))
    .apply();

// Génère BlockDelta
BlockDelta {
    in_grid_id: block.id,
    integrity: Some(50.0),
    component_changes: {
        "power": ComponentDelta::Added(PowerStorage { ... })
    }
}
```

### 2. Propagation à la Grid
```rust
// La Grid reçoit le BlockDelta
impl Grid {
    fn on_block_changed(&mut self, block_delta: BlockDelta) {
        // Enregistre le delta du block
        self.pending_block_deltas.insert(block_delta.in_grid_id, block_delta);
        
        // Crée et propage son propre delta
        let grid_delta = GridDelta {
            grid_id: self.id,
            blocks_delta: self.pending_block_deltas.clone(),
            ...
        };
        self.propagate_delta(grid_delta);
    }
}
```

### 3. Propagation au World
```rust
// Le World reçoit le GridDelta
impl World {
    fn on_grid_changed(&mut self, grid_delta: GridDelta) {
        // Accumule dans le WorldDelta
        self.pending_world_delta.grid_changes
            .insert(grid_delta.grid_id, grid_delta);
    }
}
```

## Synchronisation Client-Serveur

### Côté Serveur

1. **Accumulation des changements**
```rust
// À chaque modification
world.on_grid_changed(grid_delta);
world.on_player_changed(player_delta);
world.on_celestial_changed(celestial_delta);

// En fin de frame
if let Some(world_delta) = world.flush_world_delta() {
    network.broadcast(world_delta);
}
```

2. **Optimisation réseau**
- Les deltas sont fusionnés avant envoi
- Seuls les champs modifiés sont inclus
- Les deltas vides ne sont pas transmis

### Côté Client

1. **Réception des deltas**
```rust
fn on_world_delta_received(delta: WorldDelta) {
    // Appliquer les changements dans l'ordre
    
    // 1. Mettre à jour les grids
    for (grid_id, grid_delta) in delta.grid_changes {
        if let Some(grid) = world.get_grid_mut(grid_id) {
            // D'abord mettre à jour la grid elle-même
            grid_delta.apply_to_grid(grid);
            
            // Puis ses blocks
            for (block_id, block_delta) in grid_delta.blocks_delta {
                if let Some(block) = grid.get_block_mut(block_id) {
                    block_delta.apply_to_block(block);
                }
            }
        }
    }
    
    // 2. Mettre à jour les players
    for (player_id, player_delta) in delta.player_changes {
        if let Some(player) = world.get_player_mut(player_id) {
            player_delta.apply_to_player(player);
        }
    }
    
    // 3. Mettre à jour les celestials
    for (celestial_id, celestial_delta) in delta.celestial_changes {
        if let Some(celestial) = world.get_celestial_mut(celestial_id) {
            celestial_delta.apply_to_celestial(celestial);
        }
    }
}
```

2. **Validation et cohérence**
```rust
impl WorldDelta {
    fn validate(&self) -> bool {
        // Vérifier la séquence
        if self.sequence <= last_applied_sequence {
            return false;
        }
        
        // Vérifier les références
        for (grid_id, grid_delta) in &self.grid_changes {
            if !world.grid_exists(*grid_id) {
                return false;
            }
            // ...
        }
        
        true
    }
}
```

3. **Gestion des erreurs**
```rust
fn apply_delta_safe(delta: WorldDelta) -> Result<(), DeltaError> {
    // Créer une transaction
    let mut transaction = WorldTransaction::new();
    
    // Tenter d'appliquer
    transaction.apply_delta(delta)?;
    
    // Si tout est ok, commit
    transaction.commit();
    Ok(())
}
```

## Points importants

1. **Atomicité**
   - Les modifications sont appliquées en une seule opération
   - Si une erreur survient, l'état reste cohérent

2. **Ordre**
   - Les deltas sont appliqués dans l'ordre de leur séquence
   - Les dépendances sont respectées (grid avant blocks)

3. **Performance**
   - Les deltas sont optimisés pour la taille réseau
   - Les modifications sont appliquées directement aux bonnes entités

4. **Robustesse**
   - Gestion des erreurs à chaque niveau
   - Validation des références et des données
   - Système de rollback en cas d'erreur


