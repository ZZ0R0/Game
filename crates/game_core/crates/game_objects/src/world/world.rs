// world.rs — un monde possède son instance d’arènes
use crate::utils::arenas::{Arenas, SharedArenas};
use std::sync::{Arc, RwLock};

pub struct World {
    pub seed: u64,
    pub name: String,
    pub time: f64,
    pub arenas: SharedArenas,
}

impl World {
    #[inline]
    pub fn new(seed: u64, name: String) -> Self {
        Self {
            seed,
            name,
            time: 0.0,
            arenas: Arc::new(RwLock::new(Arenas::new())),
        }
    }

    /// Ouvre une scope TLS liant *ce monde* comme courant.
    #[inline]
    pub fn scope(&self) -> crate::utils::arenas::ArenasScope {
        crate::utils::arenas::enter_scope(self.arenas.clone())
    }
}
