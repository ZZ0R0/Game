use crate::objects::PhysicalObject;

struct HumanoidId(u32);

struct Humanoid {
    pub id: HumanoidId,
    pub name: String,
    pub physical : PhysicalObject,
    pub hitbox: Volume,
}

