use crate::objects::PhysicalObject;

pub mod humanoid {
    use super::*;

    #[derive(Debug, Clone)]
    pub struct HumanoidId(pub u32);

    #[derive(Debug, Clone)]
    pub struct Humanoid {
        pub id: HumanoidId,
        pub name: String,
        pub physical: PhysicalObject,
        pub health: f32,
        pub stamina: f32,
    }

    impl Humanoid {
        pub fn new(id: u32, name: String) -> Self {
            Self {
                id: HumanoidId(id),
                name,
                physical: PhysicalObject::default(),
                health: 100.0,
                stamina: 100.0,
            }
        }

        pub fn is_alive(&self) -> bool {
            self.health > 0.0
        }

        pub fn take_damage(&mut self, damage: f32) {
            self.health = (self.health - damage).max(0.0);
        }
    }

    pub mod human {
        use super::*;

        #[derive(Debug, Clone)]
        pub struct Human {
            pub humanoid: Humanoid,
            pub oxygen_level: f32,
            pub suit_energy: f32,
        }

        impl Human {
            pub fn new(id: u32, name: String) -> Self {
                Self {
                    humanoid: Humanoid::new(id, name),
                    oxygen_level: 100.0,
                    suit_energy: 100.0,
                }
            }

            pub fn consume_oxygen(&mut self, amount: f32) {
                self.oxygen_level = (self.oxygen_level - amount).max(0.0);
            }

            pub fn consume_energy(&mut self, amount: f32) {
                self.suit_energy = (self.suit_energy - amount).max(0.0);
            }

            pub fn needs_oxygen(&self) -> bool {
                self.oxygen_level < 20.0
            }

            pub fn needs_energy(&self) -> bool {
                self.suit_energy < 20.0
            }
        }
    }
}