/// Exemples d'utilisation du système de Delta pour Blocks et Grids
/// 
/// Ce fichier démontre comment utiliser les deltas pour :
/// 1. Tracker les changements d'un block
/// 2. Tracker les changements d'une grille (vaisseau)
/// 3. Optimiser les transferts réseau

#[allow(dead_code)]
mod examples {
    use crate::blocks::{Block, BlockComponent, BlockDelta, ComponentDelta, ComponentChange};
    use crate::grids::{Grid, GridDelta};
    use crate::objects::{FloatPosition, FloatOrientation, Velocity};
    use std::sync::Arc;

    /// Exemple 1: Utilisation des BlockDelta
    /// 
    /// Scénario: Une batterie se charge progressivement
    pub fn example_block_delta_battery_charging() {
        // Supposons qu'on a une batterie quelque part dans le code
        // let mut battery = ... créée ailleurs
        
        // === Tick 1: La batterie commence à charger ===
        // Au lieu de modifier directement, on crée un delta
        let delta1 = BlockDelta {
            in_grid_id: 42, // ID unique de cette batterie dans la grille
            integrity: None,
            mass: None,
            component_changes: {
                let mut changes = std::collections::HashMap::new();
                changes.insert(
                    "power_storage".to_string(),
                    ComponentDelta::Modified(ComponentChange::PowerStorage {
                        charge_change: Some(0.3), // 30% de charge
                    })
                );
                changes
            },
            timestamp: 1000,
            sequence: 1,
        };
        
        // On enregistre le delta (pas encore appliqué)
        // battery.record_delta(delta1);
        
        // === Tick 2: La batterie continue de charger ===
        let delta2 = BlockDelta {
            in_grid_id: 42,
            integrity: None,
            mass: None,
            component_changes: {
                let mut changes = std::collections::HashMap::new();
                changes.insert(
                    "power_storage".to_string(),
                    ComponentDelta::Modified(ComponentChange::PowerStorage {
                        charge_change: Some(0.6), // 60% de charge
                    })
                );
                changes
            },
            timestamp: 2000,
            sequence: 2,
        };
        
        // battery.record_delta(delta2);
        
        // === Synchronisation réseau ===
        // Quand on veut envoyer l'état au client, on fusionne les deltas
        // if let Some(merged_delta) = battery.compute_and_apply_pending_deltas() {
        //     println!("Delta fusionné à envoyer:");
        //     println!("  - Taille: {} bytes", merged_delta.estimated_size());
        //     println!("  - Charge finale: 60%");
        //     
        //     // Sérialiser et envoyer sur le réseau
        //     let serialized = serde_json::to_string(&merged_delta).unwrap();
        //     // send_to_client(serialized);
        // }
        
        println!("Exemple BlockDelta: 2 deltas fusionnés en 1 pour le réseau");
    }
    
    /// Exemple 2: Utilisation des GridDelta avec plusieurs blocks
    /// 
    /// Scénario: Un vaisseau se déplace et plusieurs de ses blocks changent d'état
    pub fn example_grid_delta_ship_movement() {
        // Supposons qu'on a une grille (vaisseau)
        // let mut ship = Grid::new(...);
        
        // === Changements simultanés ===
        
        // 1. Le vaisseau change de position
        let position_delta = GridDelta {
            grid_id: 123, // ID du vaisseau
            position: Some(FloatPosition::new(100.0, 200.0, 300.0)),
            orientation: None,
            velocity: Some(Velocity::new(10.0, 0.0, 5.0)),
            acceleration: None,
            mass: None,
            blocks_delta: std::collections::HashMap::new(),
            timestamp: 1000,
            sequence: 1,
        };
        
        // ship.record_delta(position_delta);
        
        // 2. Un thruster change de force
        let mut grid_delta_with_thruster = GridDelta::empty(123, 1100, 2);
        
        let thruster_delta = BlockDelta {
            in_grid_id: 10, // Thruster #10
            integrity: None,
            mass: None,
            component_changes: {
                let mut changes = std::collections::HashMap::new();
                changes.insert(
                    "thruster".to_string(),
                    ComponentDelta::Modified(ComponentChange::Thruster {
                        force_change: Some(300000.0), // Force augmentée
                    })
                );
                changes
            },
            timestamp: 1100,
            sequence: 2,
        };
        
        grid_delta_with_thruster.add_block_delta(thruster_delta);
        // ship.record_delta(grid_delta_with_thruster);
        
        // 3. Une batterie se décharge
        let mut grid_delta_with_battery = GridDelta::empty(123, 1200, 3);
        
        let battery_delta = BlockDelta {
            in_grid_id: 42, // Battery #42
            integrity: None,
            mass: None,
            component_changes: {
                let mut changes = std::collections::HashMap::new();
                changes.insert(
                    "power_storage".to_string(),
                    ComponentDelta::Modified(ComponentChange::PowerStorage {
                        charge_change: Some(0.4), // 40% (déchargée)
                    })
                );
                changes
            },
            timestamp: 1200,
            sequence: 3,
        };
        
        grid_delta_with_battery.add_block_delta(battery_delta);
        // ship.record_delta(grid_delta_with_battery);
        
        // === Synchronisation ===
        // Le serveur fusionne tout et envoie un seul delta au client
        // if let Some(merged) = ship.compute_and_apply_pending_deltas() {
        //     println!("GridDelta fusionné:");
        //     println!("  - Position: changée");
        //     println!("  - Velocity: changée");
        //     println!("  - Blocks changés: {}", merged.blocks_delta.len());
        //     println!("  - Taille totale: {} bytes", merged.estimated_size());
        //     
        //     // Un seul paquet réseau contient tout
        //     let serialized = serde_json::to_string(&merged).unwrap();
        //     // send_to_all_clients(serialized);
        // }
        
        println!("Exemple GridDelta: Position + 2 blocks changés = 1 delta réseau");
    }
    
    /// Exemple 3: Workflow typique serveur
    /// 
    /// Comment utiliser les deltas dans une boucle de jeu typique
    pub fn example_typical_server_workflow() {
        println!("\n=== Workflow typique serveur ===\n");
        
        // Tick 1: Début du frame
        println!("TICK 1 - Calcul de la physique");
        
        // 1. Les systèmes de jeu calculent les changements
        //    (physique, énergie, production, combat, etc.)
        
        // 2. Pour chaque entité qui change, on crée des deltas
        //    Au lieu de: block.current_integrity = 50.0;
        //    On fait:
        //    let delta = block.create_integrity_delta(50.0, current_time, sequence);
        //    block.record_delta(delta);
        
        println!("  - Système physique: 15 grilles ont bougé -> 15 GridDelta créés");
        println!("  - Système énergie: 200 blocks consomment -> 200 BlockDelta créés");
        println!("  - Système combat: 5 blocks endommagés -> 5 BlockDelta créés");
        
        // 3. À la fin du tick, on applique et fusionne
        println!("\nTICK 1 - Fin: Fusion des deltas");
        
        // Pour chaque grille:
        //   if let Some(delta) = grid.compute_and_apply_pending_deltas() {
        //       if !delta.is_empty() {
        //           network_queue.push(delta);
        //       }
        //   }
        
        println!("  - 15 GridDelta à envoyer (au lieu de 220 changements individuels)");
        
        // Tick 2-10: Les deltas s'accumulent si le réseau est lent
        println!("\nTICK 2-10 - Accumulation");
        println!("  - Le client #5 a du lag, on accumule ses deltas");
        println!("  - Tick 2: +1 delta");
        println!("  - Tick 3: +1 delta");
        println!("  - Tick 4: +1 delta");
        
        // Quand le client est prêt, on fusionne tout
        println!("\nTICK 11 - Client prêt");
        println!("  - Fusion de 3 deltas en 1 seul");
        println!("  - Envoi d'un gros paquet au lieu de 3 petits");
        println!("  - Bande passante économisée: ~60%");
        
        // Exemple de fusion:
        let delta1 = GridDelta::empty(1, 1000, 1);
        let delta2 = GridDelta::empty(1, 2000, 2);
        let delta3 = GridDelta::empty(1, 3000, 3);
        
        if let Some(merged) = GridDelta::merge(vec![delta1, delta2, delta3]) {
            println!("  - Delta fusionné: sequence {} -> {}", 1, merged.sequence);
        }
    }
    
    /// Exemple 4: Combat - plusieurs blocks endommagés simultanément
    pub fn example_combat_damage() {
        println!("\n=== Exemple: Combat ===\n");
        
        // Un missile frappe un vaisseau et endommage 10 blocks
        println!("Missile impact! 10 blocks endommagés");
        
        let mut grid_delta = GridDelta::empty(456, 5000, 1);
        
        // Pour chaque block touché, on ajoute son delta au GridDelta
        for block_id in 1..=10 {
            let damage_delta = BlockDelta {
                in_grid_id: block_id,
                integrity: Some(50.0), // Tous à 50% d'intégrité
                mass: None,
                component_changes: std::collections::HashMap::new(),
                timestamp: 5000,
                sequence: 1,
            };
            
            grid_delta.add_block_delta(damage_delta);
        }
        
        println!("  - 10 BlockDelta dans 1 GridDelta");
        println!("  - Taille: ~{} bytes", grid_delta.estimated_size());
        println!("  - Un seul paquet réseau pour tout synchroniser");
        
        // Le client reçoit ce delta et met à jour tous les blocks d'un coup
        // grid_delta.apply_to(&mut ship);
    }
}
