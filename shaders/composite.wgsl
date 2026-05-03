// Final post-process pass: HDR + bloom → exposure → grade → tonemap →
// triangular dither → swapchain. Replaces the Phase-1 trivial pass-through.
//
// Tonemap modes (post.tonemap_mode):
//   0 = AgX (default; Sobotka — matches Blender 4.x). Polynomial approximation
//       sandwiched between sRGB↔Rec.2020 inset/outset matrices.
//   1 = ACES Fitted (Narkowicz one-liner). Industry baseline.
//   2 = Reinhard (`x / (1 + x)`). Reference comparison only — clips fast,
//       desaturates highlights badly.

struct Post {
    exposure: f32,
    tonemap_mode: u32,
    bloom_intensity: f32,
    bloom_threshold: f32,

    bloom_radius: f32,
    deband_amount: f32,
    grade_saturation: f32,
    grade_contrast: f32,

    resolution: vec2<f32>,
    _pad0: f32,
    _pad1: f32,

    // Skybox preview camera. view_mode: 0 = flat (sample HDR by screen uv),
    // 1 = orbit-camera resample (reconstruct ray, convert to equirect uv).
    // HDR contents are identical for both modes (nebula always renders the
    // full equirect).
    view_mode: u32,
    yaw: f32,
    pitch: f32,
    fov_y: f32,

    aspect: f32,
    _pad2: f32,
    _pad3: f32,
    _pad4: f32,
};

// Subset of FrameUniforms needed by the screen-space starfield. Mirror
// MUST match the offsets of `frame.seed`, `frame.mode`, `frame.cube_face`
// in src/render/uniforms.rs::FrameUniforms — composite reads from the same
// `frame_buffer`. Other Frame fields (resolution/time/exposure/frame_index)
// are not read here so we don't bother to declare them; the std140 layout
// only requires the offsets we DO read to match.
struct Frame {
    resolution: vec2<f32>,
    time: f32,
    exposure: f32,
    seed: u32,
    frame_index: u32,
    mode: u32,
    cube_face: u32,
};

// Starfield uniform — matches NebulaUniforms-adjacent StarfieldUniforms in
// src/render/uniforms.rs.
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

@group(0) @binding(0) var hdr_tex: texture_2d<f32>;
@group(0) @binding(1) var bloom_tex: texture_2d<f32>;
@group(0) @binding(2) var post_sampler: sampler;
@group(0) @binding(3) var<uniform> post: Post;
// Phase 10.5++ — starfield moved from raymarch to here so stars stay at
// screen resolution at any FOV. Five extra bindings: noise volume +
// blackbody LUT + their samplers + starfield uniform + frame uniform.
@group(0) @binding(4) var noise_3d: texture_3d<f32>;
@group(0) @binding(5) var noise_sampler: sampler;
@group(0) @binding(6) var blackbody_tex: texture_1d<f32>;
@group(0) @binding(7) var blackbody_sampler: sampler;
@group(0) @binding(8) var<uniform> starfield: Starfield;
@group(0) @binding(9) var<uniform> frame: Frame;

struct FsIn {
    @location(0) uv: vec2<f32>,
};

// ---- ACES Fitted (Narkowicz) ----------------------------------------------

fn aces_fitted(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e),
                 vec3<f32>(0.0), vec3<f32>(1.0));
}

// ---- Reinhard --------------------------------------------------------------

fn reinhard(x: vec3<f32>) -> vec3<f32> {
    return x / (1.0 + x);
}

// ---- AgX (Sobotka) — Three.js port -----------------------------------------
//
// Pipeline: linear sRGB → linear Rec.2020 → AgX inset → log2 → contrast curve
// (6th-order polynomial) → AgX outset → sRGB encode → linear sRGB.

fn agx_default_contrast(x: vec3<f32>) -> vec3<f32> {
    let x2 = x * x;
    let x4 = x2 * x2;
    return 15.5    * x4 * x2
         - 40.14   * x4 * x
         + 31.96   * x4
         - 6.868   * x2 * x
         + 0.4298  * x2
         + 0.1191  * x
         - vec3<f32>(0.00232);
}

fn agx(color: vec3<f32>) -> vec3<f32> {
    let agx_inset = mat3x3<f32>(
        0.842479062253094, 0.0784335999999992, 0.0792237451477643,
        0.0423282422610123, 0.878468636469772, 0.0791661274605434,
        0.0423756549057051, 0.0784336, 0.879142973793104,
    );
    let agx_outset = mat3x3<f32>(
        1.19687900512017, -0.0980208811401368, -0.0990297440797205,
        -0.0528968517574562, 1.15190312990417, -0.0989611768448433,
        -0.0529716355144438, -0.0980434501171241, 1.15107367264116,
    );
    let agx_min: f32 = -12.47393;
    let agx_max: f32 =   4.026069;

    var v = max(color, vec3<f32>(0.0));
    v = agx_inset * v;
    v = max(v, vec3<f32>(1e-10));
    v = log2(v);
    v = (v - agx_min) / (agx_max - agx_min);
    v = clamp(v, vec3<f32>(0.0), vec3<f32>(1.0));
    v = agx_default_contrast(v);
    v = agx_outset * v;
    // sRGB encode then implicit decode at write — keeps result in linear.
    v = pow(max(v, vec3<f32>(0.0)), vec3<f32>(2.2));
    return clamp(v, vec3<f32>(0.0), vec3<f32>(1.0));
}

// ---- Grade -----------------------------------------------------------------

fn grade(c: vec3<f32>) -> vec3<f32> {
    // Saturation around Rec.709 luminance.
    let l = dot(c, vec3<f32>(0.2126, 0.7152, 0.0722));
    let sat = mix(vec3<f32>(l), c, post.grade_saturation);
    // Contrast around middle grey.
    let con = (sat - vec3<f32>(0.5)) * post.grade_contrast + vec3<f32>(0.5);
    return max(con, vec3<f32>(0.0));
}

// ---- Triangular-PDF deband dither -----------------------------------------
//
// Subtracts two uniform-noise samples to get triangular distribution. Scaled
// to ±1/255 so the dither is invisible at 8-bit but kills banding in dark
// gradients. Hash uses Jorge Jimenez's interleaved-gradient noise.

fn ign(p: vec2<f32>) -> f32 {
    return fract(52.9829189 * fract(0.06711056 * p.x + 0.00583715 * p.y));
}

fn dither_triangular(uv: vec2<f32>) -> f32 {
    let p = uv * post.resolution;
    return (ign(p) - ign(p + vec2<f32>(101.7, 47.3))) * (1.0 / 255.0);
}

// ---- Direction mappings ----------------------------------------------------
//
// Composite needs the per-pixel world-space ray direction in two situations:
//   - Sampling the HDR texture in skybox preview (orbit-camera reconstruction).
//   - Drawing the screen-space starfield at any pixel, regardless of view.
//
// Forward map (matches shaders/nebula/raymarch.wgsl::equirect_dir):
//   x = cos(theta) * sin(phi),  y = sin(theta),  z = cos(theta) * cos(phi)
// Inverse:
//   theta = asin(y)        → uv.y = theta / PI  + 0.5
//   phi   = atan2(x, z)    → uv.x = phi   / TAU + 0.5

const PI: f32  = 3.14159265358979;
const TAU: f32 = 6.28318530717959;

fn equirect_dir(uv: vec2<f32>) -> vec3<f32> {
    let phi = (uv.x * 2.0 - 1.0) * PI;
    let theta = (uv.y - 0.5) * PI;
    let cos_theta = cos(theta);
    return vec3<f32>(cos_theta * sin(phi), sin(theta), cos_theta * cos(phi));
}

fn cube_dir(uv: vec2<f32>, face: u32) -> vec3<f32> {
    let s = uv.x * 2.0 - 1.0;
    let t = 1.0 - uv.y * 2.0;
    var d: vec3<f32>;
    switch face {
        case 0u: { d = vec3<f32>( 1.0,    t,   -s); }
        case 1u: { d = vec3<f32>(-1.0,    t,    s); }
        case 2u: { d = vec3<f32>(   s,  1.0,   -t); }
        case 3u: { d = vec3<f32>(   s, -1.0,    t); }
        case 4u: { d = vec3<f32>(   s,    t,  1.0); }
        default: { d = vec3<f32>(  -s,    t, -1.0); }
    }
    return normalize(d);
}

// Skybox preview: build the world-space ray direction for screen pixel uv.
fn skybox_world_dir(uv: vec2<f32>) -> vec3<f32> {
    let ndc = vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
    let tan_half = tan(post.fov_y * 0.5);
    let cam_dir = normalize(vec3<f32>(
        ndc.x * tan_half * post.aspect,
        ndc.y * tan_half,
        -1.0,
    ));
    let cp = cos(post.pitch);
    let sp = sin(post.pitch);
    let after_pitch = vec3<f32>(
        cam_dir.x,
        cam_dir.y * cp - cam_dir.z * sp,
        cam_dir.y * sp + cam_dir.z * cp,
    );
    let cy = cos(post.yaw);
    let sy = sin(post.yaw);
    return vec3<f32>(
        after_pitch.x *  cy + after_pitch.z * -sy,
        after_pitch.y,
        after_pitch.x *  sy + after_pitch.z *  cy,
    );
}

fn world_dir_to_equirect_uv(d: vec3<f32>) -> vec2<f32> {
    let phi   = atan2(d.x, d.z);
    let theta = asin(clamp(d.y, -1.0, 1.0));
    return vec2<f32>(phi * (1.0 / TAU) + 0.5, theta * (1.0 / PI) + 0.5);
}

// Per-pixel world direction. Picks the right reconstruction for the
// rendering path:
//   - Live preview, skybox view → orbit camera
//   - Live preview, flat view OR equirect export → equirect_dir
//   - Cubemap export face → cube_dir
fn world_dir_at_pixel(uv: vec2<f32>) -> vec3<f32> {
    if (post.view_mode == 1u) {
        return skybox_world_dir(uv);
    }
    if (frame.mode == 1u) {
        return cube_dir(uv, frame.cube_face);
    }
    return equirect_dir(uv);
}

// HDR sample uv. For skybox preview we re-project the camera direction back
// to equirect coords (the HDR is rendered as a 2:1 equirect). For flat /
// export the screen uv IS the source uv (HDR is at the same projection).
fn hdr_sample_uv(uv: vec2<f32>) -> vec2<f32> {
    if (post.view_mode == 1u) {
        return world_dir_to_equirect_uv(skybox_world_dir(uv));
    }
    return uv;
}

// ---- Starfield (screen-space) ----------------------------------------------
//
// Same code path as the raymarch's pre-Phase-10.5++ inline starfield —
// octahedral 2D hash, Gaussian core + diffraction spikes, blackbody colour
// from temperature LUT. Drawing stars in the composite (instead of caching
// them into the HDR equirect) keeps them at SCREEN resolution, so they stay
// crisp at any skybox FOV / zoom level. Cost: same as the old per-pixel
// starfield, just in a different fragment shader.

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

fn galactic_band(dir: vec3<f32>) -> f32 {
    let n = normalize(starfield.galactic_dir);
    let d = dot(dir, n);
    let w = max(starfield.galactic_width, 1e-3);
    return exp(-(d * d) / (w * w));
}

fn dir_to_oct(d_in: vec3<f32>) -> vec2<f32> {
    let n = d_in / (abs(d_in.x) + abs(d_in.y) + abs(d_in.z));
    var uv = n.xz;
    if (n.y < 0.0) {
        let s = vec2<f32>(
            select(-1.0, 1.0, uv.x >= 0.0),
            select(-1.0, 1.0, uv.y >= 0.0),
        );
        uv = (vec2<f32>(1.0) - abs(vec2<f32>(uv.y, uv.x))) * s;
    }
    return uv;
}

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
    let oct = dir_to_oct(dir) * 0.5 + 0.5;
    let grid = oct * scale;
    let cell = vec2<i32>(floor(grid));
    let h = hash3i(vec3<i32>(cell, 0), frame.seed ^ layer_seed);

    let band = galactic_band(dir);
    let presence_threshold = 0.93 - 0.06 * (band * starfield.galactic_strength);
    if (h.x < presence_threshold) {
        return vec3<f32>(0.0);
    }

    let cell_uv = (vec2<f32>(cell) + h.xy * 0.6 + 0.2) / scale;
    let star_oct = cell_uv * 2.0 - 1.0;
    let star_dir = oct_to_dir(star_oct);

    let cos_t = clamp(dot(dir, star_dir), -1.0, 1.0);
    let ang_sq = max(2.0 - 2.0 * cos_t, 0.0);

    let mag = pow(h.y, starfield.imf_exponent);

    // Linear scale for the Gaussian falloff so brighter (lower-scale) layers
    // render ~2-pixel stars at canvas resolution and dim parallax octaves
    // shrink toward sub-pixel as a depth cue. Composite renders at SCREEN
    // resolution so this falloff now maps directly to screen pixels — same
    // visual size at any FOV / zoom.
    let core_falloff = scale * 150.0;
    let core = exp(-ang_sq * core_falloff) * mag;

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

// ---- Fragment entry --------------------------------------------------------

@fragment
fn fs_main(in: FsIn) -> @location(0) vec4<f32> {
    let sample_uv = hdr_sample_uv(in.uv);
    let hdr_sample = textureSampleLevel(hdr_tex, post_sampler, sample_uv, 0.0);
    let scene = hdr_sample.rgb;
    let bloom = textureSampleLevel(bloom_tex, post_sampler, sample_uv, 0.0).rgb;

    // Screen-space starfield. Drawn here (not in the raymarch) so stars stay
    // crisp at any skybox FOV. Transmittance from the offscreen pass is in
    // the HDR alpha — multiply so stars get attenuated by foreground dust
    // exactly as if they were behind the volume at infinity.
    let dir = world_dir_at_pixel(in.uv);
    let stars = sample_starfield(dir) * hdr_sample.a;

    var c = scene + stars + bloom * post.bloom_intensity;

    // Exposure (stops). Applied before grade & tonemap.
    c = c * exp2(post.exposure);

    c = grade(c);

    var mapped: vec3<f32>;
    if (post.tonemap_mode == 0u) {
        mapped = agx(c);
    } else if (post.tonemap_mode == 1u) {
        mapped = aces_fitted(c);
    } else if (post.tonemap_mode == 2u) {
        mapped = reinhard(c);
    } else {
        // mode == 3: linear passthrough — used when writing EXR so the
        // on-disk values are scene-referred radiance, not display-referred.
        mapped = c;
    }

    let dither = dither_triangular(in.uv) * post.deband_amount;
    return vec4<f32>(mapped + vec3<f32>(dither), 1.0);
}
