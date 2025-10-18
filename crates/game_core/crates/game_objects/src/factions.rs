use crate::players::{PlayerId};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FactionId(pub u32);


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Faction {
    pub id: FactionId,
    pub name: String,
    pub members: Vec<PlayerId>,
}

impl Faction {
    pub fn new(id: u32, name: String) -> Self {
        Self {
            id: FactionId(id),
            name,
            members: Vec::new(),
        }
    }

    pub fn add_member(&mut self, player_id: PlayerId) {
        if !self.members.contains(&player_id) {
            self.members.push(player_id);
        }
    }

    pub fn remove_member(&mut self, player_id: &PlayerId) {
        self.members.retain(|id| id != player_id);
    }
}