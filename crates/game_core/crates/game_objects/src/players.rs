use crate::objects::{PhysicalObject, Position, Orientation};

#[derive(Debug, Clone)]
pub struct PlayerId(pub u32);

#[derive(Debug, Clone)]
pub struct Player {
    pub id: PlayerId,
    pub name: String,
    pub physical: PhysicalObject,
    pub health: f32,
    pub oxygen: f32,
    pub hydrogen: f32,
    pub energy: f32,
}

impl Player {
    pub fn new(id: u32, name: String) -> Self {
        Self {
            id: PlayerId(id),
            name,
            physical: PhysicalObject::default(),
            health: 100.0,
            oxygen: 100.0,
            hydrogen: 100.0,
            energy: 100.0,
        }
    }

    pub fn spawn_at(&mut self, position: Position, orientation: Orientation) {
        self.physical.placed.position = position;
        self.physical.placed.orientation = orientation;
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    pub fn take_damage(&mut self, damage: f32) {
        self.health = (self.health - damage).max(0.0);
    }

    pub fn heal(&mut self, amount: f32) {
        self.health = (self.health + amount).min(100.0);
    }

    /// Update player resources (oxygen, hydrogen, energy) over time
    pub fn update_resources(&mut self, delta_time: f32) {
        // Consume oxygen over time (1% per second when not in atmosphere)
        self.oxygen = (self.oxygen - delta_time).max(0.0);
        
        // Energy drains slowly (0.5% per second)
        self.energy = (self.energy - delta_time * 0.5).max(0.0);
        
        // Take damage if oxygen is too low
        if self.oxygen <= 0.0 {
            self.take_damage(delta_time * 10.0); // 10 damage per second without oxygen
        }
    }

    /// Refill oxygen (e.g., when in pressurized area)
    pub fn refill_oxygen(&mut self) {
        self.oxygen = 100.0;
    }

    /// Refill energy (e.g., when near power source)
    pub fn refill_energy(&mut self) {
        self.energy = 100.0;
    }

    /// Check if player needs critical resources
    pub fn needs_oxygen(&self) -> bool {
        self.oxygen < 20.0
    }

    pub fn needs_energy(&self) -> bool {
        self.energy < 20.0
    }

    /// Move player to a new position
    pub fn move_to(&mut self, position: Position) {
        self.physical.placed.position = position;
    }

    /// Get player's current position
    pub fn get_position(&self) -> &Position {
        &self.physical.placed.position
    }
}