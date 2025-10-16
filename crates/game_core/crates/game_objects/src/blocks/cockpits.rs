/// Blocks de cockpit et contrôle
use crate::blocks::{BlockDef, BlockId, Model3DRef, BlockFace, MountPoint};

pub fn create_all() -> Vec<BlockDef> {
    vec![
        create_cockpit_large(),
        create_cockpit_small(),
        create_flight_seat_large(),
    ]
}

/// Cockpit (Large Grid)
/// Permet de piloter le vaisseau et a plusieurs composants
pub fn create_cockpit_large() -> BlockDef {
    let mut def = BlockDef::new(
        BlockId::Large(200),
        "Cockpit",
        (1, 1, 1),
        400.0,
        350.0,
        "Control",
        Model3DRef::new("models/blocks/cockpit_large.glb"),
    );
    
    // Le cockpit a des mount points limités (pas toutes les faces)
    def.add_mount_point(BlockFace::Back, MountPoint::full_face());
    def.add_mount_point(BlockFace::Left, MountPoint::full_face());
    def.add_mount_point(BlockFace::Right, MountPoint::full_face());
    def.add_mount_point(BlockFace::Bottom, MountPoint::full_face());
    
    // Le cockpit a plusieurs composants
    def = def
        .with_component("control")          // Permet de piloter
        .with_component("power_consumer")   // Consomme de l'énergie
        .with_component("inventory");       // Petit inventaire pour le pilote
    
    def
}

/// Cockpit (Small Grid)
pub fn create_cockpit_small() -> BlockDef {
    let mut def = BlockDef::new(
        BlockId::Small(200),
        "Cockpit",
        (1, 1, 1),
        50.0,
        80.0,
        "Control",
        Model3DRef::new("models/blocks/cockpit_small.glb"),
    );
    
    def.add_mount_point(BlockFace::Back, MountPoint::full_face());
    def.add_mount_point(BlockFace::Left, MountPoint::full_face());
    def.add_mount_point(BlockFace::Right, MountPoint::full_face());
    def.add_mount_point(BlockFace::Bottom, MountPoint::full_face());
    
    def = def
        .with_component("control")
        .with_component("power_consumer")
        .with_component("inventory");
    
    def
}

/// Flight Seat (Large Grid)
/// Siège de vol simple, moins de fonctionnalités que le cockpit
pub fn create_flight_seat_large() -> BlockDef {
    let mut def = BlockDef::new(
        BlockId::Large(201),
        "Flight Seat",
        (1, 1, 1),
        150.0,
        200.0,
        "Control",
        Model3DRef::new("models/blocks/flight_seat_large.glb"),
    );
    
    def.set_full_cube_mounts();
    
    // Juste le contrôle, pas d'inventaire
    def = def.with_component("control");
    
    def
}
