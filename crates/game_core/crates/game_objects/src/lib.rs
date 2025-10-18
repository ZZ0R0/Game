// Game objects library - contains all the core game object structures

pub mod blocks;
pub mod celestials;
pub mod grids;
pub mod humanoids;
pub mod inventory;
pub mod items;
pub mod objects;
pub mod players;
pub mod volume;

// Re-export commonly used types
pub use blocks::{Block, BlockDelta, ComponentDelta, ComponentChange};
pub use grids::{Grid, GridId, GridDelta};
pub use objects::{FloatPosition, IntPosition, Velocity, Acceleration, FloatOrientation, IntOrientation, PlacedObject, PhysicalObject};
pub use volume::Volume;
pub use players::{Player, PlayerId, PlayerDelta};
pub use items::{Item, ItemId, ItemStack};
pub use inventory::{Inventory};
pub use celestials::{CelestialBody, CelestialId, CelestialType, CelestialDelta};
pub use humanoids::{humanoid::Humanoid, humanoid::human::Human};