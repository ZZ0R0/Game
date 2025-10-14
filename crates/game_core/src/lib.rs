// Top-level library for the `game-core` crate.
// This file re-exports internal workspace crates to provide a simpler public surface.

// Re-export the nested `game-objects` crate so other crates can access it as
// `game_core::objects` instead of depending on it directly.
pub use game_objects as objects;

/// A small prelude for commonly used types from game-core
pub mod prelude {
	// game-objects may not expose names yet; avoid a noisy warning here
	#[allow(unused_imports)]
	pub use super::objects::*;
}

// Keep this file minimal; deeper module structure lives in sub-crates.
