use crate::gfx::Gfx;

impl<'w> Gfx<'w> {
    pub fn set_fov_radians(&mut self, fov_radians: f32) {
        self.fov_radians = fov_radians;
        self.write_camera();
    }

    pub fn get_fov_radians(&self) -> f32 {
        self.fov_radians
    }

    pub fn set_fov_degrees(&mut self, fov_degrees: f32) {
        self.set_fov_radians(fov_degrees.to_radians());
    }

    pub fn get_fov_degrees(&self) -> f32 {
        self.get_fov_radians().to_degrees()
    }

    pub fn set_fov_distance(&mut self, distance: f32) {
        self.fov_distance = distance;
        self.write_camera();
    }

    pub fn get_fov_distance(&self) -> f32 {
        self.fov_distance
    }

    pub fn set_chunk_perf_stats(&mut self, gen_time_ms: f32, mesh_time_ms: f32) {
        self.chunk_gen_time_ms = Some(gen_time_ms);
        self.chunk_mesh_time_ms = Some(mesh_time_ms);
    }
}
