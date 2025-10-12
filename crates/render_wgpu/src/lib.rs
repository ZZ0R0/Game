//! Public surface of the renderer crate.

pub use egui_wgpu::wgpu;
pub use egui_winit::winit;

mod framegraph;
pub use framegraph::FrameGraph;

pub mod buffer_pool; // GPU buffer recycling pool
pub mod camera_settings;
pub mod chunk_renderer; // per-chunk rendering system
pub mod frustum; // frustum culling for chunks
pub mod gfx; // Gfx struct + core types
pub mod mesh_upload; // upload vertex/index buffers // camera FOV and movement
pub mod occlusion_culler; // hardware occlusion culling

mod hot_reload; // shader hot-reload on file changes
mod pipeline; // shader + pipeline creation helpers
mod render;
mod resize; // size/camera/model helpers
mod texture; // depth RT + simple textures + uploads // frame rendering path
