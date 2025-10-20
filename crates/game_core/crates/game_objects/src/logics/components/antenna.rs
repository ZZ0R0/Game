use crate::logics::components::LogicalComponent;
use crate::logics::LogicalObject;
use crate::utils::ids::{FactionId, LogicalComponentId};

#[derive(Debug, Clone)]
pub struct Antenna {
    pub id: LogicalComponentId,
    pub min_range: f32,
    pub max_range: f32,
    pub range: f32,
    pub on: bool,
    pub faction_id: Option<FactionId>,
}

impl Antenna {
    /// Crée l’antenne dans l’arène **locale** du LogicalObject et retourne son LogicalComponentId.
    pub fn spawn(
        lo: &mut LogicalObject,
        min_range: f32,
        max_range: f32,
        range: f32,
        on: bool,
        faction_id: Option<FactionId>,
    ) -> LogicalComponentId {
        let id = lo.alloc_component_id();
        let a = Antenna {
            id,
            min_range,
            max_range,
            range,
            on,
            faction_id,
        };
        let back = lo.insert_component(LogicalComponent::Antenna(a));
        debug_assert_eq!(id, back);
        lo.tag_antenna(id);
        id
    }
}

/* Delta spécifique antenne */
#[derive(Debug, Clone)]
pub struct AntennaDelta {
    pub min_range: Option<f32>,
    pub max_range: Option<f32>,
    pub range: Option<f32>,
    pub on: Option<bool>,
}

impl AntennaDelta {
    pub fn apply_to(&self, a: &mut Antenna) {
        if let Some(v) = self.min_range {
            a.min_range = v;
        }
        if let Some(v) = self.max_range {
            a.max_range = v;
        }
        if let Some(v) = self.range {
            a.range = v;
        }
        if let Some(v) = self.on {
            a.on = v;
        }
    }
}
