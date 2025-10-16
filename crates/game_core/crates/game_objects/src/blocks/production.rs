/// Blocks de production (assembleur, raffinerie, etc.)
use crate::blocks::{BlockDef, BlockId, Model3DRef};

pub fn create_all() -> Vec<BlockDef> {
    vec![
        create_assembler_large(),
        create_refinery_large(),
    ]
}

/// Assembler (Large Grid)
/// Fabrique des composants à partir de matériaux
pub fn create_assembler_large() -> BlockDef {
    let mut def = BlockDef::new(
        BlockId::Large(500),
        "Assembler",
        (1, 1, 1),
        1200.0,
        500.0,
        "Production",
        Model3DRef::new("models/blocks/assembler_large.glb"),
    );
    
    def.set_full_cube_mounts();
    
    // L'assembleur a plusieurs composants
    def = def
        .with_component("producer")        // Produit des items
        .with_component("power_consumer")  // Consomme de l'énergie
        .with_component("inventory");      // Stocke input/output
    
    def
}

/// Refinery (Large Grid)
/// Raffine les minerais en matériaux utilisables
pub fn create_refinery_large() -> BlockDef {
    let mut def = BlockDef::new(
        BlockId::Large(501),
        "Refinery",
        (1, 1, 2),  // Plus grand
        2000.0,
        600.0,
        "Production",
        Model3DRef::new("models/blocks/refinery_large.glb"),
    );
    
    def.set_full_cube_mounts();
    
    def = def
        .with_component("producer")
        .with_component("power_consumer")
        .with_component("inventory");
    
    def
}
