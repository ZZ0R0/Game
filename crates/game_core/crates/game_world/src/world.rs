use crate::grids;
use crate::players;



#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Hash)]
pub struct World {
    pub seed : u64,
    pub name : String,
    pub grids : Vec<grids::Grid>,
    pub players : Vec<players::Player>,
    pub celestials : Vec<celestials::CelestialBody>,
    pub time : f64, // in seconds
}

impl World {
    pub fn new(seed: u64, name: String) -> Self {
        Self {
            seed,
            name,
            grids: Vec::new(),
            players: Vec::new(),
            celestials: Vec::new(),
            time: 0.0,
        };
    }

    pub fn delta(self) -> WorldDelta {
        let snapshot = WorldSnapshot {
            grids: Vec::new(),
            players: Vec::new(),
            celestials: Vec::new(),
            time: self.time,
        };
        for grid in grids.iter() {
            snapshot.grids.push(grid.snapshot());
        }
        for player in players.iter() {
            snapshot.players.push(player.snapshot());
        }
        for celestial in celestials.iter() {
            snapshot.celestials.push(celestial.snapshot());
        }
    }
}


pub struct WorldSnapshot { 
    pub grids: Vec<grids::Grid>,
    pub players: Vec<players::Player>,
    pub celestials: Vec<celestials::CelestialBody>,
    pub time: f64,
}