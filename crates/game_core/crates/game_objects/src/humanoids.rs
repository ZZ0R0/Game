// humanoids.rs — minimal, physics delta centralized
use crate::factions::FactionId;
use crate::physics::{PhysicalObject, PhysicalObjectDelta};
use crate::camera::Camera;
use crate::utils::arena::HasId;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HumanoidId(pub u32);

#[derive(Debug, Clone)]
pub struct Humanoid {
    pub id: HumanoidId,
    pub name: String,
    pub physical_object: PhysicalObject,
    pub health: f32,
    pub oxygen: f32,
    pub hydrogen: f32,
    pub energy: f32,
    pub pending_deltas: Vec<HumanoidDelta>,
    pub faction_id: FactionId,
    pub camera: Camera,
    pub player_controlled: bool,
}

impl HasId<u32> for Humanoid {
    #[inline]
    fn id_ref(&self) -> &u32 {
        &self.id.0
    }
}

impl Humanoid {
    pub fn new(id: u32, name: String) -> Self {
        Self {
            id: HumanoidId(id),
            name,
            physical_object: PhysicalObject::undefined(),
            health: 100.0,
            oxygen: 100.0,
            hydrogen: 100.0,
            energy: 100.0,
            pending_deltas: Vec::new(),
            faction_id: FactionId(0),
            camera: Camera::new(
                90.0,
                10.0,
                false,
            ),
            player_controlled: false,
        }
    }

    pub fn record_delta(&mut self, delta: HumanoidDelta) {
        self.pending_deltas.push(delta);
    }

    pub fn compute_and_apply_pending_deltas(&mut self) -> Option<HumanoidDelta> {
        if self.pending_deltas.is_empty() {
            return None;
        }
        let merged = HumanoidDelta::merge(std::mem::take(&mut self.pending_deltas));
        if let Some(ref d) = merged {
            d.apply_to(self);
        }
        merged
    }

    pub fn create_physics_delta(&self, phys: PhysicalObjectDelta) -> HumanoidDelta {
        HumanoidDelta {
            player_id: self.id.0,
            physics: phys,
            health: None,
            oxygen: None,
            hydrogen: None,
            energy: None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HumanoidDelta {
    pub player_id: u32,
    pub physics: PhysicalObjectDelta,
    pub health: Option<f32>,
    pub oxygen: Option<f32>,
    pub hydrogen: Option<f32>,
    pub energy: Option<f32>,
}
impl HumanoidDelta {
    pub fn empty(player_id: u32, ts: u64, seq: u64) -> Self {
        Self {
            player_id,
            physics: PhysicalObjectDelta::empty(ts, seq),
            health: None,
            oxygen: None,
            hydrogen: None,
            energy: None,
        }
    }

    pub fn apply_to(&self, h: &mut Humanoid) {
        self.physics.apply_to(&mut h.physical_object);
        if let Some(x) = self.health {
            h.health = x;
        }
        if let Some(x) = self.oxygen {
            h.oxygen = x;
        }
        if let Some(x) = self.hydrogen {
            h.hydrogen = x;
        }
        if let Some(x) = self.energy {
            h.energy = x;
        }
    }

    pub fn merge(mut v: Vec<HumanoidDelta>) -> Option<HumanoidDelta> {
        if v.is_empty() {
            return None;
        }
        v.sort_by_key(|d| d.physics.sequence);
        let mut m = v.remove(0);
        for d in v {
            if let Some(p) = PhysicalObjectDelta::merge(vec![m.physics.clone(), d.physics.clone()])
            {
                m.physics = p;
            }
            if d.health.is_some() {
                m.health = d.health;
            }
            if d.oxygen.is_some() {
                m.oxygen = d.oxygen;
            }
            if d.hydrogen.is_some() {
                m.hydrogen = d.hydrogen;
            }
            if d.energy.is_some() {
                m.energy = d.energy;
            }
        }
        Some(m)
    }

    pub fn is_empty(&self) -> bool {
        self.physics.is_empty()
            && self.health.is_none()
            && self.oxygen.is_none()
            && self.hydrogen.is_none()
            && self.energy.is_none()
    }

    pub fn estimated_size(&self) -> usize {
        std::mem::size_of::<Self>()
    }
}
