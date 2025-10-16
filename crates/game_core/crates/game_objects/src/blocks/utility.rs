/// Blocks utilitaires (cargo, connecteurs, etc.)
use crate::blocks::{BlockDef, BlockId, Model3DRef};

pub fn create_all() -> Vec<BlockDef> {
    vec![
        create_cargo_container_large(),
        create_cargo_container_small(),
        create_connector_large(),
    ]
}

/// Large Cargo Container (Large Grid)
/// Stocke beaucoup d'items
pub fn create_cargo_container_large() -> BlockDef {
    let mut def = BlockDef::new(
        BlockId::Large(600),
        "Large Cargo Container",
        (1, 1, 1),
        1000.0,
        500.0,
        "Cargo",
        Model3DRef::new("models/blocks/cargo_large.glb"),
    );
    
    def.set_full_cube_mounts();
    
    // Juste un inventaire, pas d'autres composants
    def = def.with_component("inventory");
    
    def
}

/// Small Cargo Container (Large Grid)
pub fn create_cargo_container_small() -> BlockDef {
    let mut def = BlockDef::new(
        BlockId::Large(601),
        "Small Cargo Container",
        (1, 1, 1),
        500.0,
        300.0,
        "Cargo",
        Model3DRef::new("models/blocks/cargo_small.glb"),
    );
    
    def.set_full_cube_mounts();
    def = def.with_component("inventory");
    
    def
}

/// Connector (Large Grid)
/// Permet de connecter deux vaisseaux et transférer des ressources
pub fn create_connector_large() -> BlockDef {
    let mut def = BlockDef::new(
        BlockId::Large(602),
        "Connector",
        (1, 1, 1),
        300.0,
        400.0,
        "Utility",
        Model3DRef::new("models/blocks/connector_large.glb"),
    );
    
    def.set_full_cube_mounts();
    
    // Le connecteur a un inventaire et consomme de l'énergie
    def = def
        .with_component("inventory")
        .with_component("power_consumer");
    
    def
}
