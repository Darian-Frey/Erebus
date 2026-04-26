// HDR RGBA16F render targets, 1D LUTs (gradient, blackbody planned),
// 3D noise volume (Phase 2: bake on first frame and on seed change).

use crate::app::config::HDR_FORMAT;
use crate::render::gradient::{self, GradientStop};

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

/// HDR mip pyramid used by the bloom passes. RGBA16F. Each mip is a separate
/// `TextureView` so the downsample/upsample passes can target individual
/// levels. Rebuilt whenever the HDR target resizes since mip dimensions are
/// derived from the source size.
pub struct BloomPyramid {
    #[allow(dead_code)]
    pub texture: wgpu::Texture,
    pub mips: Vec<wgpu::TextureView>,
    #[allow(dead_code)] // available for future per-mip dispatch math
    pub size: (u32, u32),
}

impl BloomPyramid {
    /// Cap on mip count. 5 → coarsest mip is 1/32 of source dimensions, which
    /// gives a roughly 32-pixel bloom radius — plenty for star/core glow.
    pub const MAX_MIPS: u32 = 5;

    pub fn new(device: &wgpu::Device, size: (u32, u32)) -> Self {
        let (w, h) = (size.0.max(2), size.1.max(2));
        let min_dim = w.min(h);
        // log2(min_dim) gives the maximum-possible mip count; cap at MAX_MIPS
        // and leave a 4-pixel coarsest mip floor.
        let max_supported = ((min_dim as f32).log2().floor() as u32).saturating_sub(2);
        let mip_count = max_supported.clamp(1, Self::MAX_MIPS);

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("erebus.bloom_pyramid"),
            size: wgpu::Extent3d {
                width: w,
                height: h,
                depth_or_array_layers: 1,
            },
            mip_level_count: mip_count,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: HDR_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let mips: Vec<_> = (0..mip_count)
            .map(|i| {
                texture.create_view(&wgpu::TextureViewDescriptor {
                    label: Some("erebus.bloom_pyramid.mip"),
                    base_mip_level: i,
                    mip_level_count: Some(1),
                    ..Default::default()
                })
            })
            .collect();

        Self {
            texture,
            mips,
            size: (w, h),
        }
    }

    pub fn mip_count(&self) -> u32 {
        self.mips.len() as u32
    }
}

/// 128³ RGBA16F volume holding pre-computed FBM. Channel layout:
///   R: 6-octave smooth FBM
///   G: 6-octave ridged FBM
///   B/A: reserved (curl noise / Worley in future phases)
/// The compute bake fills the whole texture; runtime samples it 3× for warp
/// displacement plus 1× for the main shape, replacing ~21 procedural noise
/// evaluations per density call with 4 trilinear texture fetches.
pub struct NoiseVolume {
    #[allow(dead_code)]
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

impl NoiseVolume {
    pub const SIZE: u32 = 128;
    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

    pub fn new(device: &wgpu::Device) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("erebus.noise_volume"),
            size: wgpu::Extent3d {
                width: Self::SIZE,
                height: Self::SIZE,
                depth_or_array_layers: Self::SIZE,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D3,
            format: Self::FORMAT,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D3),
            ..Default::default()
        });
        Self { texture, view }
    }
}

/// 256-texel 1D RGBA16F gradient LUT, keyed by (sqrt-mapped) density.
/// Created at init, re-uploaded by `upload()` whenever the user loads a
/// preset or edits a stop in the gradient widget (Phase 8).
pub struct GradientLut {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

impl GradientLut {
    pub const WIDTH: u32 = 256;

    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("erebus.gradient_lut"),
            size: wgpu::Extent3d {
                width: Self::WIDTH,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D1,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D1),
            ..Default::default()
        });
        let me = Self { texture, view };
        me.upload(queue, &gradient::synthwave_default());
        me
    }

    /// Re-bake and re-upload the LUT contents. Bind groups stay valid because
    /// they reference the (unchanged) view, not the bytes.
    pub fn upload(&self, queue: &wgpu::Queue, stops: &[GradientStop]) {
        let mut data = vec![0u16; (Self::WIDTH as usize) * 4];
        let last = (Self::WIDTH - 1) as f32;
        for i in 0..Self::WIDTH {
            let t = i as f32 / last;
            let rgb = gradient::sample(stops, t);
            let base = (i as usize) * 4;
            data[base] = f32_to_f16_bits(rgb[0]);
            data[base + 1] = f32_to_f16_bits(rgb[1]);
            data[base + 2] = f32_to_f16_bits(rgb[2]);
            data[base + 3] = f32_to_f16_bits(1.0);
        }
        let bytes: &[u8] = bytemuck::cast_slice(&data);
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(Self::WIDTH * 8),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: Self::WIDTH,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
    }
}

/// 1024-texel 1D RGBA16F LUT spanning 1000–40000 K. Sampled with linear
/// filtering by the starfield pass. Mitchell Charity's regression polynomial
/// (cf. Unity's `Blackbody` shader graph node) — 5 lines of math, looks right
/// against real stellar photometry within the temperature range we care about.
pub struct BlackbodyLut {
    #[allow(dead_code)]
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

impl BlackbodyLut {
    pub const TEMP_MIN: f32 = 1000.0;
    pub const TEMP_MAX: f32 = 40000.0;

    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        const W: u32 = 1024;
        let mut data = vec![0u16; (W as usize) * 4];
        for i in 0..W {
            let t_kelvin =
                Self::TEMP_MIN + (Self::TEMP_MAX - Self::TEMP_MIN) * (i as f32 / (W - 1) as f32);
            let rgb = blackbody_color(t_kelvin);
            let base = (i as usize) * 4;
            data[base] = f32_to_f16_bits(rgb[0]);
            data[base + 1] = f32_to_f16_bits(rgb[1]);
            data[base + 2] = f32_to_f16_bits(rgb[2]);
            data[base + 3] = f32_to_f16_bits(1.0);
        }
        let bytes: &[u8] = bytemuck::cast_slice(&data);

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("erebus.blackbody_lut"),
            size: wgpu::Extent3d {
                width: W,
                height: 1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D1,
            format: wgpu::TextureFormat::Rgba16Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            bytes,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(W * 8),
                rows_per_image: Some(1),
            },
            wgpu::Extent3d {
                width: W,
                height: 1,
                depth_or_array_layers: 1,
            },
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D1),
            ..Default::default()
        });
        Self { texture, view }
    }
}

fn blackbody_color(t_kelvin: f32) -> [f32; 3] {
    let temp = t_kelvin / 100.0;
    let r = if temp <= 66.0 {
        1.0
    } else {
        (329.6987 * (temp - 60.0).powf(-0.1332)).clamp(0.0, 255.0) / 255.0
    };
    let g = if temp <= 66.0 {
        ((99.4708 * temp.ln()) - 161.1196).clamp(0.0, 255.0) / 255.0
    } else {
        (288.1222 * (temp - 60.0).powf(-0.0755)).clamp(0.0, 255.0) / 255.0
    };
    let b = if temp >= 66.0 {
        1.0
    } else if temp <= 19.0 {
        0.0
    } else {
        ((138.5177 * (temp - 10.0).ln()) - 305.0448).clamp(0.0, 255.0) / 255.0
    };
    // sRGB → approximate linear (squaring beats the more expensive proper curve
    // and matches what the existing gradient LUT does — staying consistent.)
    [r * r, g * g, b * b]
}

// IEEE 754 f32 → f16 bit pattern. We do the conversion on the CPU and write
// the raw 16-bit words to the GPU because wgpu has no f16 native type.
fn f32_to_f16_bits(f: f32) -> u16 {
    let bits = f.to_bits();
    let sign = ((bits >> 16) & 0x8000) as u16;
    let exp = ((bits >> 23) & 0xff) as i32 - 127 + 15;
    let mant = bits & 0x007f_ffff;
    if exp <= 0 {
        // Subnormal or zero: flush to zero (acceptable for our LUT range).
        return sign;
    }
    if exp >= 0x1f {
        // Inf / NaN saturation.
        return sign | 0x7c00 | ((mant >> 13) as u16);
    }
    sign | ((exp as u16) << 10) | ((mant >> 13) as u16)
}
