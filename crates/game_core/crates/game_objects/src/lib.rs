// Game objects library - contains all the core game object structures

pub mod blocks;
pub mod celestials;
pub mod grids;
pub mod humanoids;
pub mod items;
pub mod objects;
pub mod players;
pub mod volume;

// Re-export commonly used types
pub use blocks::{Block, RelObject, RelPosition, RelOrientation, large_blocks::LargeBlock, small_blocks::SmallBlock};
pub use grids::{Grid, GridId, large_grids::LargeGrid, small_grids::SmallGrid};
pub use objects::{Position, Velocity, Acceleration, Orientation, PlacedObject, PhysicalObject};
pub use volume::Volume;
pub use players::{Player, PlayerId};
pub use items::{Item, ItemId, ItemStack};
pub use celestials::{CelestialBody, CelestialId, CelestialType};
pub use humanoids::{humanoid::Humanoid, humanoid::human::Human};
