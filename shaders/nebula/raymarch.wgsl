// Phase 3.5 volumetric nebula raymarch. The procedural noise primitives
// (value noise, FBM, ridged FBM) of Phases 2–3 are replaced by trilinear
// texture fetches into a 128³ volume baked once by `compute/bake_3d_noise.wgsl`
// (and re-baked when seed/octaves/lacunarity/gain change).
//
// Per main-march sample: 4 texture fetches (3 warp + 1 main shape).
// Per shadow-march sample: 1 texture fetch.
// vs. Phase 3 procedural: ~21 noise evals + ~8 hash3f each ≈ 168 ops/sample.

const PI: f32 = 3.14159265358979323846;
const NOISE_PERIOD_WORLD: f32 = 8.0; // texture covers world [0, 8); period in world units

// ---- Uniforms --------------------------------------------------------------

struct Frame {
    resolution: vec2<f32>,
    time: f32,
    exposure: f32,
    seed: u32,
    frame_index: u32,
    mode: u32,        // 0 = equirect, 1 = cubemap
    cube_face: u32,   // 0..6 when mode == 1
};

const MODE_EQUIRECT: u32 = 0u;
const MODE_CUBEMAP: u32 = 1u;

struct Nebula {
    density_scale: f32,
    octaves_density: u32,
    lacunarity: f32,
    gain: f32,

    ridged_blend: f32,
    warp_strength: f32,
    octaves_warp: u32,
    warp_kind: u32,    // 0 = FBM (legacy), 1 = curl-noise (divergence-free)

    steps: u32,
    march_length: f32,
    transmittance_cutoff: f32,
    step_density_bias: f32,

    // R2 — per-channel extinction (interstellar reddening).
    sigma_e: vec3<f32>,
    _pad_sigma: f32,

    albedo: f32,
    hg_g: f32,
    density_curve: f32,
    sigma_e_law: u32,       // R2 — UI-only hint, shader doesn't read it

    // Phase R1.2 / R1.3 additions.
    phase_kind: u32,        // 0 = HG (legacy), 1 = Cornette-Shanks
    density_pivot: f32,     // R1.3 — log-normal remap centre
    density_contrast: f32,  // R1.3 — 0 = legacy clip, >0 = exp remap dynamic range
    _pad1: f32,

    // R3 — multichannel emission.
    density_kind: u32,      // 0 = LEGACY mix(smooth, ridged), 1 = MULTICHANNEL
    palette_mode: u32,      // 0 = NATURAL, 1 = HOO
    halpha_strength: f32,
    oiii_strength: f32,

    dust_strength: f32,
    oiii_power: f32,        // power applied to [OIII] field (>1 = sharper inner zone)
    _pad2: f32,
    _pad3: f32,
};

struct PointLight {
    position: vec3<f32>,
    falloff: f32,
    color: vec3<f32>,
    intensity: f32,
};

struct Lighting {
    lights: array<PointLight, 4>,
    count: u32,
    shadow_steps: u32,
    ambient_emission: f32,
    _pad0: u32,
};

struct Starfield {
    density: f32,
    brightness: f32,
    layers: u32,
    imf_exponent: f32,

    psf_threshold: f32,
    psf_intensity: f32,
    spike_count: u32,
    spike_length: f32,

    temperature_min: f32,
    temperature_max: f32,
    galactic_strength: f32,
    galactic_width: f32,

    galactic_dir: vec3<f32>,
    _pad0: f32,
};

@group(0) @binding(0) var<uniform> frame: Frame;
@group(0) @binding(1) var<uniform> nebula: Nebula;
@group(0) @binding(2) var<uniform> lighting: Lighting;
@group(0) @binding(3) var gradient_tex: texture_1d<f32>;
@group(0) @binding(4) var gradient_sampler: sampler;
@group(0) @binding(5) var noise_3d: texture_3d<f32>;
@group(0) @binding(6) var noise_sampler: sampler;
@group(0) @binding(7) var<uniform> starfield: Starfield;
@group(0) @binding(8) var blackbody_tex: texture_1d<f32>;
@group(0) @binding(9) var blackbody_sampler: sampler;

// PCG3D — re-introduced for the grid-hash starfield since the procedural
// noise (which used to host this) was replaced by the baked volume.
fn pcg3d(v_in: vec3<u32>) -> vec3<u32> {
    var v = v_in * 1664525u + 1013904223u;
    v.x = v.x + v.y * v.z;
    v.y = v.y + v.z * v.x;
    v.z = v.z + v.x * v.y;
    v = v ^ (v >> vec3<u32>(16u));
    v.x = v.x + v.y * v.z;
    v.y = v.y + v.z * v.x;
    v.z = v.z + v.x * v.y;
    return v;
}

fn hash3i(p: vec3<i32>, seed: u32) -> vec3<f32> {
    let u = vec3<u32>(bitcast<u32>(p.x), bitcast<u32>(p.y), bitcast<u32>(p.z));
    return vec3<f32>(pcg3d(u ^ vec3<u32>(seed))) * (1.0 / 4294967296.0);
}

// ---- Mappings --------------------------------------------------------------

fn equirect_dir(uv: vec2<f32>) -> vec3<f32> {
    let phi = (uv.x * 2.0 - 1.0) * PI;
    let theta = (uv.y - 0.5) * PI;
    let cos_theta = cos(theta);
    return vec3<f32>(cos_theta * sin(phi), sin(theta), cos_theta * cos(phi));
}

// 90°-FOV perspective ray for a single cube face. Convention matches the
// canonical OpenGL / DirectX cubemap layout (+X, -X, +Y, -Y, +Z, -Z) so the
// six exported PNGs drop straight into Unity / Unreal / Bevy / Godot
// cubemap importers without per-face flipping.
fn cube_dir(uv: vec2<f32>, face: u32) -> vec3<f32> {
    let s = uv.x * 2.0 - 1.0;
    let t = 1.0 - uv.y * 2.0;
    var d: vec3<f32>;
    switch face {
        case 0u: { d = vec3<f32>( 1.0,    t,   -s); }  // +X
        case 1u: { d = vec3<f32>(-1.0,    t,    s); }  // -X
        case 2u: { d = vec3<f32>(   s,  1.0,   -t); }  // +Y
        case 3u: { d = vec3<f32>(   s, -1.0,    t); }  // -Y
        case 4u: { d = vec3<f32>(   s,    t,  1.0); }  // +Z
        default: { d = vec3<f32>(  -s,    t, -1.0); }  // -Z
    }
    return normalize(d);
}

fn ray_dir(uv: vec2<f32>) -> vec3<f32> {
    if (frame.mode == MODE_CUBEMAP) {
        return cube_dir(uv, frame.cube_face);
    }
    return equirect_dir(uv);
}

// World position → noise-volume texture coords. REPEAT addressing on the
// sampler wraps any out-of-range positions.
fn noise_uvw(p: vec3<f32>) -> vec3<f32> {
    return p / NOISE_PERIOD_WORLD;
}

// Sample the baked FBM. Returns (smooth, ridged) in (.r, .g) — back-compat
// shape used by curl_warp + the legacy density path.
fn sample_noise(p: vec3<f32>) -> vec2<f32> {
    return textureSampleLevel(noise_3d, noise_sampler, noise_uvw(p), 0.0).rg;
}

// Full 4-channel sample: (smooth/Hα, ridged/[OIII], dust, reserved).
// Used by the multichannel emission path (R3.2). One texture fetch,
// 4× wider read than `sample_noise` — about the same hardware cost.
fn sample_noise_4(p: vec3<f32>) -> vec4<f32> {
    return textureSampleLevel(noise_3d, noise_sampler, noise_uvw(p), 0.0);
}

// ---- Density ---------------------------------------------------------------

fn seed_offset() -> vec3<f32> {
    return vec3<f32>(
        f32((frame.seed >> 0u) & 0xFFu) * 0.137,
        f32((frame.seed >> 8u) & 0xFFu) * 0.241,
        f32((frame.seed >> 16u) & 0xFFu) * 0.319,
    );
}

// Map a [0, 1] FBM shape value to a non-negative density. Two regimes:
//   density_contrast ≤ 0  →  legacy linear clip `max(s - 0.45, 0) * 1.8`
//   density_contrast > 0  →  log-normal-ish exp remap centred on
//                            `density_pivot`. Real cold-ISM density follows
//                            a log-normal distribution (Vazquez-Semadeni
//                            1994); the remap restores the ~1.5 EV core /
//                            tendril contrast the linear clip throws away.
fn density_from_shape(shape: f32) -> f32 {
    if (nebula.density_contrast > 1.0e-3) {
        let centred = shape - nebula.density_pivot;
        let exp_density = exp(centred * nebula.density_contrast);
        // Subtract the value the remap takes at shape=0 so the noise floor
        // stays at zero density (otherwise the void fills with thin haze).
        let baseline = exp(-nebula.density_pivot * nebula.density_contrast);
        return max(exp_density - baseline, 0.0);
    }
    return max(shape - 0.45, 0.0) * 1.8;
}

// Cheap density for shadow marching: skips the warp.
fn shadow_density(p_in: vec3<f32>) -> f32 {
    let p_scaled = p_in * nebula.density_scale + seed_offset();
    return density_from_shape(sample_noise(p_scaled).r);
}

// Curl of three decorrelated scalar potentials → divergence-free vector field
// (Bridson 2007). Real ISM turbulence is dominantly solenoidal, so a curl
// warp produces flowing tendril structure where the FBM-vector warp produces
// cottony bulges (the FBM warp has divergence ~ curl in magnitude, half the
// motion is compressive).
//
// 12 noise fetches per call (6 central differences). Offsets pa/pb/pc
// decorrelate the three potentials. h = 0.125 ≈ 1.6 % of NOISE_PERIOD_WORLD,
// trades off finite-difference attenuation (sin(hk)/k) against f32 precision
// inside the trilinear-filtered noise volume — about 2 texels at native 128³
// noise, 1 texel at wasm 64³.
fn curl_warp(p_in: vec3<f32>) -> vec3<f32> {
    let h = 0.125;
    let inv_2h = 1.0 / (2.0 * h);
    let dx = vec3<f32>(h, 0.0, 0.0);
    let dy = vec3<f32>(0.0, h, 0.0);
    let dz = vec3<f32>(0.0, 0.0, h);

    let pa = vec3<f32>(11.7,  0.0,  0.0);   // psi_x base offset
    let pb = vec3<f32>( 0.0, 23.3,  0.0);   // psi_y base offset
    let pc = vec3<f32>( 0.0,  0.0, 31.1);   // psi_z base offset

    // Six unique partial derivatives. 12 noise fetches.
    let dpsi_x_dy = (sample_noise(p_in + pa + dy).r - sample_noise(p_in + pa - dy).r) * inv_2h;
    let dpsi_x_dz = (sample_noise(p_in + pa + dz).r - sample_noise(p_in + pa - dz).r) * inv_2h;
    let dpsi_y_dx = (sample_noise(p_in + pb + dx).r - sample_noise(p_in + pb - dx).r) * inv_2h;
    let dpsi_y_dz = (sample_noise(p_in + pb + dz).r - sample_noise(p_in + pb - dz).r) * inv_2h;
    let dpsi_z_dx = (sample_noise(p_in + pc + dx).r - sample_noise(p_in + pc - dx).r) * inv_2h;
    let dpsi_z_dy = (sample_noise(p_in + pc + dy).r - sample_noise(p_in + pc - dy).r) * inv_2h;

    return vec3<f32>(
        dpsi_z_dy - dpsi_y_dz,
        dpsi_x_dz - dpsi_z_dx,
        dpsi_y_dx - dpsi_x_dy,
    );
}

// Apply the domain warp and return the shifted sample position. Shared by
// `nebula_density` (legacy path) and the multichannel raymarch (R3.2).
fn warp_point(p_in: vec3<f32>) -> vec3<f32> {
    let p_scaled = p_in * nebula.density_scale + seed_offset();
    var w: vec3<f32>;
    if (nebula.warp_kind == 1u) {
        // Curl-noise warp. Output magnitude is roughly proportional to the
        // potential gradient; scale to roughly match the legacy warp's
        // [-1, 1] amplitude so existing `warp_strength` slider feels similar.
        w = curl_warp(p_scaled) * 1.5;
    } else {
        // Legacy FBM-vector warp. 3 decorrelated fetches → [-1, 1].
        w = vec3<f32>(
            sample_noise(p_scaled).r,
            sample_noise(p_scaled + vec3<f32>(5.2, 1.3, 7.7)).r,
            sample_noise(p_scaled + vec3<f32>(2.7, 9.1, 3.1)).r,
        ) * 2.0 - 1.0;
    }
    return p_scaled + nebula.warp_strength * w;
}

fn nebula_density(p_in: vec3<f32>) -> f32 {
    let p_warped = warp_point(p_in);
    // Single fetch reads both smooth (R) and ridged (G).
    let n = sample_noise(p_warped);
    let shape = mix(n.r, n.g, nebula.ridged_blend);
    return density_from_shape(shape);
}

// R3.2 — sample the per-channel emission/extinction model. Returns
// (halpha, oiii, dust) in (.x, .y, .z); .w reserved for [SII] / future use.
// One 4-channel texture fetch + a few cheap ALU ops.
fn nebula_multichannel(p_in: vec3<f32>) -> vec4<f32> {
    let p_warped = warp_point(p_in);
    let n = sample_noise_4(p_warped);
    let halpha = max(n.r - 0.45, 0.0) * nebula.halpha_strength;
    // [OIII] cutoff is higher and gets a power applied → sharper inner zone.
    let oiii = pow(max(n.g - 0.50, 0.0), nebula.oiii_power) * nebula.oiii_strength;
    // Dust uses a shallow cutoff so lanes show through even in thin regions.
    let dust = max(n.b - 0.05, 0.0) * nebula.dust_strength;
    return vec4<f32>(halpha, oiii, dust, 0.0);
}

// ---- Sampling helpers ------------------------------------------------------

// Cornette & Shanks (1992), Applied Optics 31(16):3152–3160. Drop-in HG
// replacement that satisfies a single-scattering symmetry HG violates and
// gives a tighter forward peak / softer back lobe at the same cost — one
// extra `(1 + cos²θ)` term.
fn cornette_shanks(cos_theta: f32, g: f32) -> f32 {
    let g2 = g * g;
    let denom = max(1.0 + g2 - 2.0 * g * cos_theta, 1e-3);
    let num = 3.0 * (1.0 - g2) * (1.0 + cos_theta * cos_theta);
    let den = 8.0 * PI * (2.0 + g2) * pow(denom, 1.5);
    return num / den;
}

fn phase_function(cos_theta: f32, g: f32) -> f32 {
    if (nebula.phase_kind == 1u) {
        return cornette_shanks(cos_theta, g);
    }
    return henyey_greenstein(cos_theta, g);
}

fn henyey_greenstein(cos_theta: f32, g: f32) -> f32 {
    let g2 = g * g;
    let denom = max(1.0 + g2 - 2.0 * g * cos_theta, 1e-3);
    return (1.0 - g2) / (4.0 * PI * pow(denom, 1.5));
}

// Static interleaved-gradient noise. The per-frame rotation we used in
// Phase 2 was deliberately temporal-aliasing — fine when bloom + tonemap
// smear it later, visible flicker when seen raw. Phase 5 will re-introduce
// temporal jitter through proper TAA accumulation.
fn ign(pixel: vec2<f32>) -> f32 {
    return fract(52.9829189 * fract(0.06711056 * pixel.x + 0.00583715 * pixel.y));
}

// ---- Lighting --------------------------------------------------------------

fn sample_lights(p: vec3<f32>, view_dir: vec3<f32>) -> vec3<f32> {
    var total = vec3<f32>(0.0);
    let n = min(lighting.count, 4u);
    let s = max(lighting.shadow_steps, 1u);
    for (var i: u32 = 0u; i < n; i = i + 1u) {
        let light = lighting.lights[i];
        if (light.intensity < 1e-4) {
            continue;
        }
        let to_light = light.position - p;
        let dist = max(length(to_light), 1e-3);
        let l_dir = to_light / dist;

        let cos_theta = dot(view_dir, l_dir);
        let phase = phase_function(cos_theta, nebula.hg_g);

        let shadow_dt = dist / f32(s);
        var shadow_optical = vec3<f32>(0.0);
        for (var k: u32 = 0u; k < s; k = k + 1u) {
            let sp = p + l_dir * (shadow_dt * (f32(k) + 0.5));
            shadow_optical = shadow_optical
                + nebula.sigma_e * shadow_density(sp) * shadow_dt;
            // R2: per-channel extinction. Stop only when ALL channels are
            // effectively dark — `min` is the conservative test (a light
            // could still contribute via the least-extinguished channel).
            if (min(shadow_optical.r, min(shadow_optical.g, shadow_optical.b)) > 6.0) {
                break;
            }
        }
        let shadow_t = exp(-shadow_optical);

        let dist_attenuation = 1.0 / pow(max(dist, 0.05), light.falloff);

        total = total
            + light.color * light.intensity * phase * shadow_t * dist_attenuation;
    }
    return total;
}

// ---- Starfield -------------------------------------------------------------
//
// Octahedral 2D hash. We project the unit direction onto an octahedron and
// unfold into a square in [-1, 1]² — this is a standard sphere
// parametrisation with near-uniform solid-angle coverage (worst case ~1.5×
// distortion at the square corners, vs ~3× and grid-aligned for a 3D cube
// hash). Crucially, every (u, v) cell maps to a UNIQUE direction on the
// sphere — no radial duplicates like `floor(dir * scale)` had — so we get
// honest per-direction stars and dropping the `scale * 2^i` layered hash
// produces clean octave parallax instead of moiré clusters.
//
// Encoding: Cigolle et al. "A Survey of Efficient Representations for
// Independent Unit Vectors" (JCGT 2014).

// 1.0 inside the galactic plane band, falling off gaussian-style outside.
fn galactic_band(dir: vec3<f32>) -> f32 {
    let n = normalize(starfield.galactic_dir);
    let d = dot(dir, n);
    let w = max(starfield.galactic_width, 1e-3);
    return exp(-(d * d) / (w * w));
}

// Unit direction → [-1, 1]² square. Continuous everywhere; the equator maps
// to |u|+|v|=1, the +Y pole to (0, 0), and -Y to the four corners.
fn dir_to_oct(d_in: vec3<f32>) -> vec2<f32> {
    let n = d_in / (abs(d_in.x) + abs(d_in.y) + abs(d_in.z));
    var uv = n.xz;
    if (n.y < 0.0) {
        // Lower hemisphere: unfold the octant flap into the corners.
        let s = vec2<f32>(
            select(-1.0, 1.0, uv.x >= 0.0),
            select(-1.0, 1.0, uv.y >= 0.0),
        );
        uv = (vec2<f32>(1.0) - abs(vec2<f32>(uv.y, uv.x))) * s;
    }
    return uv;
}

// Inverse of dir_to_oct. Used to reconstruct the per-cell star direction
// from a 2D cell centre (the cell uv).
fn oct_to_dir(uv: vec2<f32>) -> vec3<f32> {
    var d = vec3<f32>(uv.x, 1.0 - abs(uv.x) - abs(uv.y), uv.y);
    if (d.y < 0.0) {
        let s = vec2<f32>(
            select(-1.0, 1.0, d.x >= 0.0),
            select(-1.0, 1.0, d.z >= 0.0),
        );
        let abs_xz = abs(vec2<f32>(d.x, d.z));
        d.x = (1.0 - abs_xz.y) * s.x;
        d.z = (1.0 - abs_xz.x) * s.y;
    }
    return normalize(d);
}

fn star_layer(dir: vec3<f32>, scale: f32, layer_seed: u32) -> vec3<f32> {
    // Project to 2D octahedral square in [0, 1]², scale to integer cell.
    let oct = dir_to_oct(dir) * 0.5 + 0.5;
    let grid = oct * scale;
    let cell = vec2<i32>(floor(grid));
    // Hash on (cell.x, cell.y, 0) so we can keep using the existing
    // hash3i helper without inventing a 2D variant.
    let h = hash3i(vec3<i32>(cell, 0), frame.seed ^ layer_seed);

    // Per-cell existence probability. Galactic plane lifts the threshold so
    // the band is markedly denser than the rest of the sphere.
    let band = galactic_band(dir);
    let presence_threshold = 0.93 - 0.06 * (band * starfield.galactic_strength);
    if (h.x < presence_threshold) {
        return vec3<f32>(0.0);
    }

    // Star direction: cell centre plus jitter, kept inside the cell middle
    // so adjacent cells don't put stars on top of each other at the
    // boundary. Convert back to a direction via the octahedral inverse.
    let cell_uv = (vec2<f32>(cell) + h.xy * 0.6 + 0.2) / scale;
    let star_oct = cell_uv * 2.0 - 1.0;
    let star_dir = oct_to_dir(star_oct);

    let cos_t = clamp(dot(dir, star_dir), -1.0, 1.0);
    let ang_sq = max(2.0 - 2.0 * cos_t, 0.0);

    // IMF-biased magnitude: pow(rand, exp) where exp~5 gives ~95 % dim stars.
    let mag = pow(h.y, starfield.imf_exponent);

    // Gaussian core. Falloff is linear in scale so the brightest layer
    // (smallest scale) renders ~2-pixel stars at the live preview resolution
    // and the dimmer parallax octaves shrink toward sub-pixel as a depth
    // cue. The previous `scale² × 100` was tuned against a much higher HDR
    // resolution and produced sub-pixel stars at every layer — they read
    // as quantized diamonds because rasterisation lands on a single bright
    // pixel surrounded by fully-zero ones.
    let core_falloff = scale * 150.0;
    let core = exp(-ang_sq * core_falloff) * mag;

    // Diffraction spikes. Length capped to half the cell width so the cross
    // fits inside one cell — no need for neighbour search to avoid clipping.
    var spikes: f32 = 0.0;
    if (mag > starfield.psf_threshold) {
        let delta = dir - star_dir;
        let up = select(
            vec3<f32>(0.0, 1.0, 0.0),
            vec3<f32>(1.0, 0.0, 0.0),
            abs(star_dir.y) > 0.95,
        );
        let tx = normalize(cross(up, star_dir));
        let ty = cross(star_dir, tx);
        let dx = dot(delta, tx);
        let dy = dot(delta, ty);
        let ax = abs(dx);
        let ay = abs(dy);
        let len = min(starfield.spike_length, 1.0 / scale);
        let h_spike = exp(-ay * 600.0) * smoothstep(len, 0.0, ax);
        let v_spike = exp(-ax * 600.0) * smoothstep(len, 0.0, ay);
        spikes = max(h_spike, v_spike) * starfield.psf_intensity * mag;
    }

    // Independent hash for temperature so colour is uncorrelated with
    // magnitude. Sharing `h.y` would pin bright stars to `T_max` and dim
    // stars to `T_min` — and since IMF biases the field toward sub-pixel
    // dim stars, the visible population would all sit at `T_max`, making
    // the temperature_min slider almost imperceptible.
    let temp_h = hash3i(vec3<i32>(cell + vec2<i32>(13, 17), 23), frame.seed ^ layer_seed);
    let temp = mix(starfield.temperature_min, starfield.temperature_max, temp_h.x);
    let temp_uv = clamp((temp - 1000.0) / 39000.0, 0.0, 1.0);
    let color = textureSampleLevel(blackbody_tex, blackbody_sampler, temp_uv, 0.0).rgb;

    return color * (core + spikes) * starfield.brightness;
}

fn sample_starfield(dir: vec3<f32>) -> vec3<f32> {
    var col = vec3<f32>(0.0);
    let n = min(starfield.layers, 3u);
    for (var i: u32 = 0u; i < n; i = i + 1u) {
        let scale = starfield.density * pow(2.0, f32(i));
        col = col + star_layer(dir, scale, 1664525u * (i + 1u));
    }
    return col;
}

// ---- Fragment --------------------------------------------------------------

struct FsIn {
    @location(0) uv: vec2<f32>,
};

@fragment
fn fs_main(in: FsIn) -> @location(0) vec4<f32> {
    let dir = ray_dir(in.uv);
    let origin = vec3<f32>(0.0);

    let steps = max(nebula.steps, 1u);
    let dt_base = nebula.march_length / f32(steps);

    let pixel = in.uv * frame.resolution;
    let jitter = ign(pixel) * dt_base;

    var colour = vec3<f32>(0.0);
    // R2 — transmittance is per-channel so dust reddens transmitted light
    // (interstellar reddening). Beer-Lambert applied per channel below.
    var transmittance = vec3<f32>(1.0);
    var t = jitter;

    for (var i: u32 = 0u; i < steps; i = i + 1u) {
        let p = origin + dir * t;

        // R3.2 — pick density model. LEGACY uses a single mix(smooth, ridged)
        // density; MULTICHANNEL evaluates Hα + [OIII] + dust separately.
        var d: f32;
        var halpha: f32 = 0.0;
        var oiii: f32 = 0.0;
        if (nebula.density_kind == 1u) {
            let mc = nebula_multichannel(p);
            halpha = mc.x;
            oiii = mc.y;
            d = mc.z;  // dust drives extinction + adaptive stepping
        } else {
            d = nebula_density(p);
        }

        // Density-adaptive step: denser regions take smaller steps. In
        // multichannel mode `d` is dust, which is the right driver because
        // dust is what eats light along the ray.
        let dt = dt_base * max(0.25, nebula.step_density_bias - d);

        // Skip step entirely only when nothing is contributing — note
        // emission lines can light up regions with very thin dust.
        if (d > 0.001 || halpha > 0.001 || oiii > 0.001) {
            // R2 — per-channel extinction. sigma_e is vec3.
            let optical = nebula.sigma_e * d * dt;
            let absorbed = vec3<f32>(1.0) - exp(-optical);

            var in_scatter = vec3<f32>(0.0);
            var self_emission = vec3<f32>(0.0);
            var emission = vec3<f32>(0.0);

            if (nebula.density_kind == 1u) {
                // R3.2 — emission per spectral line at fixed sRGB peaks.
                // Hα = 656 nm → red. [OIII] = 500.7 nm → teal in NATURAL,
                // remapped to cyan in HOO so the line shows on a different
                // primary than Hα. `emission * dt` integrates the volumetric
                // line emissivity; `absorbed` factor in scattering.
                let halpha_color = vec3<f32>(1.00, 0.05, 0.10);
                var oiii_color: vec3<f32>;
                if (nebula.palette_mode == 1u) {
                    // HOO: Hα → R, [OIII] → cyan rim (G + B)
                    oiii_color = vec3<f32>(0.0, 0.7, 1.0);
                } else {
                    // NATURAL: teal core (slightly green-biased)
                    oiii_color = vec3<f32>(0.0, 1.0, 0.7);
                }
                emission = halpha * halpha_color + oiii * oiii_color;

                // Dust scatters in-volume lighting; uses the per-step albedo
                // colour as in legacy (or, when no LUT path, gray albedo).
                if (d > 0.05 && lighting.count > 0u) {
                    in_scatter = sample_lights(p, dir) * nebula.albedo;
                }
                self_emission = vec3<f32>(lighting.ambient_emission);
            } else {
                // LEGACY — gradient LUT drives both emission colour and
                // dust albedo. Identical to Phase 5 behaviour.
                let lut_t = clamp(pow(d, nebula.density_curve), 0.0, 1.0);
                // textureSampleLevel (not textureSample) because we're inside
                // non-uniform control flow (loop with break). WebGPU's strict
                // spec rejects implicit-LOD textureSample from divergent paths.
                let albedo_color = textureSampleLevel(gradient_tex, gradient_sampler, lut_t, 0.0).rgb;
                if (d > 0.05 && lighting.count > 0u) {
                    in_scatter = sample_lights(p, dir) * albedo_color * nebula.albedo;
                }
                self_emission = albedo_color * lighting.ambient_emission;
            }

            // Composition. Emission integrates over the path length; in-scatter
            // is gated by the absorbed fraction of the step (energy actually
            // interacted with dust along this dt).
            colour = colour
                + transmittance * (absorbed * (in_scatter + self_emission) + emission * dt);

            transmittance = transmittance * exp(-optical);
            // Per-channel transmittance cutoff: stop only when even the
            // longest-mean-free-path channel (red, under ISM reddening) has
            // dropped below the cutoff.
            if (max(transmittance.r, max(transmittance.g, transmittance.b))
                < nebula.transmittance_cutoff)
            {
                break;
            }
        }

        t = t + dt;
        if (t > nebula.march_length) {
            break;
        }
    }

    // Stars are no longer drawn here — they're evaluated in screen space by
    // the composite pass so they don't get smeared by the equirect → canvas
    // resample in skybox view. `transmittance` (the residual fraction of
    // background light at this direction after the dust march) is encoded
    // in the HDR alpha channel; composite reads it back to attenuate the
    // screen-space starfield by foreground dust. Use the green channel as
    // the canonical scalar (ISM reddening is normalised to G = 1).
    return vec4<f32>(colour, transmittance.g);
}
