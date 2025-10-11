use crate::wgpu;

pub fn create_depth_view(device: &wgpu::Device, width: u32, height: u32) -> wgpu::TextureView {
    let depth = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("depth"),
        size: wgpu::Extent3d {
            width: width.max(1),
            height: height.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Depth24Plus,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats: &[],
    });
    depth.create_view(&wgpu::TextureViewDescriptor::default())
}

pub fn make_checker_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    w: u32,
    h: u32,
) -> (wgpu::TextureView, wgpu::Sampler) {
    let mut data = Vec::<u8>::with_capacity((w * h * 4) as usize);
    for y in 0..h {
        for x in 0..w {
            let c = if ((x / 16) + (y / 16)) % 2 == 0 {
                220
            } else {
                40
            };
            data.extend_from_slice(&[c, c, 255, 255]);
        }
    }
    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("checker_tex"),
        size: wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    upload_rgba8(queue, &tex, w, h, &data);
    let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        label: Some("checker_sampler"),
        address_mode_u: wgpu::AddressMode::ClampToEdge,
        address_mode_v: wgpu::AddressMode::ClampToEdge,
        address_mode_w: wgpu::AddressMode::ClampToEdge,
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        mipmap_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });
    (view, sampler)
}

/// Align bytes_per_row and upload RGBA8 safely.
pub fn upload_rgba8(queue: &wgpu::Queue, tex: &wgpu::Texture, w: u32, h: u32, data: &[u8]) {
    let row_bytes = 4 * w;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = row_bytes.div_ceil(align) * align;
    if padded == row_bytes {
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: tex,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            data,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(row_bytes),
                rows_per_image: Some(h),
            },
            wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
        );
        return;
    }
    let mut staged = vec![0u8; (padded * h) as usize];
    for y in 0..h {
        let src = &data[(y * row_bytes) as usize..(y * row_bytes + row_bytes) as usize];
        let dst = &mut staged[(y * padded) as usize..(y * padded + row_bytes) as usize];
        dst.copy_from_slice(src);
    }
    queue.write_texture(
        wgpu::ImageCopyTexture {
            texture: tex,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &staged,
        wgpu::ImageDataLayout {
            offset: 0,
            bytes_per_row: Some(padded),
            rows_per_image: Some(h),
        },
        wgpu::Extent3d {
            width: w,
            height: h,
            depth_or_array_layers: 1,
        },
    );
}

use crate::gfx::Gfx;
use std::path::Path;

impl<'w> Gfx<'w> {
    pub fn load_texture_path(&mut self, path: &Path) -> Result<(), String> {
        let img = image::open(path).map_err(|e| e.to_string())?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();

        let tex = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("img_tex"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        upload_rgba8(&self.queue, &tex, w, h, &rgba);

        self.tex_view = tex.create_view(&wgpu::TextureViewDescriptor::default());
        self.tex_sampler = self.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("img_sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        self.tex_bg = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("tex_bg"),
            layout: &self.tex_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&self.tex_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.tex_sampler),
                },
            ],
        });

        self.last_img = path.file_name().map(|s| s.to_string_lossy().to_string());
        Ok(())
    }
}
