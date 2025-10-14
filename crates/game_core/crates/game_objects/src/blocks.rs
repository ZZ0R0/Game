use crate::objects::Position;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RelPosition {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl RelPosition {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn zero() -> Self {
        Self::new(0, 0, 0)
    }

    pub fn to_world_position(&self, grid_position: &Position) -> Position {
        Position::new(
            grid_position.x + self.x as f32 * 2.5, // 2.5m per block
            grid_position.y + self.y as f32 * 2.5,
            grid_position.z + self.z as f32 * 2.5,
        )
    }
}

#[derive(Debug, Clone)]
pub struct RelOrientation {
    pub pitch: i32,
    pub yaw: i32,
    pub roll: i32,
}

impl RelOrientation {
    pub fn new(pitch: i32, yaw: i32, roll: i32) -> Self {
        Self { pitch, yaw, roll }
    }

    pub fn identity() -> Self {
        Self::new(0, 0, 0)
    }
}

#[derive(Debug, Clone)]
pub struct RelObject {
    pub position: RelPosition,
    pub orientation: RelOrientation,
    pub mass: f32,
}

impl RelObject {
    pub fn new(position: RelPosition, orientation: RelOrientation, mass: f32) -> Self {
        Self { position, orientation, mass }
    }

    pub fn default() -> Self {
        Self {
            position: RelPosition::zero(),
            orientation: RelOrientation::identity(),
            mass: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub name: String,
    pub integrity: f32,
    pub rel_object: RelObject,
}

impl Block {
    pub fn new(name: String, integrity: f32, rel_object: RelObject) -> Self {
        Self { name, integrity, rel_object }
    }

    pub fn default_armor() -> Self {
        Self {
            name: "Light Armor Block".to_string(),
            integrity: 100.0,
            rel_object: RelObject::default(),
        }
    }

    pub fn default_heavy_armor() -> Self {
        Self {
            name: "Heavy Armor Block".to_string(),
            integrity: 200.0,
            rel_object: RelObject::default(),
        }
    }
}

pub mod large_blocks {
    use super::{Block, RelObject, RelPosition, RelOrientation};

    #[derive(Debug, Clone)]
    pub struct LargeBlock {
        pub id: u32,
        pub name: String,
        pub block_type: String,
        pub block: Block,
    }

    impl LargeBlock {
        pub fn new(id: u32, name: String, block_type: String, block: Block) -> Self {
            Self { id, name, block_type, block }
        }

        pub fn default_armor(id: u32) -> Self {
            Self {
                id,
                name: "Large Armor Block".to_string(),
                block_type: "heavy_armor_block".to_string(),
                block: Block::default_heavy_armor(),
            }
        }

        pub fn light_armor_block(id: u32, position: RelPosition) -> Self {
            let rel_object = RelObject::new(position, RelOrientation::identity(), 500.0);
            let block = Block::new("Light Armor Block".to_string(), 100.0, rel_object);
            Self {
                id,
                name: "Light Armor Block".to_string(),
                block_type: "light_armor_block".to_string(),
                block,
            }
        }
    }
}

pub mod small_blocks {
    use super::Block;

    #[derive(Debug, Clone)]
    pub struct SmallBlock {
        pub id: u32,
        pub name: String,
        pub block: Block,
    }

    impl SmallBlock {
        pub fn new(id: u32, name: String, block: Block) -> Self {
            Self { id, name, block }
        }

        pub fn default_armor(id: u32) -> Self {
            Self {
                id,
                name: "Small Armor Block".to_string(),
                block: Block::default_armor(),
            }
        }
    }
}