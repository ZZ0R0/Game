use crate::logics::components::{LogicalComponent, LogicalComponentDelta};
use crate::utils::arenas::Arena;
use crate::utils::ids::LogicalComponentId;

#[derive(Default, Debug, Clone)]
pub struct ComponentLists {
    pub antenna_ids: Vec<LogicalComponentId>,
    // pub inventory_ids: Vec<LogicalComponentId>,
    // pub tank_ids: Vec<LogicalComponentId>,
    // pub thruster_ids: Vec<LogicalComponentId>,
    // pub health_ids: Vec<LogicalComponentId>,
    // pub energy_ids: Vec<LogicalComponentId>,
}

#[derive(Debug, Clone)]
pub struct LogicalObject {
    pub timestamp: Option<u64>,
    pub components: Arena<LogicalComponent, LogicalComponentId>, // arène locale
    pub comp_lists: ComponentLists,                              // listes locales
    comp_counter: u32,                                           // alloc locale d’IDs
    pub pending_deltas: Vec<LogicalObjectDelta>,
}

impl LogicalObject {
    pub fn new(timestamp: Option<u64>) -> Self {
        Self {
            timestamp,
            components: Arena::new(),
            comp_lists: ComponentLists::default(),
            comp_counter: 0,
            pending_deltas: Vec::new(),
        }
    }

    // ---------- gestion IDs composants ----------
    #[inline]
    pub fn alloc_component_id(&mut self) -> LogicalComponentId {
        let v = self.comp_counter;
        self.comp_counter = v.wrapping_add(1);
        LogicalComponentId(v)
    }

    /// Insère un composant déjà porteur de son id.
    #[inline]
    pub fn insert_component(&mut self, c: LogicalComponent) -> LogicalComponentId {
        self.components.insert(c)
    }

    // ---------- tagging listes ----------
    #[inline]
    pub fn tag_antenna(&mut self, id: LogicalComponentId) {
        if !self.comp_lists.antenna_ids.contains(&id) {
            self.comp_lists.antenna_ids.push(id);
        }
    }

    // ---------- deltas ----------
    pub fn record_delta(&mut self, delta: LogicalObjectDelta) {
        self.pending_deltas.push(delta);
    }

    pub fn compute_and_apply_pending_deltas(&mut self) -> Option<LogicalObjectDelta> {
        if self.pending_deltas.is_empty() {
            return None;
        }
        let merged = LogicalObjectDelta::merge(std::mem::take(&mut self.pending_deltas));
        if let Some(ref d) = merged {
            d.apply_to(self);
        }
        merged
    }
}

/* -------------------- Deltas -------------------- */

#[derive(Debug, Clone)]
pub enum LogicalOp {
    /// Ajoute un nouveau composant (déjà construit avec son id).
    Add { component: LogicalComponent },
    /// Attache un id existant à la bonne liste (utile si restauré depuis save).
    AttachAntenna { id: LogicalComponentId },
    /// Met à jour un composant ciblé.
    Update {
        id: LogicalComponentId,
        delta: ComponentDelta,
    },
    /// Détache et optionnellement supprimer de l’arène.
    Remove {
        id: LogicalComponentId,
        delete: bool,
    },
}

#[derive(Debug, Clone)]
pub struct LogicalObjectDelta {
    pub timestamp: Option<u64>,
    pub ops: Vec<LogicalOp>,
}

impl LogicalObjectDelta {
    pub fn merge(mut deltas: Vec<LogicalObjectDelta>) -> Option<LogicalObjectDelta> {
        if deltas.is_empty() {
            return None;
        }
        deltas.sort_by_key(|d| d.timestamp);
        let mut m = deltas.remove(0);
        for d in deltas {
            if d.timestamp.is_some() {
                m.timestamp = d.timestamp;
            }
            m.ops.extend(d.ops);
        }
        Some(m)
    }

    pub fn apply_to(&self, lo: &mut LogicalObject) {
        if let Some(ts) = self.timestamp {
            lo.timestamp = Some(ts);
        }

        for op in self.ops.clone() {
            match op {
                LogicalOp::Add { component } => {
                    let id = lo.insert_component(component);
                    // tag automatique si besoin (détecter le type)
                    if let Some(c) = lo.components.get(id) {
                        match c {
                            LogicalComponent::Antenna(_) => lo.tag_antenna(id),
                        }
                    }
                }
                LogicalOp::AttachAntenna { id } => {
                    lo.tag_antenna(id);
                }
                LogicalOp::Update { id, delta } => {
                    delta.apply(lo, id);
                }
                LogicalOp::Remove { id, delete } => {
                    lo.comp_lists.antenna_ids.retain(|&x| x != id);
                    if delete {
                        let _ = lo.components.remove(id);
                    }
                }
            }
        }
    }
}
