// Packed POD uniform structs (#[repr(C)] + bytemuck) shared with WGSL.
// Field order and padding here MUST match the matching WGSL `struct` exactly.

use bytemuck::{Pod, Zeroable};
use serde::{Deserialize, Serialize};

// `default` here lets new fields appear in older preset RONs without
// the loader exploding. `skip` keeps `_pad*` words out of the on-disk
// representation.
fn zero_u32() -> u32 {
    0
}
fn zero_f32() -> f32 {
    0.0
}
fn zero_pad2() -> [f32; 2] {
    [0.0; 2]
}

/// Render-projection mode. Selected per-pass via `FrameUniforms::mode`.
pub const RENDER_MODE_EQUIRECT: u32 = 0;
pub const RENDER_MODE_CUBEMAP: u32 = 1;

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct FrameUniforms {
    pub resolution: [f32; 2],
    pub time: f32,
    pub exposure: f32,
    pub seed: u32,
    pub frame_index: u32,
    /// 0 = equirect, 1 = cubemap. Set by the export path; live preview is
    /// always equirect.
    pub mode: u32,
    /// 0..6 cube-face index when `mode == 1`. Order: +X, -X, +Y, -Y, +Z, -Z.
    pub cube_face: u32,
}

impl Default for FrameUniforms {
    fn default() -> Self {
        Self {
            resolution: [1.0, 1.0],
            time: 0.0,
            exposure: 0.0,
            seed: 0,
            frame_index: 0,
            mode: RENDER_MODE_EQUIRECT,
            cube_face: 0,
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

// Post-processing parameters (5 vec4 = 80 bytes).
//
// Includes the skybox-preview camera (`view_mode` + yaw/pitch/fov_y/aspect)
// because the composite pass already binds this UBO and branches on
// view_mode to switch between flat equirect display and orbit-camera
// resampling. Keeping the skybox params here avoids a second bind group.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, Serialize, Deserialize)]
pub struct PostUniforms {
    pub exposure: f32,        // EV stops applied right before tonemap
    pub tonemap_mode: u32,    // 0 = AgX, 1 = ACES Fitted, 2 = Reinhard
    pub bloom_intensity: f32, // 0..2; 0 disables bloom contribution
    pub bloom_threshold: f32, // luminance threshold for first-mip bright pass

    pub bloom_radius: f32,     // tent filter radius in pixels of next-finer mip
    pub deband_amount: f32,    // 0..1 multiplier on triangular dither
    pub grade_saturation: f32, // 1.0 neutral
    pub grade_contrast: f32,   // 1.0 neutral, ~1.1 for "punchy"

    // resolution is a per-frame value injected by the renderer; never part
    // of a saved preset.
    #[serde(skip, default = "default_resolution")]
    pub resolution: [f32; 2],
    #[serde(skip, default = "zero_pad2")]
    pub _pad: [f32; 2],

    // ---- Skybox preview camera --------------------------------------------
    // Live-only — never serialised. composite.wgsl branches: 0 = sample HDR
    // by screen UV (flat equirect), 1 = reconstruct a camera ray from screen
    // UV + camera state, convert to equirect UV, sample there.
    #[serde(skip, default = "zero_u32")]
    pub view_mode: u32,
    #[serde(skip, default = "zero_f32")]
    pub yaw: f32,
    #[serde(skip, default = "zero_f32")]
    pub pitch: f32,
    #[serde(skip, default = "zero_f32")]
    pub fov_y: f32, // radians

    #[serde(skip, default = "zero_f32")]
    pub aspect: f32, // canvas width / height
    #[serde(skip, default = "zero_f32")]
    pub _pad2: f32,
    #[serde(skip, default = "zero_f32")]
    pub _pad3: f32,
    #[serde(skip, default = "zero_f32")]
    pub _pad4: f32,
}

fn default_resolution() -> [f32; 2] {
    [1.0, 1.0]
}

impl Default for PostUniforms {
    fn default() -> Self {
        Self {
            exposure: 0.0,
            tonemap_mode: 0, // AgX is the agreed default per Phase 5 research.
            bloom_intensity: 0.6,
            bloom_threshold: 1.0,
            bloom_radius: 1.0,
            deband_amount: 1.0,
            grade_saturation: 1.0,
            grade_contrast: 1.0,
            resolution: [1.0, 1.0],
            _pad: [0.0; 2],
            view_mode: 0,
            yaw: 0.0,
            pitch: 0.0,
            fov_y: 70.0_f32.to_radians(),
            aspect: 1.0,
            _pad2: 0.0,
            _pad3: 0.0,
            _pad4: 0.0,
        }
    }
}

// Per-pass bloom flag. Tells the downsample shader whether to apply the
// brightness-threshold + Karis-average path (only on the first mip) or the
// plain 13-tap path (every subsequent mip). 16 bytes — the smallest std140
// uniform block we can ship.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct BloomPassUniforms {
    pub apply_threshold: u32,
    pub _pad0: u32,
    pub _pad1: u32,
    pub _pad2: u32,
}

// Starfield parameters (4 vec4 = 64 bytes).
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, Serialize, Deserialize)]
pub struct StarfieldUniforms {
    pub density: f32,        // grid scale of layer 0 (cells per radian-ish)
    pub brightness: f32,     // global multiplier
    pub layers: u32,         // 1..=3 parallax octaves
    pub imf_exponent: f32,   // pow(rand, exp) — biases toward dim stars

    pub psf_threshold: f32,  // brightness above which diffraction spikes appear
    pub psf_intensity: f32,  // spike multiplier
    pub spike_count: u32,    // currently 4 (axis-aligned cross); 6/8 in Phase 5
    pub spike_length: f32,   // angular extent of each spike

    pub temperature_min: f32,    // K — cool red stars
    pub temperature_max: f32,    // K — hot blue stars
    pub galactic_strength: f32,  // density boost in the galactic plane
    pub galactic_width: f32,     // gaussian falloff width away from plane

    pub galactic_dir: [f32; 3],  // tilted up-vector of the galactic plane
    #[serde(skip, default = "zero_f32")]
    pub _pad0: f32,
}

impl Default for StarfieldUniforms {
    fn default() -> Self {
        Self {
            density: 80.0,
            brightness: 1.0,
            layers: 3,
            imf_exponent: 5.0,

            psf_threshold: 0.6,
            psf_intensity: 0.4,
            spike_count: 4,
            spike_length: 0.012,

            temperature_min: 2700.0,
            temperature_max: 30000.0,
            galactic_strength: 1.5,
            galactic_width: 0.3,

            // Slightly tilted band to avoid axis-aligned blandness.
            galactic_dir: [0.3, 1.0, 0.2],
            _pad0: 0.0,
        }
    }
}

// In-volume point light. Two vec4-sized rows = 32 bytes; an array of 4 fits
// std140 with no extra padding.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, Serialize, Deserialize)]
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
#[derive(Copy, Clone, Debug, Pod, Zeroable, Serialize, Deserialize)]
pub struct LightingUniforms {
    pub lights: [PointLight; 4],
    pub count: u32,
    pub shadow_steps: u32,
    pub ambient_emission: f32,
    #[serde(skip, default = "zero_u32")]
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

/// Domain-warp algorithm selector. See [`NebulaUniforms::warp_kind`].
pub const WARP_FBM: u32 = 0;
pub const WARP_CURL: u32 = 1;

/// Single-scattering phase function selector. See [`NebulaUniforms::phase_kind`].
pub const PHASE_HG: u32 = 0; // Henyey-Greenstein (legacy)
pub const PHASE_CS: u32 = 1; // Cornette-Shanks (gas-like, default)

/// Per-channel extinction "reddening law" UI selector. See
/// [`NebulaUniforms::sigma_e_law`].
pub const SIGMA_LAW_ISM: u32 = 0;    // R_V = 3.1 interstellar (default)
pub const SIGMA_LAW_GRAY: u32 = 1;   // wavelength-flat (no reddening)
pub const SIGMA_LAW_CUSTOM: u32 = 2; // user RGB

/// Density / emission model selector. See [`NebulaUniforms::density_kind`].
pub const DENSITY_LEGACY: u32 = 0;       // mix(smooth, ridged) → density (Phase 2)
pub const DENSITY_MULTICHANNEL: u32 = 1; // separate Hα / [OIII] / dust (Phase 10.5 R3)

/// Palette mode for multichannel emission. See [`NebulaUniforms::palette_mode`].
pub const PALETTE_NATURAL: u32 = 0;      // Hα → red, [OIII] → teal
pub const PALETTE_HOO: u32 = 1;          // Hα → R, [OIII] → G+B (cyan rim)

/// Reference per-channel extinction ratios. The R, G, B values approximate
/// Mathis-Rumpl-Nordsieck dust normalised so that G = 1.0 (R_V = 3.1).
pub const REDDENING_ISM: [f32; 3] = [0.65, 1.00, 1.45];
pub const REDDENING_GRAY: [f32; 3] = [1.00, 1.00, 1.00];

/// Custom serde deserializer that accepts either a scalar (legacy v1 preset)
/// or a 3-element array (current shape) for `NebulaUniforms::sigma_e`.
/// Lets old presets load without a format-version migration walk.
fn deserialize_sigma_e<'de, D>(deserializer: D) -> Result<[f32; 3], D::Error>
where
    D: serde::Deserializer<'de>,
{
    use std::fmt;
    struct ScalarOrSeq;
    impl<'de> serde::de::Visitor<'de> for ScalarOrSeq {
        type Value = [f32; 3];
        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a scalar number or a 3-element array")
        }
        fn visit_f64<E: serde::de::Error>(self, v: f64) -> Result<[f32; 3], E> {
            Ok([v as f32; 3])
        }
        fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<[f32; 3], E> {
            Ok([v as f32; 3])
        }
        fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<[f32; 3], E> {
            Ok([v as f32; 3])
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut seq: A) -> Result<[f32; 3], A::Error> {
            use serde::de::Error;
            let r = seq.next_element::<f32>()?.ok_or_else(|| A::Error::custom("missing R"))?;
            let g = seq.next_element::<f32>()?.ok_or_else(|| A::Error::custom("missing G"))?;
            let b = seq.next_element::<f32>()?.ok_or_else(|| A::Error::custom("missing B"))?;
            Ok([r, g, b])
        }
    }
    deserializer.deserialize_any(ScalarOrSeq)
}

fn default_sigma_e() -> [f32; 3] {
    [REDDENING_ISM[0] * 2.0, REDDENING_ISM[1] * 2.0, REDDENING_ISM[2] * 2.0]
}

// Nebula raymarch parameters. All vec4-aligned for std140-compatible layout.
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable, Serialize, Deserialize)]
pub struct NebulaUniforms {
    // Shape
    pub density_scale: f32,
    pub octaves_density: u32,
    pub lacunarity: f32,
    pub gain: f32,

    pub ridged_blend: f32,
    pub warp_strength: f32,
    pub octaves_warp: u32,
    /// Domain-warp algorithm. `WARP_FBM = 0` (legacy: 3 decorrelated FBM
    /// fetches mapped to [-1, 1]) or `WARP_CURL = 1` (Bridson 2007: finite-
    /// difference curl of three scalar potentials → divergence-free vector
    /// field, produces flowing tendrils instead of cottony bulges).
    /// Old presets without this field fall back to FBM via the serde default.
    #[serde(default = "zero_u32")]
    pub warp_kind: u32,

    // March
    pub steps: u32,
    pub march_length: f32,
    pub transmittance_cutoff: f32,
    pub step_density_bias: f32,

    // Scattering — sigma_e is per-channel (R2). std140 requires the vec3 to
    // be 16-byte-aligned with 4 bytes of trailing padding before the next
    // field, so it gets its own vec4 slot.
    /// Beer-Lambert extinction per RGB channel. Default ISM ratio at
    /// R_V = 3.1 makes blue ~2× more extinguished than red — interstellar
    /// reddening. Old v1 presets stored this as a scalar; the custom
    /// deserializer broadcasts that to all three channels.
    #[serde(default = "default_sigma_e", deserialize_with = "deserialize_sigma_e")]
    pub sigma_e: [f32; 3],
    #[serde(skip, default = "zero_f32")]
    pub _pad_sigma: f32,

    pub albedo: f32,
    pub hg_g: f32,
    pub density_curve: f32, // gamma applied to density before LUT lookup (0.5 = sqrt)
    /// UI hint: which "reddening law" the slider in the panel is currently
    /// modelling. Doesn't change the GPU value (sigma_e drives that
    /// directly); only used by the panel to remap the user's intensity
    /// slider when they switch laws. `SIGMA_LAW_ISM` (default) /
    /// `SIGMA_LAW_GRAY` / `SIGMA_LAW_CUSTOM`.
    #[serde(default = "zero_u32")]
    pub sigma_e_law: u32,

    // Phase R1.2 / R1.3 additions. New rows so existing Phase-3 fields keep
    // their offsets — old presets without these load with serde defaults.
    /// Single-scattering phase function. `PHASE_HG = 0` (legacy) or
    /// `PHASE_CS = 1` (Cornette-Shanks; sharper forward peak, satisfies
    /// the symmetry HG violates). New presets default to CS.
    #[serde(default = "zero_u32")]
    pub phase_kind: u32,
    /// (R1.3 placeholder.) Pivot point for the log-normal density remap.
    #[serde(default = "default_density_pivot")]
    pub density_pivot: f32,
    /// (R1.3 placeholder.) Dynamic-range exponent for the density remap.
    /// 0 = legacy linear cutoff; >0 enables the log-normal-ish remap.
    #[serde(default = "zero_f32")]
    pub density_contrast: f32,
    #[serde(skip, default = "zero_f32")]
    pub _pad1: f32,

    // Phase 10.5 R3 — multichannel emission. `density_kind` selects between
    // the legacy `mix(smooth, ridged) → density` path (back-compat) and the
    // per-line emission model (Hα + [OIII] from baked R/G channels, dust
    // from baked B). Palette mode remaps the emission RGB just before
    // accumulation so NATURAL vs HOO can swap without a re-bake.
    #[serde(default = "zero_u32")]
    pub density_kind: u32,
    #[serde(default = "zero_u32")]
    pub palette_mode: u32,
    #[serde(default = "default_halpha_strength")]
    pub halpha_strength: f32,
    #[serde(default = "default_oiii_strength")]
    pub oiii_strength: f32,

    #[serde(default = "default_dust_strength")]
    pub dust_strength: f32,
    /// Power applied to the [OIII] field after cutoff. >1 → sharper inner
    /// ionised zone. 1 = same shape as Hα; default 3 gives the classic
    /// teal-core-inside-pink-shell look.
    #[serde(default = "default_oiii_power")]
    pub oiii_power: f32,
    #[serde(skip, default = "zero_f32")]
    pub _pad2: f32,
    #[serde(skip, default = "zero_f32")]
    pub _pad3: f32,
}

fn default_halpha_strength() -> f32 {
    1.0
}
fn default_oiii_strength() -> f32 {
    0.5
}
fn default_dust_strength() -> f32 {
    1.0
}
fn default_oiii_power() -> f32 {
    3.0
}

fn default_density_pivot() -> f32 {
    0.5
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
            warp_kind: WARP_CURL,

            steps: 96,
            march_length: 1.0,
            transmittance_cutoff: 0.01,
            step_density_bias: 1.5,

            sigma_e: default_sigma_e(), // ISM ratio × intensity 2.0
            _pad_sigma: 0.0,
            albedo: 0.6,
            hg_g: 0.6,
            density_curve: 0.5,
            sigma_e_law: SIGMA_LAW_ISM,

            phase_kind: PHASE_CS,
            density_pivot: 0.5,
            density_contrast: 0.0, // R1.3 disabled by default until tuned per-preset
            _pad1: 0.0,

            density_kind: DENSITY_LEGACY, // back-compat — opt in via Nebula panel
            palette_mode: PALETTE_NATURAL,
            halpha_strength: default_halpha_strength(),
            oiii_strength: default_oiii_strength(),

            dust_strength: default_dust_strength(),
            oiii_power: default_oiii_power(),
            _pad2: 0.0,
            _pad3: 0.0,
        }
    }
}
