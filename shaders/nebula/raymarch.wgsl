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
    _pad0: u32,

    steps: u32,
    march_length: f32,
    transmittance_cutoff: f32,
    step_density_bias: f32,

    sigma_e: f32,
    albedo: f32,
    hg_g: f32,
    density_curve: f32,
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

// Sample the baked FBM. Returns (smooth, ridged) in (.r, .g).
fn sample_noise(p: vec3<f32>) -> vec2<f32> {
    return textureSampleLevel(noise_3d, noise_sampler, noise_uvw(p), 0.0).rg;
}

// ---- Density ---------------------------------------------------------------

fn seed_offset() -> vec3<f32> {
    return vec3<f32>(
        f32((frame.seed >> 0u) & 0xFFu) * 0.137,
        f32((frame.seed >> 8u) & 0xFFu) * 0.241,
        f32((frame.seed >> 16u) & 0xFFu) * 0.319,
    );
}

// Cheap density for shadow marching: skips the warp.
fn shadow_density(p_in: vec3<f32>) -> f32 {
    let p_scaled = p_in * nebula.density_scale + seed_offset();
    let s = sample_noise(p_scaled).r;
    return max(s - 0.45, 0.0) * 1.8;
}

fn nebula_density(p_in: vec3<f32>) -> f32 {
    let p_scaled = p_in * nebula.density_scale + seed_offset();

    // Domain warp: 3 fetches at decorrelated offsets, mapped to [-1, 1].
    let w = vec3<f32>(
        sample_noise(p_scaled + vec3<f32>(0.0, 0.0, 0.0)).r,
        sample_noise(p_scaled + vec3<f32>(5.2, 1.3, 7.7)).r,
        sample_noise(p_scaled + vec3<f32>(2.7, 9.1, 3.1)).r,
    ) * 2.0 - 1.0;
    let p_warped = p_scaled + nebula.warp_strength * w;

    // Single fetch reads both smooth (R) and ridged (G).
    let n = sample_noise(p_warped);
    let shape = mix(n.r, n.g, nebula.ridged_blend);
    return max(shape - 0.45, 0.0) * 1.8;
}

// ---- Sampling helpers ------------------------------------------------------

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
        let phase = henyey_greenstein(cos_theta, nebula.hg_g);

        let shadow_dt = dist / f32(s);
        var shadow_optical: f32 = 0.0;
        for (var k: u32 = 0u; k < s; k = k + 1u) {
            let sp = p + l_dir * (shadow_dt * (f32(k) + 0.5));
            shadow_optical = shadow_optical
                + nebula.sigma_e * shadow_density(sp) * shadow_dt;
            if (shadow_optical > 6.0) {
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

// 1.0 inside the galactic plane band, falling off gaussian-style outside.
fn galactic_band(dir: vec3<f32>) -> f32 {
    let n = normalize(starfield.galactic_dir);
    let d = dot(dir, n);
    let w = max(starfield.galactic_width, 1e-3);
    return exp(-(d * d) / (w * w));
}

fn star_layer(dir: vec3<f32>, scale: f32, layer_seed: u32) -> vec3<f32> {
    let grid = dir * scale;
    let cell = vec3<i32>(floor(grid));
    let h = hash3i(cell, frame.seed ^ layer_seed);

    // Per-cell existence probability. Galactic plane lifts the threshold so
    // the band is markedly denser than the rest of the sphere.
    let band = galactic_band(dir);
    let presence_threshold = 0.93 - 0.06 * (band * starfield.galactic_strength);
    if (h.x < presence_threshold) {
        return vec3<f32>(0.0);
    }

    // Star direction: cell centre + jitter, kept inside the cell middle so
    // adjacent cells don't have stars touching at the boundary.
    let star_pos = (vec3<f32>(cell) + h * 0.6 + 0.2) / scale;
    let star_dir = normalize(star_pos);

    let cos_t = clamp(dot(dir, star_dir), -1.0, 1.0);
    let ang_sq = max(2.0 - 2.0 * cos_t, 0.0);

    // IMF-biased magnitude: pow(rand, exp) where exp~5 gives ~95 % dim stars.
    let mag = pow(h.y, starfield.imf_exponent);

    // Tight gaussian core. Falloff scaled by grid density so cells get a
    // roughly constant pixel-size star regardless of density slider.
    let core_falloff = scale * scale * 100.0;
    let core = exp(-ang_sq * core_falloff) * mag;

    // Diffraction spikes. Length is clamped to half the cell angular size so
    // the cross fits inside one cell and never gets truncated at a boundary.
    var spikes: f32 = 0.0;
    if (mag > starfield.psf_threshold) {
        let delta = dir - star_dir;
        // Build a cheap orthonormal basis on the sphere — pick the smaller
        // axis for the up-vector to avoid degenerating near poles.
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
        let len = min(starfield.spike_length, 0.5 / scale);
        let h_spike = exp(-ay * 600.0) * smoothstep(len, 0.0, ax);
        let v_spike = exp(-ax * 600.0) * smoothstep(len, 0.0, ay);
        spikes = max(h_spike, v_spike) * starfield.psf_intensity * mag;
    }

    // Independent hash for temperature so colour is uncorrelated with
    // magnitude. Sharing `h.y` would pin bright stars to `T_max` and dim
    // stars to `T_min` — and since IMF biases the field toward sub-pixel
    // dim stars, the visible population would all sit at `T_max`, making
    // the temperature_min slider almost imperceptible. With this fresh
    // hash you get red giants and blue dwarfs at all magnitudes.
    let temp_h = hash3i(cell + vec3<i32>(13, 17, 23), frame.seed ^ layer_seed);
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
    var transmittance: f32 = 1.0;
    var t = jitter;

    for (var i: u32 = 0u; i < steps; i = i + 1u) {
        let p = origin + dir * t;
        let d = nebula_density(p);

        // Density-adaptive step: denser regions take smaller steps.
        let dt = dt_base * max(0.25, nebula.step_density_bias - d);

        if (d > 0.001) {
            let sigma_e = nebula.sigma_e * d;
            let optical = sigma_e * dt;
            let absorbed = 1.0 - exp(-optical);

            let lut_t = clamp(pow(d, nebula.density_curve), 0.0, 1.0);
            // textureSampleLevel (not textureSample) because we're inside
            // non-uniform control flow (loop with break). WebGPU's strict
            // spec rejects implicit-LOD textureSample from divergent paths.
            let albedo_color = textureSampleLevel(gradient_tex, gradient_sampler, lut_t, 0.0).rgb;

            var in_scatter = vec3<f32>(0.0);
            if (d > 0.05 && lighting.count > 0u) {
                in_scatter = sample_lights(p, dir) * albedo_color * nebula.albedo;
            }

            let self_emission = albedo_color * lighting.ambient_emission;

            colour = colour + transmittance * absorbed * (in_scatter + self_emission);

            transmittance = transmittance * exp(-optical);
            if (transmittance < nebula.transmittance_cutoff) {
                break;
            }
        }

        t = t + dt;
        if (t > nebula.march_length) {
            break;
        }
    }

    // Stars lie behind the nebula at infinity — `transmittance` after the
    // march is exactly the fraction of background light reaching the camera.
    let stars = sample_starfield(dir) * transmittance;
    let final_colour = colour + stars;

    // Exposure moved to the post pass (Phase 5) so bloom thresholds against
    // unexposed scene radiance.
    return vec4<f32>(final_colour, 1.0);
}
