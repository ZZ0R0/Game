#[derive(Debug, Clone, PartialEq)]
pub struct ItemId(pub u32);

#[derive(Debug, Clone)]
pub struct Item {
    pub id: ItemId,
    pub name: String,
    pub mass_per_unit: f32,
    pub volume_per_unit: f32,
}

impl Item {
    pub fn new(id: u32, name: String, mass_per_unit: f32, volume_per_unit: f32) -> Self {
        Self {
            id: ItemId(id),
            name,
            mass_per_unit,
            volume_per_unit,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ItemStack {
    pub item: Item,
    pub quantity: u32,
}

impl ItemStack {
    pub fn new(item: Item, quantity: u32) -> Self {
        Self { item, quantity }
    }

    pub fn total_mass(&self) -> f32 {
        self.item.mass_per_unit * self.quantity as f32
    }

    pub fn total_volume(&self) -> f32 {
        self.item.volume_per_unit * self.quantity as f32
    }

    pub fn add(&mut self, amount: u32) {
        self.quantity += amount;
    }

    pub fn remove(&mut self, amount: u32) -> u32 {
        let removed = self.quantity.min(amount);
        self.quantity -= removed;
        removed
    }
}
