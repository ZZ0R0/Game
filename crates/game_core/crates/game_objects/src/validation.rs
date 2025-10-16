use std::time::{Duration, Instant};
use crate::objects::{FloatPosition, Velocity, Acceleration};

/// Résultat de validation d'une entité
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
}

impl ValidationResult {
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
        }
    }

    pub fn invalid(error: String) -> Self {
        Self {
            is_valid: false,
            errors: vec![error],
        }
    }

    pub fn with_error(mut self, error: String) -> Self {
        self.is_valid = false;
        self.errors.push(error);
        self
    }

    pub fn combine(mut self, other: ValidationResult) -> Self {
        if !other.is_valid {
            self.is_valid = false;
            self.errors.extend(other.errors);
        }
        self
    }
}

/// Contexte de validation contenant les données temporelles et limites
#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub delta_time: f32,
    pub max_speed: f32,
    pub max_acceleration: f32,
    pub max_angular_velocity: f32,
    pub last_update: Option<Instant>,
}

impl ValidationContext {
    pub fn new(delta_time: f32) -> Self {
        Self {
            delta_time,
            max_speed: 100.0, // m/s
            max_acceleration: 50.0, // m/s²
            max_angular_velocity: 3.14, // rad/s
            last_update: None,
        }
    }

    pub fn with_physics_limits(mut self, max_speed: f32, max_acceleration: f32) -> Self {
        self.max_speed = max_speed;
        self.max_acceleration = max_acceleration;
        self
    }
}

/// Trait pour valider les données d'entité
pub trait EntityValidation {
    /// Valide les données de l'entité contre une version précédente
    fn validate_data(&self, previous: &Self, context: &ValidationContext) -> ValidationResult;
}

/// Fonctions utilitaires pour la validation physique
pub fn validate_position_change(
    current: &FloatPosition,
    previous: &FloatPosition,
    velocity: &Velocity,
    context: &ValidationContext,
) -> ValidationResult {
    let dx = current.x - previous.x;
    let dy = current.y - previous.y;
    let dz = current.z - previous.z;
    let distance = (dx * dx + dy * dy + dz * dz).sqrt();
    
    // Calcul de la distance maximale autorisée basée sur la vitesse
    let current_speed = (velocity.x * velocity.x + velocity.y * velocity.y + velocity.z * velocity.z).sqrt();
    let max_distance = (current_speed + context.max_acceleration * context.delta_time) * context.delta_time;
    
    if distance > max_distance {
        ValidationResult::invalid(format!(
            "FloatPosition change too large: {:.2}m > {:.2}m (speed: {:.2}m/s)",
            distance, max_distance, current_speed
        ))
    } else {
        ValidationResult::valid()
    }
}

pub fn validate_velocity_change(
    current: &Velocity,
    previous: &Velocity,
    context: &ValidationContext,
) -> ValidationResult {
    let dv_x = current.x - previous.x;
    let dv_y = current.y - previous.y;
    let dv_z = current.z - previous.z;
    let acceleration_magnitude = (dv_x * dv_x + dv_y * dv_y + dv_z * dv_z).sqrt() / context.delta_time;
    
    if acceleration_magnitude > context.max_acceleration {
        ValidationResult::invalid(format!(
            "Acceleration too high: {:.2}m/s² > {:.2}m/s²",
            acceleration_magnitude, context.max_acceleration
        ))
    } else {
        ValidationResult::valid()
    }
}

pub fn validate_speed_limit(velocity: &Velocity, context: &ValidationContext) -> ValidationResult {
    let speed = (velocity.x * velocity.x + velocity.y * velocity.y + velocity.z * velocity.z).sqrt();
    
    if speed > context.max_speed {
        ValidationResult::invalid(format!(
            "Speed too high: {:.2}m/s > {:.2}m/s",
            speed, context.max_speed
        ))
    } else {
        ValidationResult::valid()
    }
}