use crate::humanoids::HumanoidId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Faction {
    pub id: FactionId,
    pub name: String,
    pub members: Vec<HumanoidId>,
}

impl Faction {
    pub fn new(id: u32, name: String) -> Self {
        Self {
            id: FactionId(id),
            name,
            members: Vec::new(),
        }
    }

    pub fn add_member(&mut self, player_id: HumanoidId) {
        if !self.members.contains(&player_id) {
            self.members.push(player_id);
        }
    }

    pub fn remove_member(&mut self, player_id: &HumanoidId) {
        self.members.retain(|id| id != player_id);
    }
}
