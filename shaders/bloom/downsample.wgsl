// 13-tap downsample (Jimenez / CoD:AW Advanced Warfare). Two entry points:
//
//   fs_main_first  — sampled from the HDR target. Each tap is run through a
//                    soft luminance threshold and the cluster averaged with
//                    Karis weighting to suppress firefly highlights from
//                    sharp star pixels.
//   fs_main        — sampled from a previous bloom mip. Plain 13-tap average,
//                    no threshold, no Karis.
//
// Both share the same bind-group layout: source texture + sampler + post
// uniform (we read `bloom_threshold` in the first-pass entry only).

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

@group(0) @binding(0) var src_tex: texture_2d<f32>;
@group(0) @binding(1) var src_sampler: sampler;
@group(0) @binding(2) var<uniform> post: Post;

struct VsOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

fn luminance(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.2627, 0.6780, 0.0593));
}

fn karis_weight(c: vec3<f32>) -> f32 {
    return 1.0 / (1.0 + luminance(c));
}

// Soft luminance threshold (Jimenez). Below threshold → 0, smoothly ramps in.
fn soft_threshold(c: vec3<f32>) -> vec3<f32> {
    let t = post.bloom_threshold;
    let knee = t * 0.5;
    let l = luminance(c);
    let soft = l - t + knee;
    let soft_clamped = clamp(soft, 0.0, 2.0 * knee);
    let soft_pow = soft_clamped * soft_clamped / max(4.0 * knee + 1e-5, 1e-5);
    let contribution = max(soft_pow, l - t);
    return c * (contribution / max(l, 1e-5));
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let texel = 1.0 / vec2<f32>(textureDimensions(src_tex, 0));

    // 13 taps in the CoD:AW pattern. Centre cluster (D, E, I, J) gets half
    // the weight; surrounding 4 quads (ABFG, BCGH, FGKL, GHLM) split the
    // other half evenly. Total weight = 1.
    let A = textureSample(src_tex, src_sampler, in.uv + texel * vec2<f32>(-1.0,  1.0)).rgb;
    let B = textureSample(src_tex, src_sampler, in.uv + texel * vec2<f32>( 0.0,  1.0)).rgb;
    let C = textureSample(src_tex, src_sampler, in.uv + texel * vec2<f32>( 1.0,  1.0)).rgb;
    let D = textureSample(src_tex, src_sampler, in.uv + texel * vec2<f32>(-0.5,  0.5)).rgb;
    let E = textureSample(src_tex, src_sampler, in.uv + texel * vec2<f32>( 0.5,  0.5)).rgb;
    let F = textureSample(src_tex, src_sampler, in.uv + texel * vec2<f32>(-1.0,  0.0)).rgb;
    let G = textureSample(src_tex, src_sampler, in.uv).rgb;
    let H = textureSample(src_tex, src_sampler, in.uv + texel * vec2<f32>( 1.0,  0.0)).rgb;
    let I = textureSample(src_tex, src_sampler, in.uv + texel * vec2<f32>(-0.5, -0.5)).rgb;
    let J = textureSample(src_tex, src_sampler, in.uv + texel * vec2<f32>( 0.5, -0.5)).rgb;
    let K = textureSample(src_tex, src_sampler, in.uv + texel * vec2<f32>(-1.0, -1.0)).rgb;
    let L = textureSample(src_tex, src_sampler, in.uv + texel * vec2<f32>( 0.0, -1.0)).rgb;
    let M = textureSample(src_tex, src_sampler, in.uv + texel * vec2<f32>( 1.0, -1.0)).rgb;

    let result =
          (A + B + F + G) * 0.03125
        + (B + C + G + H) * 0.03125
        + (F + G + K + L) * 0.03125
        + (G + H + L + M) * 0.03125
        + (D + E + I + J) * 0.125;

    return vec4<f32>(result, 1.0);
}

@fragment
fn fs_main_first(in: VsOut) -> @location(0) vec4<f32> {
    let texel = 1.0 / vec2<f32>(textureDimensions(src_tex, 0));

    let A = textureSample(src_tex, src_sampler, in.uv + texel * vec2<f32>(-1.0,  1.0)).rgb;
    let B = textureSample(src_tex, src_sampler, in.uv + texel * vec2<f32>( 0.0,  1.0)).rgb;
    let C = textureSample(src_tex, src_sampler, in.uv + texel * vec2<f32>( 1.0,  1.0)).rgb;
    let D = textureSample(src_tex, src_sampler, in.uv + texel * vec2<f32>(-0.5,  0.5)).rgb;
    let E = textureSample(src_tex, src_sampler, in.uv + texel * vec2<f32>( 0.5,  0.5)).rgb;
    let F = textureSample(src_tex, src_sampler, in.uv + texel * vec2<f32>(-1.0,  0.0)).rgb;
    let G = textureSample(src_tex, src_sampler, in.uv).rgb;
    let H = textureSample(src_tex, src_sampler, in.uv + texel * vec2<f32>( 1.0,  0.0)).rgb;
    let I = textureSample(src_tex, src_sampler, in.uv + texel * vec2<f32>(-0.5, -0.5)).rgb;
    let J = textureSample(src_tex, src_sampler, in.uv + texel * vec2<f32>( 0.5, -0.5)).rgb;
    let K = textureSample(src_tex, src_sampler, in.uv + texel * vec2<f32>(-1.0, -1.0)).rgb;
    let L = textureSample(src_tex, src_sampler, in.uv + texel * vec2<f32>( 0.0, -1.0)).rgb;
    let M = textureSample(src_tex, src_sampler, in.uv + texel * vec2<f32>( 1.0, -1.0)).rgb;

    // Apply soft threshold to every tap before averaging.
    let aT = soft_threshold(A);
    let bT = soft_threshold(B);
    let cT = soft_threshold(C);
    let dT = soft_threshold(D);
    let eT = soft_threshold(E);
    let fT = soft_threshold(F);
    let gT = soft_threshold(G);
    let hT = soft_threshold(H);
    let iT = soft_threshold(I);
    let jT = soft_threshold(J);
    let kT = soft_threshold(K);
    let lT = soft_threshold(L);
    let mT = soft_threshold(M);

    // Karis-weighted cluster averages — 5 cluster contributions, each
    // averaged with inverse-luminance weights so a single firefly tap
    // can't dominate the cluster.
    let g1 = (aT * karis_weight(aT) + bT * karis_weight(bT)
            + fT * karis_weight(fT) + gT * karis_weight(gT))
           / max(karis_weight(aT) + karis_weight(bT) + karis_weight(fT) + karis_weight(gT), 1e-5);
    let g2 = (bT * karis_weight(bT) + cT * karis_weight(cT)
            + gT * karis_weight(gT) + hT * karis_weight(hT))
           / max(karis_weight(bT) + karis_weight(cT) + karis_weight(gT) + karis_weight(hT), 1e-5);
    let g3 = (fT * karis_weight(fT) + gT * karis_weight(gT)
            + kT * karis_weight(kT) + lT * karis_weight(lT))
           / max(karis_weight(fT) + karis_weight(gT) + karis_weight(kT) + karis_weight(lT), 1e-5);
    let g4 = (gT * karis_weight(gT) + hT * karis_weight(hT)
            + lT * karis_weight(lT) + mT * karis_weight(mT))
           / max(karis_weight(gT) + karis_weight(hT) + karis_weight(lT) + karis_weight(mT), 1e-5);
    let g5 = (dT * karis_weight(dT) + eT * karis_weight(eT)
            + iT * karis_weight(iT) + jT * karis_weight(jT))
           / max(karis_weight(dT) + karis_weight(eT) + karis_weight(iT) + karis_weight(jT), 1e-5);

    let result = (g1 + g2 + g3 + g4) * 0.125 + g5 * 0.5;
    return vec4<f32>(result, 1.0);
}
