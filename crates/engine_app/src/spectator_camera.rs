use glam::{Mat4, Vec3};

/// Caméra spectateur indépendante qui permet de "sortir" de la caméra principale
/// sans affecter la génération des chunks ni le meshing.
#[derive(Debug, Clone)]
pub struct SpectatorCamera {
    /// Position de la caméra spectateur
    pub position: Vec3,
    /// Direction cible de la caméra spectateur
    pub target: Vec3,
    /// Rotation yaw (horizontal)
    pub yaw: f32,
    /// Rotation pitch (vertical)
    pub pitch: f32,
    /// Vitesse de déplacement de la caméra spectateur
    pub move_speed: f32,
    /// Sensibilité de la souris pour la caméra spectateur
    pub mouse_sensitivity: f32,
    /// Si la caméra spectateur est active
    pub is_active: bool,
}

impl Default for SpectatorCamera {
    fn default() -> Self {
        Self {
            position: Vec3::new(32.0, 50.0, 32.0), // Position élevée par défaut
            target: Vec3::new(32.0, 49.0, 33.0),   // Regarde vers le bas
            yaw: 0.0,
            pitch: -0.3,
            move_speed: 20.0,       // Plus rapide que la caméra normale
            mouse_sensitivity: 0.002,
            is_active: false,
        }
    }
}

impl SpectatorCamera {
    pub fn new() -> Self {
        Self::default()
    }

    /// Active ou désactive la caméra spectateur
    pub fn toggle(&mut self) {
        self.is_active = !self.is_active;
    }

    /// Met à jour la direction cible basée sur yaw/pitch
    pub fn update_target(&mut self) {
        let forward = Vec3::new(
            self.yaw.cos() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.sin() * self.pitch.cos(),
        );
        self.target = self.position + forward;
    }

    /// Déplace la caméra spectateur dans l'espace libre
    pub fn move_camera(&mut self, forward: f32, right: f32, up: f32) {
        if !self.is_active {
            return;
        }

        let forward_dir = (self.target - self.position).normalize();
        let right_dir = forward_dir.cross(Vec3::Y).normalize();
        let up_dir = Vec3::Y;

        self.position += forward_dir * forward + right_dir * right + up_dir * up;
        self.update_target();
    }

    /// Fait tourner la caméra spectateur
    pub fn rotate(&mut self, yaw_delta: f32, pitch_delta: f32) {
        if !self.is_active {
            return;
        }

        self.yaw += yaw_delta * self.mouse_sensitivity;
        self.pitch = (self.pitch + pitch_delta * self.mouse_sensitivity).clamp(-1.5, 1.5);
        self.update_target();
    }

    /// Copie la position et rotation de la caméra principale
    pub fn copy_from_main_camera(&mut self, main_position: Vec3, main_yaw: f32, main_pitch: f32) {
        self.position = main_position;
        self.yaw = main_yaw;
        self.pitch = main_pitch;
        self.update_target();
    }

    /// Obtient la matrice view-projection pour le rendu
    pub fn get_view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.position, self.target, Vec3::Y)
    }

    /// Vérifie si la caméra spectateur est active
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Active la caméra spectateur
    pub fn activate(&mut self) {
        self.is_active = true;
    }

    /// Désactive la caméra spectateur  
    pub fn deactivate(&mut self) {
        self.is_active = false;
    }
}