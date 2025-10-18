use crate::objects::IntPosition;


pub const CHUNK_SIZE: i32 = 1;

pub struct ChunkId(pub u32);


pub struct Chunk {
    pub id: ChunkId,
}



pub struct ChunkGrid {
    pub chunks: Vec<Chunk>,
}