// Math helpers shared by every pass. Pure functions; no bindings.
//
// Hashing: PCG3D / PCG4D (Mark Jarzynski 2020) — small, fast, no axis-aligned
// repetition at our scales. See https://www.jcgt.org/published/0009/03/02/
//
// Mappings: equirect <-> direction, cube-face UV -> direction. Used by the
// nebula pass to synthesise per-pixel rays for the chosen output mode.

const PI: f32 = 3.14159265358979323846;
const TWO_PI: f32 = 6.28318530717958647692;
const INV_PI: f32 = 0.31830988618379067154;

// ---- Hashing ---------------------------------------------------------------

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

// 3-D u32 -> 3-D float in [0,1).
fn hash33(p: vec3<u32>) -> vec3<f32> {
    return vec3<f32>(pcg3d(p)) * (1.0 / 4294967296.0);
}

// 3-D float -> 3-D float in [0,1) via hashed integer coordinates.
fn hash3f(p: vec3<f32>) -> vec3<f32> {
    return hash33(bitcast<vec3<u32>>(p));
}

fn hash1u(seed: u32) -> f32 {
    var v = seed * 747796405u + 2891336453u;
    let w = ((v >> ((v >> 28u) + 4u)) ^ v) * 277803737u;
    return f32((w >> 22u) ^ w) * (1.0 / 4294967296.0);
}

// ---- Mappings --------------------------------------------------------------

// UV in [0,1]^2 -> unit ray direction via equirectangular (latlong) projection.
// V=0 → north pole (+Y), V=1 → south pole (-Y). Tiles in U.
fn equirect_dir(uv: vec2<f32>) -> vec3<f32> {
    let phi = (uv.x * 2.0 - 1.0) * PI;
    let theta = (uv.y - 0.5) * PI;
    let cos_theta = cos(theta);
    return vec3<f32>(cos_theta * sin(phi), sin(theta), cos_theta * cos(phi));
}

// Unit direction -> equirect UV (inverse of `equirect_dir`).
fn dir_equirect(d: vec3<f32>) -> vec2<f32> {
    let u = atan2(d.x, d.z) * (0.5 * INV_PI) + 0.5;
    let v = asin(clamp(d.y, -1.0, 1.0)) * INV_PI + 0.5;
    return vec2<f32>(u, v);
}

// ---- Misc ------------------------------------------------------------------

fn remap(x: f32, a0: f32, a1: f32, b0: f32, b1: f32) -> f32 {
    return b0 + (clamp((x - a0) / (a1 - a0), 0.0, 1.0)) * (b1 - b0);
}

fn saturate3(v: vec3<f32>) -> vec3<f32> {
    return clamp(v, vec3<f32>(0.0), vec3<f32>(1.0));
}
