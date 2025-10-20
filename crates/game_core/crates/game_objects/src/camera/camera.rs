use crate::entities::Entity;
use crate::physics::FloatPositionDelta;

#[derive(Debug, Clone)]
pub struct Camera {
    pub assigned_entity_position_delta: FloatPositionDelta,
    pub field_of_view: f32,
    pub distance_of_view: f32,
    pub is_active: bool,
}

impl Camera {
    pub fn new(field_of_view: f32, distance_of_view: f32, is_active: bool) -> Self {
        Self {
            assigned_entity_position_delta: FloatPositionDelta::undefined(),
            field_of_view,
            distance_of_view,
            is_active,
        }
    }
}
