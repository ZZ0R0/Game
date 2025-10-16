/// Blocks d'armure et de structure
use crate::blocks::{BlockDef, BlockId, Model3DRef};

pub fn create_all() -> Vec<BlockDef> {
    vec![
        create_light_armor_large(),
        create_heavy_armor_large(),
        create_light_armor_small(),
        create_heavy_armor_small(),
    ]
}

/// Light Armor Block (Large Grid)
pub fn create_light_armor_large() -> BlockDef {
    let mut def = BlockDef::new(
        BlockId::Large(1),
        "Light Armor Block",
        (1, 1, 1),
        300.0,      // masse en kg
        400.0,      // intégrité
        "Armor",
        Model3DRef::new("models/blocks/armor_light_large.glb"),
    );
    
    // Toutes les faces sont connectables (cube plein)
    def.set_full_cube_mounts();
    
    def
}

/// Heavy Armor Block (Large Grid)
pub fn create_heavy_armor_large() -> BlockDef {
    let mut def = BlockDef::new(
        BlockId::Large(2),
        "Heavy Armor Block",
        (1, 1, 1),
        1000.0,     // masse en kg
        1500.0,     // intégrité
        "Armor",
        Model3DRef::new("models/blocks/armor_heavy_large.glb"),
    );
    
    def.set_full_cube_mounts();
    
    def
}

/// Light Armor Block (Small Grid)
pub fn create_light_armor_small() -> BlockDef {
    let mut def = BlockDef::new(
        BlockId::Small(1),
        "Light Armor Block",
        (1, 1, 1),
        15.0,       // masse en kg
        50.0,       // intégrité
        "Armor",
        Model3DRef::new("models/blocks/armor_light_small.glb"),
    );
    
    def.set_full_cube_mounts();
    
    def
}

/// Heavy Armor Block (Small Grid)
pub fn create_heavy_armor_small() -> BlockDef {
    let mut def = BlockDef::new(
        BlockId::Small(2),
        "Heavy Armor Block",
        (1, 1, 1),
        50.0,       // masse en kg
        200.0,      // intégrité
        "Armor",
        Model3DRef::new("models/blocks/armor_heavy_small.glb"),
    );
    
    def.set_full_cube_mounts();
    
    def
}
