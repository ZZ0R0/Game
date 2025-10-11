use std::fs;

use crate::gfx::{Gfx, HotReload};
use crate::pipeline::create_pipeline_with_shader;

impl<'w> Gfx<'w> {
    pub fn enable_shader_hot_reload<P: Into<std::path::PathBuf>>(
        &mut self,
        path: P,
    ) -> std::io::Result<()> {
        let path = path.into();
        let md = fs::metadata(&path)?;
        let mtime = md.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
        self.hot = Some(HotReload {
            path,
            mtime,
            last_error: None,
        });
        Ok(())
    }

    pub(crate) fn try_hot_reload(&mut self) {
        let Some(h) = self.hot.clone() else {
            return;
        };
        let Ok(md) = fs::metadata(&h.path) else {
            return;
        };
        let Ok(new_mtime) = md.modified() else {
            return;
        };
        if new_mtime <= h.mtime {
            return;
        }

        let Ok(code) = fs::read_to_string(&h.path) else {
            return;
        };

        self.device
            .push_error_scope(crate::wgpu::ErrorFilter::Validation);
        let shader = self
            .device
            .create_shader_module(crate::wgpu::ShaderModuleDescriptor {
                label: Some("hot_shader"),
                source: crate::wgpu::ShaderSource::Wgsl(code.into()),
            });
        let err = pollster::block_on(self.device.pop_error_scope());

        if let Some(e) = err {
            self.hot.as_mut().unwrap().last_error = Some(e.to_string());
        } else {
            let new_pipeline = create_pipeline_with_shader(
                &self.device,
                &[&self.cam_layout, &self.tex_layout, &self.obj_layout],
                &shader,
                self.config.format,
            );
            self.pipeline = new_pipeline;
            let mut hot = self.hot.take().unwrap();
            hot.mtime = new_mtime;
            hot.last_error = None;
            self.hot = Some(hot);
        }
    }
}
