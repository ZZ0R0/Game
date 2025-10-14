use crate::objects::PhysicalObject;
use crate::blocks::{large_blocks::LargeBlock, small_blocks::SmallBlock, RelPosition};

#[derive(Debug, Clone)]
pub struct GridId(pub u32);

#[derive(Debug, Clone)]
pub struct Grid {
    pub id: GridId,
    pub name: String,
    pub physical: PhysicalObject,
    pub size: u32,
}

impl Grid {
    pub fn new(id: u32, name: String, physical: PhysicalObject, size: u32) -> Self {
        Self {
            id: GridId(id),
            name,
            physical,
            size,
        }
    }

    pub fn default(id: u32) -> Self {
        Self {
            id: GridId(id),
            name: "Default Grid".to_string(),
            physical: PhysicalObject::default(),
            size: 1,
        }
    }
}

pub mod large_grids {
    use super::*;

    #[derive(Debug, Clone)]
    pub struct LargeGrid {
        pub grid: Grid,
        pub large_blocks: Vec<LargeBlock>,
    }

    impl LargeGrid {
        pub fn new(grid: Grid) -> Self {
            Self {
                grid,
                large_blocks: Vec::new(),
            }
        }

        pub fn default_ship(id: u32) -> Self {
            let mut grid = Grid::default(id);
            grid.name = "Default Ship".to_string();
            
            let mut large_grid = Self::new(grid);
            
            // Add a single light armor block at origin
            let light_armor = LargeBlock::light_armor_block(0, RelPosition::zero());
            large_grid.add_block(light_armor);
            
            large_grid
        }

        pub fn add_block(&mut self, block: LargeBlock) {
            self.large_blocks.push(block);
            self.grid.size = self.large_blocks.len() as u32;
        }

        pub fn remove_block(&mut self, block_id: u32) -> Option<LargeBlock> {
            if let Some(pos) = self.large_blocks.iter().position(|b| b.id == block_id) {
                let block = self.large_blocks.remove(pos);
                self.grid.size = self.large_blocks.len() as u32;
                Some(block)
            } else {
                None
            }
        }

        /// Get the total mass of the grid
        pub fn total_mass(&self) -> f32 {
            let blocks_mass: f32 = self.large_blocks.iter()
                .map(|block| block.block.rel_object.mass)
                .sum();
            self.grid.physical.mass + blocks_mass
        }

        /// Move the grid to a new position
        pub fn set_position(&mut self, position: crate::objects::Position) {
            self.grid.physical.placed.position = position;
        }

        /// Get current position
        pub fn get_position(&self) -> &crate::objects::Position {
            &self.grid.physical.placed.position
        }

        /// Apply damage to a specific block
        pub fn damage_block(&mut self, block_id: u32, damage: f32) -> bool {
            if let Some(block) = self.large_blocks.iter_mut().find(|b| b.id == block_id) {
                block.block.integrity -= damage;
                if block.block.integrity <= 0.0 {
                    self.remove_block(block_id);
                    true // Block destroyed
                } else {
                    false // Block damaged but not destroyed
                }
            } else {
                false // Block not found
            }
        }

        /// Get next available block ID
        pub fn next_block_id(&self) -> u32 {
            self.large_blocks.iter()
                .map(|block| block.id)
                .max()
                .unwrap_or(0) + 1
        }
    }
}

pub mod small_grids {
    use super::*;

    #[derive(Debug, Clone)]
    pub struct SmallGrid {
        pub grid: Grid,
        pub small_blocks: Vec<SmallBlock>,
    }

    impl SmallGrid {
        pub fn new(grid: Grid) -> Self {
            Self {
                grid,
                small_blocks: Vec::new(),
            }
        }

        pub fn default_ship(id: u32) -> Self {
            let mut grid = Grid::default(id);
            grid.name = "Default Small Ship".to_string();
            
            let mut small_grid = Self::new(grid);
            
            // Add a single default armor block
            let default_block = SmallBlock::default_armor(0);
            small_grid.add_block(default_block);
            
            small_grid
        }

        pub fn add_block(&mut self, block: SmallBlock) {
            self.small_blocks.push(block);
            self.grid.size = self.small_blocks.len() as u32;
        }

        pub fn remove_block(&mut self, block_id: u32) -> Option<SmallBlock> {
            if let Some(pos) = self.small_blocks.iter().position(|b| b.id == block_id) {
                let block = self.small_blocks.remove(pos);
                self.grid.size = self.small_blocks.len() as u32;
                Some(block)
            } else {
                None
            }
        }
    }
}