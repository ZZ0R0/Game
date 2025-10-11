//! Configuration system for engine settings
//!
//! Loads settings from `config/settings.json` or creates default if missing

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Logging verbosity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    /// Only essential startup and error messages
    Silent,
    /// Performance summary only (default)
    Summary,
    /// Summary + important events (mode switches, chunk operations)
    Normal,
    /// All debug information
    Verbose,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Summary
    }
}

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameConfig {
    /// Graphics settings
    pub graphics: GraphicsConfig,

    /// Chunk/world rendering settings
    pub world: WorldConfig,

    /// Camera settings
    pub camera: CameraConfig,

    /// Performance settings
    pub performance: PerformanceConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphicsConfig {
    /// Window width in pixels
    pub window_width: u32,

    /// Window height in pixels
    pub window_height: u32,

    /// Start in fullscreen mode
    pub fullscreen: bool,

    /// Enable VSync (Fifo presentation mode)
    pub vsync: bool,

    /// Field of view in degrees
    pub fov_degrees: f32,

    /// Render distance in blocks
    pub render_distance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldConfig {
    /// Chunk size (blocks per chunk dimension)
    pub chunk_size: u32,

    /// View radius in chunks (automatically calculated from render_distance)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_radius_override: Option<i32>,

    /// Number of worker threads for chunk generation
    pub worker_threads: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraConfig {
    /// Camera movement speed (blocks per second)
    pub move_speed: f32,

    /// Camera movement speed multiplier when shift is held
    pub sprint_multiplier: f32,

    /// Mouse sensitivity for camera rotation
    pub mouse_sensitivity: f32,

    /// Starting camera position
    pub start_position: [f32; 3],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Target FPS (frames per second)
    pub target_fps: u32,

    /// Enable frustum culling for chunks
    pub frustum_culling: bool,

    /// Enable greedy meshing optimization
    pub greedy_meshing: bool,

    /// Logging verbosity level
    #[serde(default)]
    pub log_level: LogLevel,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            graphics: GraphicsConfig {
                window_width: 1920,
                window_height: 1080,
                fullscreen: false,
                vsync: false,
                fov_degrees: 90.0,
                render_distance: 128.0, // Reduced from 100 for better balance (was causing 1600+ chunks)
            },
            world: WorldConfig {
                chunk_size: 32,
                view_radius_override: None,
                worker_threads: 4,
            },
            camera: CameraConfig {
                move_speed: 10.0,
                sprint_multiplier: 3.0,
                mouse_sensitivity: 0.003,
                start_position: [0.0, 100.0, 0.0],
            },
            performance: PerformanceConfig {
                target_fps: 60,
                frustum_culling: true,
                greedy_meshing: true,
                log_level: LogLevel::Summary,
            },
        }
    }
}

impl GameConfig {
    /// Load configuration from file, or create default if missing
    pub fn load() -> Result<Self, String> {
        let config_path = Self::config_path();

        if config_path.exists() {
            // Load existing config
            let content = fs::read_to_string(&config_path)
                .map_err(|e| format!("Failed to read config file: {}", e))?;

            let config: GameConfig = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse config file: {}", e))?;

            println!("✅ Loaded configuration from: {}", config_path.display());
            Ok(config)
        } else {
            // Create default config
            println!(
                "⚠️  No config file found, creating default at: {}",
                config_path.display()
            );
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<(), String> {
        let config_path = Self::config_path();

        // Create config directory if it doesn't exist
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        // Serialize with pretty printing
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        fs::write(&config_path, content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        println!("💾 Saved configuration to: {}", config_path.display());
        Ok(())
    }

    /// Get the path to the config file
    fn config_path() -> PathBuf {
        // Try to place it relative to the current executable or working directory
        let config_dir = Path::new("config");
        config_dir.join("settings.json")
    }

    /// Calculate view radius from render distance
    /// With cylindrical loading:
    ///   - render_distance=100 → ~6 horizontal × 3 vertical = ~340 chunks
    ///   - render_distance=128 → ~7 horizontal × 3 vertical = ~490 chunks
    ///   - render_distance=160 → ~8 horizontal × 3 vertical = ~640 chunks
    pub fn calculate_view_radius(&self) -> i32 {
        if let Some(override_radius) = self.world.view_radius_override {
            override_radius
        } else {
            // Calculate horizontal radius from render distance
            // Reduced margin from +2 to +1 with cylindrical loading
            ((self.graphics.render_distance / self.world.chunk_size as f32).ceil() as i32) + 1
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = GameConfig::default();
        assert_eq!(config.graphics.window_width, 1920);
        assert_eq!(config.world.chunk_size, 32);
    }

    #[test]
    fn test_serialize_deserialize() {
        let config = GameConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: GameConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(
            config.graphics.fov_degrees,
            deserialized.graphics.fov_degrees
        );
    }
}
