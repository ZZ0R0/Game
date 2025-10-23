use crate::utils::ids::FactionId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Faction {
    pub id: FactionId,
    pub name: String,
}

impl Faction {
    pub fn new(id: FactionId, name: String) -> Self {
        Self {
            id: id,
            name: name,
        }
    }
}
