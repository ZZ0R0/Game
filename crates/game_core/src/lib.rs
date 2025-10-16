// Top-level library for the `game-core` crate.
// This file re-exports internal workspace crates to provide a simpler public surface.

// Re-export the nested `game-objects` crate so other crates can access it as
// `game_core::objects` instead of depending on it directly.
pub use game_objects as objects;
pub use game_world as world;