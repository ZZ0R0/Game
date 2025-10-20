use crate::items::{Item, ItemStack};

#[derive(Debug, Clone)]
pub struct Inventory {
    slots: Vec<ItemStack>,
    max_volume: f32, // in cubic meters
    max_mass: f32,   // in kg
}

impl Inventory {
    pub fn new(max_volume: f32, max_mass: f32) -> Self {
        Self {
            slots: Vec::new(),
            max_volume,
            max_mass,
        }
    }

    pub fn current_volume(&self) -> f32 {
        self.slots.iter().map(|slot| slot.total_volume()).sum()
    }

    pub fn current_mass(&self) -> f32 {
        self.slots.iter().map(|slot| slot.total_mass()).sum()
    }

    pub fn max_mass_to_add(&self, item: &Item, quantity: u32) -> u32 {
        (((self.max_mass - self.current_mass()) / &item.mass_per_unit).floor() as u32).min(quantity)
    }

    pub fn max_volume_to_add(&self, item: &Item, quantity: u32) -> u32 {
        (((self.max_volume - self.current_volume()) / &item.volume_per_unit).floor() as u32)
            .min(quantity)
    }

    pub fn max_to_add(&self, item: &Item, quantity: u32) -> u32 {
        self.max_mass_to_add(&item, quantity)
            .min(self.max_volume_to_add(&item, quantity))
    }

    pub fn can_add(&self, item: &Item, quantity: u32) -> bool {
        self.max_to_add(&item, quantity) > 0
    }

    pub fn add(&mut self, item: Item, quantity: u32) -> bool {
        if self.can_add(&item, quantity) {
            if let Some(stack) = self.slots.iter_mut().find(|s| s.item.id == item.id) {
                stack.add(quantity);
            }
            // Otherwise, create a new stack
            self.slots.push(ItemStack::new(item, quantity));
            return true;
        }
        false
    }

    pub fn add_max(&mut self, item: Item, quantity: u32) -> bool {
        let to_add = self.max_to_add(&item, quantity);
        if to_add > 0 {
            if let Some(stack) = self.slots.iter_mut().find(|s| s.item.id == item.id) {
                stack.add(to_add);
            } else {
                // Otherwise, create a new stack
                self.slots.push(ItemStack::new(item, to_add));
            }
            return true;
        }
        false
    }
}
