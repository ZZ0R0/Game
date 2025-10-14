use crate::blocks::block::Block;
use crate::grids::small_grids::small_grid::SmallGrid;

struct SmallBlock {
    pub id: u32,
    pub name: String,
    pub block: Block,
    pub ref_small_grid: SmallGrid,
}