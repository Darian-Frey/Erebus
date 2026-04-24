// HDR RGBA16F render targets, mip chains for bloom, 3D noise volume,
// 1D LUTs (blackbody, gradient), blue-noise dither texture.

use crate::app::config::HDR_FORMAT;

pub struct HdrTarget {
    pub size: (u32, u32),
    // Held to keep the GPU resource alive — the view is what bind groups touch.
    #[allow(dead_code)]
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

impl HdrTarget {
    pub fn new(device: &wgpu::Device, size: (u32, u32)) -> Self {
        let (w, h) = (size.0.max(1), size.1.max(1));
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("erebus.hdr_target"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: HDR_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            size: (w, h),
            texture,
            view,
        }
    }
}
