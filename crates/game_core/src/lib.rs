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

/// Ship factory module for creating default ships and game entities
pub mod factory {
	use crate::objects::{LargeGrid, Position, Player};

	/// Creates a default ship with a single large grid block
	pub fn create_default_ship(ship_id: u32) -> LargeGrid {
		LargeGrid::default_ship(ship_id)
	}

	/// Creates a ship at a specific position
	pub fn create_ship_at_position(ship_id: u32, position: Position) -> LargeGrid {
		let mut ship = LargeGrid::default_ship(ship_id);
		ship.grid.physical.placed.position = position;
		ship
	}

	/// Creates a basic starter ship with better stats
	pub fn create_starter_ship(ship_id: u32, player_name: &str) -> LargeGrid {
		let mut ship = LargeGrid::default_ship(ship_id);
		ship.grid.name = format!("{}'s Starter Ship", player_name);
		ship.grid.physical.mass = 1000.0; // 1 ton
		ship
	}

	/// Creates a new player with default stats
	pub fn create_player(player_id: u32, name: &str) -> Player {
		Player::new(player_id, name.to_string())
	}

	/// Creates a player spawned at a specific position
	pub fn spawn_player_at(player_id: u32, name: &str, position: Position) -> Player {
		let mut player = Player::new(player_id, name.to_string());
		player.move_to(position);
		player
	}
}

/// Simple game world that manages players and ships
pub mod world {
	use std::collections::HashMap;
	use crate::objects::{Player, LargeGrid, Position};

	pub struct GameWorld {
		pub players: HashMap<u32, Player>,
		pub ships: HashMap<u32, LargeGrid>,
		next_player_id: u32,
		next_ship_id: u32,
	}

	impl GameWorld {
		pub fn new() -> Self {
			Self {
				players: HashMap::new(),
				ships: HashMap::new(),
				next_player_id: 1,
				next_ship_id: 1,
			}
		}

		/// Add a new player to the world
		pub fn add_player(&mut self, name: &str) -> u32 {
			let player_id = self.next_player_id;
			self.next_player_id += 1;
			
			let player = crate::factory::create_player(player_id, name);
			self.players.insert(player_id, player);
			
			player_id
		}

		/// Add a new ship to the world
		pub fn add_ship(&mut self, owner_name: &str) -> u32 {
			let ship_id = self.next_ship_id;
			self.next_ship_id += 1;
			
			let ship = crate::factory::create_starter_ship(ship_id, owner_name);
			self.ships.insert(ship_id, ship);
			
			ship_id
		}

		/// Spawn a player with their starter ship
		pub fn spawn_new_player(&mut self, name: &str, spawn_position: Position) -> (u32, u32) {
			let player_id = self.add_player(name);
			let ship_id = self.add_ship(name);
			
			// Position player and ship at spawn location
			if let Some(player) = self.players.get_mut(&player_id) {
				player.move_to(spawn_position.clone());
			}
			
			if let Some(ship) = self.ships.get_mut(&ship_id) {
				ship.set_position(Position::new(spawn_position.x + 10.0, spawn_position.y, spawn_position.z));
			}
			
			(player_id, ship_id)
		}

		/// Update all players (resources, health, etc.)
		pub fn update(&mut self, delta_time: f32) {
			for player in self.players.values_mut() {
				player.update_resources(delta_time);
			}
		}
	}
}

// Keep this file minimal; deeper module structure lives in sub-crates.
