use objects_data::*;

struct GridId(u32);

struct Grid {
    pub id: GridId,
    pub name: String,
    pub physical : physical_object,
    pub size:u32,
}