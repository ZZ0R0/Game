// ids.rs

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorldId(pub u32);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId(pub u32);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CellId(pub i32, pub i32, pub i32);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChunkId(pub i32, pub i32, pub i32);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RegionId(pub i32, pub i32, pub i32);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FactionId(pub u32);

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct MapId {
    pub cell_id: CellId,
    pub chunk_id: ChunkId,
    pub region_id: RegionId,
}

impl MapId {
    pub fn undefined() -> Self {
        Self {
            cell_id: CellId(0, 0, 0),
            chunk_id: ChunkId(0, 0, 0),
            region_id: RegionId(0, 0, 0),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlayerId(pub u32);

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LogicalComponentId(pub u32);

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum BlockDefId {
    Large(u32),
    Small(u32),
}
impl BlockDefId {
    pub fn value(&self) -> u32 {
        match self {
            BlockDefId::Large(v) | BlockDefId::Small(v) => *v,
        }
    }
    pub fn is_large(&self) -> bool {
        matches!(self, BlockDefId::Large(_))
    }
    pub fn is_small(&self) -> bool {
        matches!(self, BlockDefId::Small(_))
    }
    pub fn unique_key(&self) -> u64 {
        let type_id: u64 = if self.is_large() { 1 } else { 2 };
        (type_id << 32) | (self.value() as u64)
    }
}

impl From<u32> for EntityId {
    fn from(v: u32) -> Self {
        EntityId(v)
    }
}
