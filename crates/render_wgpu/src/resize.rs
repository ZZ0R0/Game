use glam::Vec3;
use crate::winit::{dpi::PhysicalSize, window::Window};

use crate::gfx::{CameraUBO, Gfx, ObjectUBO};
use crate::texture::create_depth_view;

impl<'w> Gfx<'w> {
    pub fn resize(&mut self, new_size: PhysicalSize<u32>) {
        if new_size.width == 0 || new_size.height == 0 {
            return;
        }
        self.size = new_size;
        self.config.width = new_size.width;
        self.config.height = new_size.height;
        self.surface.configure(&self.device, &self.config);
        self.depth_view = create_depth_view(&self.device, self.config.width, self.config.height);
        self.write_camera();
    }

    pub fn size(&self) -> PhysicalSize<u32> { self.size }

    pub fn on_window_event(&mut self, window: &Window, event: &crate::winit::event::WindowEvent) {
        let _ = self.egui_state.on_window_event(window, event);
    }

    pub fn pan(&mut self, dx: f32, dy: f32) {
        self.cam_eye.x += dx;
        self.cam_eye.y += dy;
        self.write_camera();
    }
    pub fn zoom(&mut self, dz: f32) {
        self.cam_eye.z = (self.cam_eye.z + dz).clamp(0.2, 50.0);
        self.write_camera();
    }

    pub fn rotate_camera(&mut self, yaw_delta: f32, pitch_delta: f32) {
        self.cam_yaw += yaw_delta;
        self.cam_pitch = (self.cam_pitch + pitch_delta).clamp(-1.5, 1.5);
        self.update_camera_target();
        self.write_camera();
    }

    fn update_camera_target(&mut self) {
        let forward = Vec3::new(
            self.cam_yaw.cos() * self.cam_pitch.cos(),
            self.cam_pitch.sin(),
            self.cam_yaw.sin() * self.cam_pitch.cos(),
        );
        self.cam_target = self.cam_eye + forward;
    }

    pub fn move_camera(&mut self, forward: f32, right: f32, up: f32) {
        let forward_dir = (self.cam_target - self.cam_eye).normalize();
        let right_dir = forward_dir.cross(Vec3::Y).normalize();
        let up_dir = Vec3::Y;
        
        self.cam_eye += forward_dir * forward + right_dir * right + up_dir * up;
        self.update_camera_target();
        self.write_camera();
    }

    pub fn set_fps(&mut self, fps: f32) { self.hud_fps = Some(fps); }

    pub fn toggle_vsync(&mut self) {
        let has_mailbox = self.present_modes.contains(&crate::wgpu::PresentMode::Mailbox);
        let new_mode = if self.config.present_mode == crate::wgpu::PresentMode::Fifo && has_mailbox {
            crate::wgpu::PresentMode::Mailbox
        } else {
            crate::wgpu::PresentMode::Fifo
        };
        self.config.present_mode = new_mode;
        self.surface.configure(&self.device, &self.config);
    }

    pub fn set_model(&mut self, m: glam::Mat4) { self.model = m; self.write_object(); }
    pub fn set_tint_rgba(&mut self, rgba: [f32; 4]) { self.tint = rgba; self.write_object(); }
    
    /// Get the current view-projection matrix for frustum culling
    pub fn get_vp_matrix(&self) -> glam::Mat4 {
        let aspect = (self.config.width.max(1) as f32) / (self.config.height.max(1) as f32);
        let view = glam::Mat4::look_at_rh(self.cam_eye, self.cam_target, Vec3::Y);
        let fov_degrees: f32 = 150.0;
        let fov_radians = fov_degrees.to_radians();
        let proj = glam::Mat4::perspective_rh(fov_radians, aspect, 0.01, 100.0);
        proj * view
    }

    pub(crate) fn write_camera(&mut self) {
        let aspect = (self.config.width.max(1) as f32) / (self.config.height.max(1) as f32);
        let view = glam::Mat4::look_at_rh(self.cam_eye, self.cam_target, Vec3::Y);
        
        // FOV ultra-large : 150° (2.618 radians)
        // Valeurs courantes:
        // - 60° (π/3)   : FOV standard
        // - 90° (π/2)   : FOV large
        // - 120°        : FOV très large (FPS style)
        // - 150°        : FOV ultra-large (effet fisheye)
        // - 170°        : Maximum pratique avant distorsion extrême
        let fov_degrees: f32 = 150.0;
        let fov_radians = fov_degrees.to_radians();
        
        let proj = glam::Mat4::perspective_rh(fov_radians, aspect, 0.01, 100.0);
        let vp = proj * view;
        let ubo = CameraUBO { vp: vp.to_cols_array_2d() };
        self.queue.write_buffer(&self.cam_buf, 0, bytemuck::bytes_of(&ubo));
    }

    pub(crate) fn write_object(&mut self) {
        let ubo = ObjectUBO { model: self.model.to_cols_array_2d(), tint: self.tint };
        self.queue.write_buffer(&self.obj_buf, 0, bytemuck::bytes_of(&ubo));
    }
}
