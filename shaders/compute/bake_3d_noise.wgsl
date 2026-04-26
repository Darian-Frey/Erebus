// Bake a 128³ RGBA16F volume holding pre-computed FBM.
//   R: smooth FBM (octaves of value-noise)
//   G: ridged FBM (octaves of (1 - |2n - 1|)²)
//   B: reserved
//   A: 1.0
//
// Workgroup size 4×4×4 = 64 invocations — well below the 256-invocation
// limit of every wgpu adapter. Dispatched 32×32×32 → 32k workgroups, each
// computing 64 cells.
//
// The bake covers world-space [0, 8)³ at unit step. Runtime samples with
// REPEAT addressing using `tex_coord = world_pos / 8`. The result is not
// strictly tiling — value-noise hashes by integer coords without periodic
// wrap — but for the [-2, 2] world-coord range that the nebula raymarch
// uses we never sample across a seam.

struct BakeParams {
    seed: u32,
    octaves: u32,
    lacunarity: f32,
    gain: f32,
};

@group(0) @binding(0) var<uniform> bake: BakeParams;
@group(0) @binding(1) var output: texture_storage_3d<rgba16float, write>;

const FBM_MAX_OCTAVES: u32 = 8u;
const VOLUME_SIZE: u32 = 128u;
const VOLUME_WORLD: f32 = 8.0; // texture covers world [0, 8)

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

fn hash3f(p: vec3<f32>) -> f32 {
    return f32(pcg3d(bitcast<vec3<u32>>(p) ^ vec3<u32>(bake.seed)).x)
         * (1.0 / 4294967296.0);
}

fn value_noise(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = p - i;
    let u = f * f * (3.0 - 2.0 * f);

    let n000 = hash3f(i + vec3<f32>(0.0, 0.0, 0.0));
    let n100 = hash3f(i + vec3<f32>(1.0, 0.0, 0.0));
    let n010 = hash3f(i + vec3<f32>(0.0, 1.0, 0.0));
    let n110 = hash3f(i + vec3<f32>(1.0, 1.0, 0.0));
    let n001 = hash3f(i + vec3<f32>(0.0, 0.0, 1.0));
    let n101 = hash3f(i + vec3<f32>(1.0, 0.0, 1.0));
    let n011 = hash3f(i + vec3<f32>(0.0, 1.0, 1.0));
    let n111 = hash3f(i + vec3<f32>(1.0, 1.0, 1.0));

    let nx00 = mix(n000, n100, u.x);
    let nx10 = mix(n010, n110, u.x);
    let nx01 = mix(n001, n101, u.x);
    let nx11 = mix(n011, n111, u.x);
    let nxy0 = mix(nx00, nx10, u.y);
    let nxy1 = mix(nx01, nx11, u.y);
    return mix(nxy0, nxy1, u.z);
}

@compute @workgroup_size(4, 4, 4)
fn cs_main(@builtin(global_invocation_id) id: vec3<u32>) {
    if (any(id >= vec3<u32>(VOLUME_SIZE))) {
        return;
    }

    // Cell index → world-space position in [0, 8).
    let p = vec3<f32>(id) * (VOLUME_WORLD / f32(VOLUME_SIZE));

    var p_smooth = p;
    var p_ridged = p;
    var amp: f32 = 0.5;
    var sum_smooth: f32 = 0.0;
    var sum_ridged: f32 = 0.0;
    var norm: f32 = 0.0;

    let n = min(bake.octaves, FBM_MAX_OCTAVES);
    for (var i: u32 = 0u; i < n; i = i + 1u) {
        let v = value_noise(p_smooth);
        sum_smooth = sum_smooth + amp * v;
        // Ridged: fold around 0.5, square for sharper ridges.
        let r = 1.0 - abs(v * 2.0 - 1.0);
        sum_ridged = sum_ridged + amp * r * r;

        norm = norm + amp;
        p_smooth = p_smooth * bake.lacunarity;
        amp = amp * bake.gain;
    }

    let out_smooth = select(0.0, sum_smooth / norm, norm > 0.0);
    let out_ridged = select(0.0, sum_ridged / norm, norm > 0.0);

    textureStore(
        output,
        vec3<i32>(id),
        vec4<f32>(out_smooth, out_ridged, 0.0, 1.0),
    );
}
