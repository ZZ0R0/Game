/// Blocks d'armes
use crate::blocks::{BlockDef, BlockId, Model3DRef};

pub fn create_all() -> Vec<BlockDef> {
    vec![
        create_gatling_turret_large(),
        create_missile_launcher_large(),
    ]
}

/// Gatling Turret (Large Grid)
pub fn create_gatling_turret_large() -> BlockDef {
    let mut def = BlockDef::new(
        BlockId::Large(400),
        "Gatling Turret",
        (1, 1, 1),
        800.0,
        400.0,
        "Weapon",
        Model3DRef::new("models/blocks/gatling_turret_large.glb"),
    );
    
    def.set_full_cube_mounts();
    
    def = def
        .with_component("weapon")
        .with_component("power_consumer")
        .with_component("inventory");  // Pour stocker les munitions
    
    def
}

/// Missile Launcher (Large Grid)
pub fn create_missile_launcher_large() -> BlockDef {
    let mut def = BlockDef::new(
        BlockId::Large(401),
        "Missile Launcher",
        (1, 1, 2),
        1500.0,
        500.0,
        "Weapon",
        Model3DRef::new("models/blocks/missile_launcher_large.glb"),
    );
    
    def.set_full_cube_mounts();
    
    def = def
        .with_component("weapon")
        .with_component("power_consumer")
        .with_component("inventory");
    
    def
}
