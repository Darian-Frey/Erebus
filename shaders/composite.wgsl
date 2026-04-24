// Phase 1 composite: trivial pass-through with [0,1] clamp. Replaced in
// Phase 5 by the full exposure -> AgX/ACES tonemap -> grade -> dither chain.

@group(0) @binding(0) var hdr_tex: texture_2d<f32>;
@group(0) @binding(1) var hdr_sampler: sampler;

struct FsIn {
    @location(0) uv: vec2<f32>,
};

@fragment
fn fs_main(in: FsIn) -> @location(0) vec4<f32> {
    let hdr = textureSample(hdr_tex, hdr_sampler, in.uv).rgb;
    return vec4<f32>(clamp(hdr, vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
