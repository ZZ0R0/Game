/// Exemples d'utilisation des deltas pour Players et CelestialBody
/// 
/// Ce fichier démontre comment utiliser les deltas pour :
/// 1. Synchroniser les joueurs (position, ressources)
/// 2. Synchroniser les corps célestes (orbites, rotation)

#[allow(dead_code)]
mod examples {
    use crate::players::{Player, PlayerDelta};
    use crate::celestials::{CelestialBody, CelestialDelta};
    use crate::objects::{FloatPosition, FloatOrientation, Velocity};

    /// Exemple 1: Synchronisation d'un joueur en mouvement
    /// 
    /// Scénario: Un joueur se déplace et consomme de l'oxygène
    pub fn example_player_movement_and_resources() {
        println!("\n=== Exemple: Joueur en mouvement ===\n");
        
        // Supposons qu'on a un joueur
        // let mut player = Player::new(1, "Astronaut".to_string());
        
        // Tick 1: Le joueur se déplace
        let movement_delta = PlayerDelta {
            player_id: 1,
            position: Some(FloatPosition::new(100.0, 50.0, 200.0)),
            orientation: Some(FloatOrientation::new(0.0, 0.0, 0.0)), // Pas de rotation
            velocity: Some(Velocity::new(5.0, 0.0, 2.0)),
            health: None,
            oxygen: None,
            hydrogen: None,
            energy: None,
            timestamp: 1000,
            sequence: 1,
        };
        
        // player.record_delta(movement_delta);
        
        // Tick 2: L'oxygène diminue
        let resource_delta = PlayerDelta {
            player_id: 1,
            position: None,
            orientation: None,
            velocity: None,
            health: None,
            oxygen: Some(95.0), // 95%
            hydrogen: None,
            energy: Some(98.0), // 98%
            timestamp: 2000,
            sequence: 2,
        };
        
        // player.record_delta(resource_delta);
        
        // Tick 3: Encore plus d'oxygène consommé
        let resource_delta2 = PlayerDelta {
            player_id: 1,
            position: None,
            orientation: None,
            velocity: None,
            health: None,
            oxygen: Some(90.0), // 90%
            hydrogen: None,
            energy: Some(96.0), // 96%
            timestamp: 3000,
            sequence: 3,
        };
        
        // player.record_delta(resource_delta2);
        
        println!("Joueur: 3 deltas enregistrés");
        println!("  - Tick 1: Position changée");
        println!("  - Tick 2: Oxygène 95%, Énergie 98%");
        println!("  - Tick 3: Oxygène 90%, Énergie 96%");
        
        // Synchronisation réseau
        // if let Some(merged) = player.compute_and_apply_pending_deltas() {
        //     println!("\nDelta fusionné:");
        //     println!("  - Position finale: (100, 50, 200)");
        //     println!("  - Oxygène final: 90%");
        //     println!("  - Énergie finale: 96%");
        //     println!("  - Taille: {} bytes", merged.estimated_size());
        // }
        
        // Fusion des deltas
        let merged = PlayerDelta::merge(vec![
            movement_delta,
            resource_delta,
            resource_delta2,
        ]);
        
        if let Some(delta) = merged {
            println!("\nDelta fusionné envoyé au réseau:");
            println!("  - Contient position finale ET ressources finales");
            println!("  - Un seul paquet au lieu de 3");
        }
    }
    
    /// Exemple 2: Joueur en combat
    /// 
    /// Scénario: Un joueur prend des dégâts et perd de la santé
    pub fn example_player_combat() {
        println!("\n=== Exemple: Joueur en combat ===\n");
        
        // Tick 1: Le joueur prend des dégâts
        let damage_delta = PlayerDelta {
            player_id: 1,
            position: None,
            orientation: None,
            velocity: None,
            health: Some(80.0), // -20 HP
            oxygen: None,
            hydrogen: None,
            energy: None,
            timestamp: 1000,
            sequence: 1,
        };
        
        println!("Tick 1: Joueur touché! HP: 100 -> 80");
        
        // Tick 2: Encore des dégâts
        let damage_delta2 = PlayerDelta {
            player_id: 1,
            position: None,
            orientation: None,
            velocity: None,
            health: Some(60.0), // -20 HP
            oxygen: None,
            hydrogen: None,
            energy: None,
            timestamp: 2000,
            sequence: 2,
        };
        
        println!("Tick 2: Encore touché! HP: 80 -> 60");
        
        // Tick 3: Le joueur se soigne
        let heal_delta = PlayerDelta {
            player_id: 1,
            position: None,
            orientation: None,
            velocity: None,
            health: Some(75.0), // +15 HP
            oxygen: None,
            hydrogen: None,
            energy: None,
            timestamp: 3000,
            sequence: 3,
        };
        
        println!("Tick 3: Soigné! HP: 60 -> 75");
        
        // Fusion
        let merged = PlayerDelta::merge(vec![damage_delta, damage_delta2, heal_delta]);
        
        if let Some(delta) = merged {
            println!("\nDelta fusionné:");
            println!("  - HP final: 75 (au lieu de 3 mises à jour séparées)");
            println!("  - Taille: {} bytes", delta.estimated_size());
        }
    }
    
    /// Exemple 3: Corps céleste en orbite
    /// 
    /// Scénario: Une planète tourne sur elle-même et orbite autour d'une étoile
    pub fn example_celestial_orbit() {
        println!("\n=== Exemple: Planète en orbite ===\n");
        
        // Supposons qu'on a une planète
        // let mut planet = CelestialBody::earth_like_planet(1);
        
        // Tick 1: La planète se déplace sur son orbite
        let orbit_delta = CelestialDelta {
            celestial_id: 1,
            position: Some(FloatPosition::new(1000000.0, 0.0, 0.0)),
            orientation: None,
            velocity: Some(Velocity::new(0.0, 0.0, 30000.0)), // 30 km/s
            acceleration: None,
            timestamp: 1000,
            sequence: 1,
        };
        
        // planet.record_delta(orbit_delta);
        
        // Tick 2: La planète tourne sur elle-même
        let rotation_delta = CelestialDelta {
            celestial_id: 1,
            position: None,
            orientation: Some(FloatOrientation::new(0.0, 0.1, 0.0)), // Quaternion
            velocity: None,
            acceleration: None,
            timestamp: 2000,
            sequence: 2,
        };
        
        // planet.record_delta(rotation_delta);
        
        // Tick 3: Continue sur l'orbite
        let orbit_delta2 = CelestialDelta {
            celestial_id: 1,
            position: Some(FloatPosition::new(999000.0, 0.0, 30000.0)),
            orientation: None,
            velocity: Some(Velocity::new(0.0, 0.0, 30000.0)),
            acceleration: None,
            timestamp: 3000,
            sequence: 3,
        };
        
        // planet.record_delta(orbit_delta2);
        
        println!("Planète: 3 deltas enregistrés");
        println!("  - Tick 1: Position orbitale");
        println!("  - Tick 2: Rotation sur elle-même");
        println!("  - Tick 3: Nouvelle position orbitale");
        
        // Fusion
        let merged = CelestialDelta::merge(vec![
            orbit_delta,
            rotation_delta,
            orbit_delta2,
        ]);
        
        if let Some(delta) = merged {
            println!("\nDelta fusionné:");
            println!("  - Position finale + Rotation finale");
            println!("  - Taille: {} bytes", delta.estimated_size());
            println!("  - Synchronise mouvement orbital et rotation");
        }
    }
    
    /// Exemple 4: Système complet - Plusieurs entités
    /// 
    /// Scénario: Un système solaire avec planète + joueur
    pub fn example_full_system() {
        println!("\n=== Exemple: Système complet ===\n");
        
        // Dans un tick de jeu typique:
        
        // 1. Mise à jour des corps célestes (physique orbitale)
        println!("Physique céleste:");
        println!("  - Planète #1: nouveau delta (orbite)");
        println!("  - Lune #2: nouveau delta (orbite)");
        println!("  - Astéroïde #3: nouveau delta (dérive)");
        
        // 2. Mise à jour des joueurs
        println!("\nJoueurs:");
        println!("  - Player #1: nouveau delta (mouvement + ressources)");
        println!("  - Player #2: nouveau delta (ressources seulement)");
        println!("  - Player #3: nouveau delta (combat)");
        
        // 3. À la fin du tick, fusion et envoi
        println!("\nSynchronisation réseau:");
        println!("  - 3 CelestialDelta fusionnés -> envoyés");
        println!("  - 3 PlayerDelta fusionnés -> envoyés");
        println!("  - Total: 6 paquets réseau pour tout le système");
        
        // Au lieu de:
        println!("\nSans delta (pour comparaison):");
        println!("  - État complet de 3 planètes: ~3 KB");
        println!("  - État complet de 3 joueurs: ~2 KB");
        println!("  - Total: 5 KB par tick");
        
        println!("\nAvec delta:");
        println!("  - 3 CelestialDelta: ~300 bytes");
        println!("  - 3 PlayerDelta: ~200 bytes");
        println!("  - Total: 500 bytes par tick (90% d'économie!)");
    }
    
    /// Exemple 5: Gestion du lag client
    /// 
    /// Scénario: Un client a du lag, on accumule les deltas
    pub fn example_client_lag() {
        println!("\n=== Exemple: Gestion du lag ===\n");
        
        // Le client #5 a du lag pendant 5 secondes
        println!("Client #5: Lag détecté (5 secondes)");
        
        // Tick 1-50: On accumule les deltas du joueur
        let mut accumulated_deltas = Vec::new();
        
        for tick in 1..=50 {
            let delta = PlayerDelta {
                player_id: 5,
                position: Some(FloatPosition::new(
                    100.0 + tick as f32,
                    50.0,
                    200.0,
                )),
                orientation: None,
                velocity: None,
                health: None,
                oxygen: Some(100.0 - tick as f32 * 0.1), // Oxygène diminue
                hydrogen: None,
                energy: None,
                timestamp: tick * 100,
                sequence: tick,
            };
            
            accumulated_deltas.push(delta);
        }
        
        println!("  - 50 deltas accumulés en mémoire");
        
        // Quand le client est de retour
        println!("\nClient #5: Retour en ligne");
        
        let merged = PlayerDelta::merge(accumulated_deltas);
        
        if let Some(delta) = merged {
            println!("  - 50 deltas fusionnés en 1 seul");
            println!("  - Position finale: (150, 50, 200)");
            println!("  - Oxygène final: 95%");
            println!("  - Envoi d'un seul gros paquet");
            println!("  - Taille: {} bytes (au lieu de ~5000 bytes)", delta.estimated_size());
        }
        
        println!("\nAvantages:");
        println!("  - Le client rattrape son retard instantanément");
        println!("  - Pas de spam de 50 paquets");
        println!("  - Bande passante optimisée");
    }
}
