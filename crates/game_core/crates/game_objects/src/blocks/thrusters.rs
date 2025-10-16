/// Blocks de propulsion (thrusters)
use crate::blocks::{BlockDef, BlockId, Model3DRef};

pub fn create_all() -> Vec<BlockDef> {
    vec![
        create_large_thruster(),
        create_small_thruster(),
    ]
}

/// Large Thruster (Large Grid)
pub fn create_large_thruster() -> BlockDef {
    let mut def = BlockDef::new(
        BlockId::Large(300),
        "Large Thruster",
        (1, 1, 2),  // 1x1x2 blocks
        1200.0,
        500.0,
        "Thruster",
        Model3DRef::new("models/blocks/thruster_large.glb"),
    );
    
    def.set_full_cube_mounts();
    
    def = def
        .with_component("thruster")
        .with_component("power_consumer");
    
    def
}

/// Small Thruster (Small Grid)
pub fn create_small_thruster() -> BlockDef {
    let mut def = BlockDef::new(
        BlockId::Small(300),
        "Small Thruster",
        (1, 1, 1),
        80.0,
        120.0,
        "Thruster",
        Model3DRef::new("models/blocks/thruster_small.glb"),
    );
    
    def.set_full_cube_mounts();
    
    def = def
        .with_component("thruster")
        .with_component("power_consumer");
    
    def
}
