#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Handle { pub index: u32, pub gen: u32 }

impl Handle {
    pub const INVALID: Handle = Handle { index: u32::MAX, gen: u32::MAX };
}
