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
    _pad0: u32,
    _pad1: u32,
};

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

@group(0) @binding(0) var<uniform> frame: Frame;
@group(0) @binding(1) var<uniform> nebula: Nebula;
@group(0) @binding(2) var<uniform> lighting: Lighting;
@group(0) @binding(3) var gradient_tex: texture_1d<f32>;
@group(0) @binding(4) var gradient_sampler: sampler;
@group(0) @binding(5) var noise_3d: texture_3d<f32>;
@group(0) @binding(6) var noise_sampler: sampler;

// ---- Mappings --------------------------------------------------------------

fn equirect_dir(uv: vec2<f32>) -> vec3<f32> {
    let phi = (uv.x * 2.0 - 1.0) * PI;
    let theta = (uv.y - 0.5) * PI;
    let cos_theta = cos(theta);
    return vec3<f32>(cos_theta * sin(phi), sin(theta), cos_theta * cos(phi));
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

// ---- Fragment --------------------------------------------------------------

struct FsIn {
    @location(0) uv: vec2<f32>,
};

@fragment
fn fs_main(in: FsIn) -> @location(0) vec4<f32> {
    let dir = equirect_dir(in.uv);
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
            let albedo_color = textureSample(gradient_tex, gradient_sampler, lut_t).rgb;

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

    let intensity = exp2(frame.exposure);
    return vec4<f32>(colour * intensity, 1.0);
}
