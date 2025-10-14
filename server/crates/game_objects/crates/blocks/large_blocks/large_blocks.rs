use crate::blocks::block::Block;
use crate::grids::large_grids::large_grid::LargeGrid;

struct LargeBlock {
    pub id: u32,
    pub name: String,
    pub block: Block,
    pub ref_large_grid: LargeGrid,
}