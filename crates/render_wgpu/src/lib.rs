//! Public surface of the renderer crate.

pub use egui_wgpu::wgpu;
pub use egui_winit::winit;

mod framegraph;
pub use framegraph::FrameGraph;

pub mod gfx; // Gfx struct + core types
pub mod mesh_upload;    // upload vertex/index buffers
pub mod frustum;        // frustum culling for chunks
pub mod buffer_pool;    // GPU buffer recycling pool
pub mod chunk_renderer; // per-chunk rendering system
pub mod camera_settings; // camera FOV and movement

mod texture;          // depth RT + simple textures + uploads
mod pipeline;         // shader + pipeline creation helpers
mod hot_reload;       // shader hot-reload on file changes
mod resize;           // size/camera/model helpers
mod render;           // frame rendering path