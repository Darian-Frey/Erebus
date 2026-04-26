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
};

@group(0) @binding(0) var hdr_tex: texture_2d<f32>;
@group(0) @binding(1) var bloom_tex: texture_2d<f32>;
@group(0) @binding(2) var post_sampler: sampler;
@group(0) @binding(3) var<uniform> post: Post;

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

// ---- Fragment entry --------------------------------------------------------

@fragment
fn fs_main(in: FsIn) -> @location(0) vec4<f32> {
    let scene = textureSampleLevel(hdr_tex, post_sampler, in.uv, 0.0).rgb;
    let bloom = textureSampleLevel(bloom_tex, post_sampler, in.uv, 0.0).rgb;

    var c = scene + bloom * post.bloom_intensity;

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
