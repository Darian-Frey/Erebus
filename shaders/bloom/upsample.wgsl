// 9-tap tent-filter upsample. Reads from a smaller mip and additively blends
// into the next-finer mip. The pipeline that uses this shader has additive
// blending configured at the wgpu level — we just output the filtered value.
//
// `bloom_radius` scales the tap offsets: 1.0 = standard tent, 1.5 = soft
// haze, 0.5 = tighter halo. Linearly interpolated by the sampler so it does
// not need to align with texel boundaries.

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

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
    let texel = post.bloom_radius / vec2<f32>(textureDimensions(src_tex, 0));

    // 9-tap tent filter. Centre weight 4, edges 2, corners 1 → total 16.
    let A = textureSample(src_tex, src_sampler, in.uv + vec2<f32>(-texel.x,  texel.y)).rgb;
    let B = textureSample(src_tex, src_sampler, in.uv + vec2<f32>( 0.0,      texel.y)).rgb;
    let C = textureSample(src_tex, src_sampler, in.uv + vec2<f32>( texel.x,  texel.y)).rgb;
    let D = textureSample(src_tex, src_sampler, in.uv + vec2<f32>(-texel.x,  0.0)).rgb;
    let E = textureSample(src_tex, src_sampler, in.uv).rgb;
    let F = textureSample(src_tex, src_sampler, in.uv + vec2<f32>( texel.x,  0.0)).rgb;
    let G = textureSample(src_tex, src_sampler, in.uv + vec2<f32>(-texel.x, -texel.y)).rgb;
    let H = textureSample(src_tex, src_sampler, in.uv + vec2<f32>( 0.0,     -texel.y)).rgb;
    let I = textureSample(src_tex, src_sampler, in.uv + vec2<f32>( texel.x, -texel.y)).rgb;

    let result = (A + C + G + I) * (1.0 / 16.0)
               + (B + D + F + H) * (2.0 / 16.0)
               +  E              * (4.0 / 16.0);

    return vec4<f32>(result, 1.0);
}
