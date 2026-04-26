// Packed POD uniform structs (#[repr(C)] + bytemuck) shared with WGSL.
// Field order and padding here MUST match the matching WGSL `struct` exactly.

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct FrameUniforms {
    pub resolution: [f32; 2],
    pub time: f32,
    pub exposure: f32,
    pub seed: u32,
    pub frame_index: u32,
    pub _pad0: u32,
    pub _pad1: u32,
}

impl Default for FrameUniforms {
    fn default() -> Self {
        Self {
            resolution: [1.0, 1.0],
            time: 0.0,
            exposure: 0.0,
            seed: 0,
            frame_index: 0,
            _pad0: 0,
            _pad1: 0,
        }
    }
}

// Parameters consumed by the noise-bake compute pass. Bake runs once at
// startup and on any change to these fields; everything else is a runtime
// uniform that does NOT trigger a re-bake.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct BakeUniforms {
    pub seed: u32,
    pub octaves: u32,
    pub lacunarity: f32,
    pub gain: f32,
}

impl Default for BakeUniforms {
    fn default() -> Self {
        Self {
            seed: 0,
            octaves: 6,
            lacunarity: 2.02,
            gain: 0.5,
        }
    }
}

impl BakeUniforms {
    /// Returns true if the runtime nebula state would invalidate the current
    /// bake. Used to gate compute dispatches.
    pub fn differs(&self, other: &Self) -> bool {
        self.seed != other.seed
            || self.octaves != other.octaves
            || (self.lacunarity - other.lacunarity).abs() > 1e-5
            || (self.gain - other.gain).abs() > 1e-5
    }
}

// In-volume point light. Two vec4-sized rows = 32 bytes; an array of 4 fits
// std140 with no extra padding.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct PointLight {
    pub position: [f32; 3],
    pub falloff: f32, // exponent in 1 / dist^falloff
    pub color: [f32; 3],
    pub intensity: f32,
}

impl PointLight {
    pub const fn off() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            falloff: 2.0,
            color: [1.0, 1.0, 1.0],
            intensity: 0.0,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct LightingUniforms {
    pub lights: [PointLight; 4],
    pub count: u32,
    pub shadow_steps: u32,
    pub ambient_emission: f32,
    pub _pad0: u32,
}

impl Default for LightingUniforms {
    fn default() -> Self {
        let mut lights = [PointLight::off(); 4];
        // One warm key light, slightly off-centre so its asymmetry is visible.
        lights[0] = PointLight {
            position: [0.35, 0.20, 0.30],
            falloff: 2.0,
            color: [1.00, 0.85, 0.65],
            intensity: 4.0,
        };
        // Cool fill, opposite side, dimmer — gives bi-colour rim light.
        lights[1] = PointLight {
            position: [-0.30, -0.10, -0.20],
            falloff: 2.5,
            color: [0.45, 0.65, 1.00],
            intensity: 1.5,
        };
        Self {
            lights,
            count: 2,
            // 4 is Heckel's lower bound; with our optical-depth early-out
            // most shadow rays terminate in 2–3 steps anyway. Push to 6+
            // for export-quality renders.
            shadow_steps: 4,
            // 0 = lights only, 1 = Phase-2 self-glow look. 0.25 keeps a faint
            // baseline so unlit regions are not pitch black.
            ambient_emission: 0.25,
            _pad0: 0,
        }
    }
}

// Nebula raymarch parameters. All vec4-aligned for std140-compatible layout.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct NebulaUniforms {
    // Shape
    pub density_scale: f32,
    pub octaves_density: u32,
    pub lacunarity: f32,
    pub gain: f32,

    pub ridged_blend: f32,
    pub warp_strength: f32,
    pub octaves_warp: u32,
    pub _pad0: u32,

    // March
    pub steps: u32,
    pub march_length: f32,
    pub transmittance_cutoff: f32,
    pub step_density_bias: f32,

    // Scattering
    pub sigma_e: f32,
    pub albedo: f32,
    pub hg_g: f32,
    pub density_curve: f32, // gamma applied to density before LUT lookup (0.5 = sqrt)
}

impl Default for NebulaUniforms {
    fn default() -> Self {
        // Defaults from Phase-2 research (see docs/SHADER_NOTES.md):
        //   6 / 3 octaves, lac 2.02, gain 0.5
        //   warp 1.5 (we already have ridged blend)
        //   64 steps, transmittance cutoff 0.01, adaptive step bias 1.5
        //   HG g 0.6, sigma_e 1.5, albedo 0.6
        //   sqrt density curve
        Self {
            density_scale: 1.6,
            octaves_density: 6,
            lacunarity: 2.02,
            gain: 0.5,

            ridged_blend: 0.5,
            warp_strength: 1.5,
            octaves_warp: 3,
            _pad0: 0,

            steps: 96,
            march_length: 1.0,
            transmittance_cutoff: 0.01,
            step_density_bias: 1.5,

            sigma_e: 2.0,
            albedo: 0.6,
            hg_g: 0.6,
            density_curve: 0.5,
        }
    }
}
