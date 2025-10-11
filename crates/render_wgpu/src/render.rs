use egui::{RichText, TopBottomPanel};
use crate::winit::window::Window;

use crate::framegraph::Node;
use crate::gfx::{Gfx, ScreenDescriptor};
use crate::FrameGraph;

impl<'w> Gfx<'w> {
    pub fn render_with(
        &mut self,
        window: &Window,
        fg: &FrameGraph,
        voxel_mesh: Option<&crate::mesh_upload::MeshBuffers>,
    ) -> Result<(), crate::wgpu::SurfaceError> {
        self.try_hot_reload();

        // egui begin
        let input = self.egui_state.take_egui_input(window);
        let full_output = self.egui_ctx.run(input, |ctx| {
            TopBottomPanel::top("overlay").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(RichText::new("Dev Engine").strong());
                    ui.separator();
                    ui.label(format!("GPU: {}", self.adapter_name));
                    ui.separator();
                    ui.label(format!("{}×{}", self.config.width, self.config.height));
                    ui.separator();
                    let vsync_on = self.config.present_mode == crate::wgpu::PresentMode::Fifo;
                    ui.label(if vsync_on { "VSync: On (Fifo)" } else { "VSync: Off (Mailbox)" });
                    if let Some(fps) = self.hud_fps {
                        ui.separator();
                        ui.label(format!("FPS: {:.1}", fps));
                    }
                    
                    // NEW: Render statistics
                    let stats = &self.chunk_renderer.stats;
                    if stats.total_chunks > 0 {
                        ui.separator();
                        ui.label(format!("Pos: ({:.1}, {:.1}, {:.1})", self.cam_eye.x, self.cam_eye.y, self.cam_eye.z));
                        ui.separator();
                        ui.label(format!("View: {:.0}m", self.fov_distance));
                        ui.separator();
                        ui.label(format!("Chunks: {}/{}", stats.visible_chunks, stats.total_chunks));
                        ui.separator();
                        ui.label(format!("Triangles: {}K", stats.rendered_triangles / 1000));
                        ui.separator();
                        ui.label(format!("Drawcalls: {}", stats.draw_calls));
                        ui.separator();
                        ui.label(format!("Culled: {}", stats.culled_chunks));
                        
                        // Chunk generation performance
                        if let (Some(gen_ms), Some(mesh_ms)) = (self.chunk_gen_time_ms, self.chunk_mesh_time_ms) {
                            ui.separator();
                            ui.label(format!("Gen: {:.2}ms", gen_ms));
                            ui.separator();
                            ui.label(format!("Mesh: {:.2}ms", mesh_ms));
                        }
                    }
                    
                    if let Some(name) = &self.last_img {
                        ui.separator();
                        ui.label(format!("Texture: {}", name));
                    }
                    if let Some(h) = &self.hot {
                        if let Some(err) = &h.last_error {
                            ui.separator();
                            ui.colored_label(
                                egui::Color32::from_rgb(220, 100, 100),
                                format!("WGSL error: {}", err),
                            );
                        }
                    }
                });
            });
        });
        self.egui_state.handle_platform_output(window, full_output.platform_output);

        // main color
        let frame = self.surface.get_current_texture()?;
        let view = frame.texture.create_view(&crate::wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&crate::wgpu::CommandEncoderDescriptor { label: Some("encoder") });

        // framegraph
        for node in &fg.nodes {
            match *node {
                Node::Clear(color) => {
                    let _rp = encoder.begin_render_pass(&crate::wgpu::RenderPassDescriptor {
                        label: Some("fg_clear"),
                        color_attachments: &[Some(crate::wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: crate::wgpu::Operations {
                                load: crate::wgpu::LoadOp::Clear(color),
                                store: crate::wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: Some(crate::wgpu::RenderPassDepthStencilAttachment {
                            view: &self.depth_view,
                            depth_ops: Some(crate::wgpu::Operations {
                                load: crate::wgpu::LoadOp::Clear(1.0),
                                store: crate::wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                }
                Node::Scene => {
                    let mut rp = encoder.begin_render_pass(&crate::wgpu::RenderPassDescriptor {
                        label: Some("fg_scene"),
                        color_attachments: &[Some(crate::wgpu::RenderPassColorAttachment {
                            view: &view,
                            resolve_target: None,
                            ops: crate::wgpu::Operations {
                                load: crate::wgpu::LoadOp::Load,
                                store: crate::wgpu::StoreOp::Store,
                            },
                        })],
                        depth_stencil_attachment: Some(crate::wgpu::RenderPassDepthStencilAttachment {
                            view: &self.depth_view,
                            depth_ops: Some(crate::wgpu::Operations {
                                load: crate::wgpu::LoadOp::Load,
                                store: crate::wgpu::StoreOp::Store,
                            }),
                            stencil_ops: None,
                        }),
                        timestamp_writes: None,
                        occlusion_query_set: None,
                    });
                    rp.set_pipeline(&self.pipeline);
                    rp.set_bind_group(0, &self.cam_bg, &[]);
                    rp.set_bind_group(1, &self.tex_bg, &[]);
                    rp.set_bind_group(2, &self.obj_bg, &[]);
                    
                    // NEW: Render chunks with frustum culling
                    let vp_matrix = self.get_vp_matrix();
                    let visible_chunks = self.chunk_renderer.cull_chunks(vp_matrix);
                    
                    // Draw each visible chunk
                    for chunk_pos in &visible_chunks {
                        if let Some(mesh) = self.chunk_renderer.get_mesh(*chunk_pos) {
                            rp.set_vertex_buffer(0, mesh.vbuf.slice(..));
                            rp.set_index_buffer(mesh.ibuf.slice(..), crate::wgpu::IndexFormat::Uint32);
                            rp.draw_indexed(0..mesh.index_count, 0, 0..1);
                        }
                    }
                    
                    // Fallback: if no chunks, draw default quad
                    if self.chunk_renderer.meshes.is_empty() {
                        if let Some(mesh) = voxel_mesh {
                            rp.set_vertex_buffer(0, mesh.vbuf.slice(..));
                            rp.set_index_buffer(mesh.ibuf.slice(..), crate::wgpu::IndexFormat::Uint32);
                            rp.draw_indexed(0..mesh.index_count, 0, 0..1);
                        } else {
                            rp.set_vertex_buffer(0, self.vbuf.slice(..));
                            rp.set_index_buffer(self.ibuf.slice(..), crate::wgpu::IndexFormat::Uint16);
                            rp.draw_indexed(0..self.index_count, 0, 0..1);
                        }
                    }
                }
            }
        }

        // egui paint
        let screen = ScreenDescriptor {
            size_in_pixels: [self.config.width, self.config.height],
            pixels_per_point: self.egui_ctx.pixels_per_point(),
        };
        let clipped = self.egui_ctx.tessellate(full_output.shapes, screen.pixels_per_point);

        for (id, delta) in &full_output.textures_delta.set {
            self.egui_painter
                .update_texture(&self.device, &self.queue, *id, delta);
        }
        self.egui_painter.update_buffers(
            &self.device,
            &self.queue,
            &mut encoder,
            &clipped,
            &screen,
        );
        {
            let mut rp = encoder
                .begin_render_pass(&crate::wgpu::RenderPassDescriptor {
                    label: Some("egui"),
                    color_attachments: &[Some(crate::wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: crate::wgpu::Operations {
                            load: crate::wgpu::LoadOp::Load,
                            store: crate::wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                })
                .forget_lifetime();
            self.egui_painter.render(&mut rp, &clipped, &screen);
        }
        for id in &full_output.textures_delta.free {
            self.egui_painter.free_texture(id);
        }

        self.queue.submit(Some(encoder.finish()));
        frame.present();
        Ok(())
    }
}
