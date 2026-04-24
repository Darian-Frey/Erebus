// Phase 1 placeholder: emits a UV+time gradient into the HDR target so we
// can verify the offscreen render path is alive. Phase 2 replaces this with
// the actual volumetric raymarch (FBM density, Beer-Lambert, HG scattering).

struct Frame {
    resolution: vec2<f32>,
    time: f32,
    exposure: f32,
    seed: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

@group(0) @binding(0) var<uniform> frame: Frame;

struct FsIn {
    @location(0) uv: vec2<f32>,
};

@fragment
fn fs_main(in: FsIn) -> @location(0) vec4<f32> {
    let r = in.uv.x;
    let g = in.uv.y;
    let b = 0.5 + 0.5 * sin(frame.time);
    // Push slightly above 1.0 so HDR + tonemap (Phase 5) can demonstrably reduce it.
    let intensity = exp2(frame.exposure);
    return vec4<f32>(vec3<f32>(r, g, b) * intensity, 1.0);
}
