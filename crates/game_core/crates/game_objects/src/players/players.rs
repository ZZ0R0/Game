use crate::humanoids::Humanoid;

pub struct PlayerId(pub u32);

pub struct Player {
    pub id: PlayerId,
    pub humanoid: Humanoid,
    pub in_range_entities: Vec<u32>,
}
