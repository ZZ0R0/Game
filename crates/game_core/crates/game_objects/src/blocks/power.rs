/// Blocks de production et stockage d'énergie
use crate::blocks::{BlockDef, BlockId, Model3DRef};

pub fn create_all() -> Vec<BlockDef> {
    vec![
        create_battery_large(),
        create_battery_small(),
        create_reactor_large(),
        create_reactor_small(),
        create_solar_panel_large(),
    ]
}

/// Battery Block (Large Grid)
/// Ce block peut stocker et distribuer de l'énergie
pub fn create_battery_large() -> BlockDef {
    let mut def = BlockDef::new(
        BlockId::Large(100),
        "Battery Block",
        (1, 1, 1),
        800.0,
        400.0,
        "Power",
        Model3DRef::new("models/blocks/battery_large.glb"),
    );
    
    def.set_full_cube_mounts();
    
    // Déclarer que ce block supporte le composant PowerStorage
    def = def
        .with_component("power_storage")
        .with_component("power_consumer"); // Peut aussi consommer pour se recharger
    
    def
}

/// Battery Block (Small Grid)
pub fn create_battery_small() -> BlockDef {
    let mut def = BlockDef::new(
        BlockId::Small(100),
        "Battery Block",
        (1, 1, 1),
        50.0,
        80.0,
        "Power",
        Model3DRef::new("models/blocks/battery_small.glb"),
    );
    
    def.set_full_cube_mounts();
    def = def.with_component("power_storage").with_component("power_consumer");
    
    def
}

/// Large Reactor (Large Grid)
/// Produit de l'énergie en consommant du carburant
pub fn create_reactor_large() -> BlockDef {
    let mut def = BlockDef::new(
        BlockId::Large(101),
        "Large Reactor",
        (1, 1, 1),
        2500.0,
        600.0,
        "Power",
        Model3DRef::new("models/blocks/reactor_large.glb"),
    );
    
    def.set_full_cube_mounts();
    
    // Le réacteur a plusieurs composants
    def = def
        .with_component("power_producer")  // Produit de l'énergie
        .with_component("inventory");      // Stocke le carburant
    
    def
}

/// Small Reactor (Small Grid)
pub fn create_reactor_small() -> BlockDef {
    let mut def = BlockDef::new(
        BlockId::Small(101),
        "Small Reactor",
        (1, 1, 1),
        150.0,
        100.0,
        "Power",
        Model3DRef::new("models/blocks/reactor_small.glb"),
    );
    
    def.set_full_cube_mounts();
    def = def.with_component("power_producer").with_component("inventory");
    
    def
}

/// Solar Panel (Large Grid)
/// Produit de l'énergie à partir du soleil
pub fn create_solar_panel_large() -> BlockDef {
    let mut def = BlockDef::new(
        BlockId::Large(102),
        "Solar Panel",
        (1, 1, 1),
        150.0,
        200.0,
        "Power",
        Model3DRef::new("models/blocks/solar_panel_large.glb"),
    );
    
    def.set_full_cube_mounts();
    
    // Produit simplement de l'énergie
    def = def.with_component("power_producer");
    
    def
}
