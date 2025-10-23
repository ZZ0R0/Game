use crate::logics::components::antenna::{Antenna, AntennaDelta};
use crate::logics::LogicalObject;
use crate::utils::arenas::HasId;
use crate::utils::ids::LogicalComponentId;

#[derive(Debug, Clone)]
pub enum LogicalComponent {
    Antenna(Antenna),
    // Inventory(Inventory),
    // Tank(Tank),
    // Thruster(Thruster),
    // Health(Health),
    // Energy(Energy),
}

impl HasId<LogicalComponentId> for LogicalComponent {
    #[inline]
    fn id_ref(&self) -> &LogicalComponentId {
        match self {
            LogicalComponent::Antenna(a) => &a.id,
        }
    }
    #[inline]
    fn id_mut(&mut self) -> &mut LogicalComponentId {
        match self {
            LogicalComponent::Antenna(a) => &mut a.id,
        }
    }
}

#[derive(Debug, Clone)]
pub enum LogicalComponentDelta {
    Antenna(AntennaDelta),
    // Inventory(InventoryDelta),
    // Tank(TankDelta),
    // Thruster(ThrusterDelta),
    // Health(HealthDelta),
    // Energy(EnergyDelta),
}

impl LogicalComponentDelta {
    /// Applique le delta sur le composant `id` dans l’arène locale de `lo`.
    pub fn apply(self, lo: &mut LogicalObject, id: LogicalComponentId) {
        if let Some(comp) = lo.components.get_mut(id) {
            match (comp, self) {
                (LogicalComponent::Antenna(ref mut ant), LogicalComponentDelta::Antenna(d)) => {
                    d.apply_to(ant)
                }
            }
        }
    }
}
