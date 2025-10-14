// Items don't need PhysicalObject currently

#[derive(Debug, Clone)]
pub struct ItemId(pub u32);

#[derive(Debug, Clone)]
pub struct Item {
    pub id: ItemId,
    pub name: String,
    pub stack_size: u32,
    pub mass_per_unit: f32,
}

impl Item {
    pub fn new(id: u32, name: String, stack_size: u32, mass_per_unit: f32) -> Self {
        Self {
            id: ItemId(id),
            name,
            stack_size,
            mass_per_unit,
        }
    }

    pub fn steel_plate() -> Self {
        Self::new(1, "Steel Plate".to_string(), 1000, 0.5)
    }

    pub fn iron_ore() -> Self {
        Self::new(2, "Iron Ore".to_string(), 500, 1.0)
    }
}

#[derive(Debug, Clone)]
pub struct ItemStack {
    pub item: Item,
    pub quantity: u32,
}

impl ItemStack {
    pub fn new(item: Item, quantity: u32) -> Self {
        let quantity = quantity.min(item.stack_size);
        Self { item, quantity }
    }

    pub fn total_mass(&self) -> f32 {
        self.item.mass_per_unit * self.quantity as f32
    }

    pub fn can_add(&self, amount: u32) -> u32 {
        (self.item.stack_size - self.quantity).min(amount)
    }

    pub fn add(&mut self, amount: u32) -> u32 {
        let can_add = self.can_add(amount);
        self.quantity += can_add;
        can_add
    }

    pub fn remove(&mut self, amount: u32) -> u32 {
        let removed = self.quantity.min(amount);
        self.quantity -= removed;
        removed
    }
}