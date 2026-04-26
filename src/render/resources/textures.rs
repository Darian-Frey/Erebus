// HDR RGBA16F render targets, 1D LUTs (gradient, blackbody planned),
// 3D noise volume (Phase 2: bake on first frame and on seed change).

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

/// 256-texel 1D RGBA16F gradient LUT, keyed by (sqrt-mapped) density. The
/// shipped colour stops below are a synthwave-leaning ramp; the in-app
/// gradient editor (Phase 7) will rewrite this texture on demand.
pub struct GradientLut {
    #[allow(dead_code)]
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

impl GradientLut {
    pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
        const W: u32 = 256;
        // (density, R, G, B). Linear sRGB. Densest = brightest core.
        let stops: &[(f32, [f32; 3])] = &[
            (0.00, [0.00, 0.00, 0.00]),
            (0.10, [0.04, 0.00, 0.10]),
            (0.30, [0.30, 0.05, 0.45]), // deep magenta
            (0.55, [0.85, 0.20, 0.65]), // hot pink
            (0.78, [0.40, 0.55, 1.20]), // cyan core
            (1.00, [1.20, 1.10, 1.40]), // emissive white-violet
        ];
        let mut data = vec![0u16; (W as usize) * 4];
        for i in 0..W {
            let t = i as f32 / (W - 1) as f32;
            let rgb = sample_gradient(stops, t);
            let base = (i as usize) * 4;
            data[base] = f32_to_f16_bits(rgb[0]);
            data[base + 1] = f32_to_f16_bits(rgb[1]);
            data[base + 2] = f32_to_f16_bits(rgb[2]);
            data[base + 3] = f32_to_f16_bits(1.0);
        }
        let bytes: &[u8] = bytemuck::cast_slice(&data);

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("erebus.gradient_lut"),
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

fn sample_gradient(stops: &[(f32, [f32; 3])], t: f32) -> [f32; 3] {
    if t <= stops[0].0 {
        return stops[0].1;
    }
    if t >= stops[stops.len() - 1].0 {
        return stops[stops.len() - 1].1;
    }
    for i in 0..stops.len() - 1 {
        let (a_t, a_c) = stops[i];
        let (b_t, b_c) = stops[i + 1];
        if t >= a_t && t <= b_t {
            let k = (t - a_t) / (b_t - a_t);
            return [
                a_c[0] + (b_c[0] - a_c[0]) * k,
                a_c[1] + (b_c[1] - a_c[1]) * k,
                a_c[2] + (b_c[2] - a_c[2]) * k,
            ];
        }
    }
    [0.0, 0.0, 0.0]
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
