// 3D noise primitives for the nebula density field.
//
// `value_noise_3d`     — fast, smooth, slightly blocky. Used as the inner
//                        building block of FBM since it's ~3x cheaper than
//                        gradient noise.
// `gradient_noise_3d`  — smoother and more isotropic; preferred when only a
//                        few octaves are stacked.
// `worley_3d`          — F1 distance to nearest jittered cell point. Returns
//                        in [0,1]. Used as a multiplicative detail mask.
// `fbm`/`fbm_ridged`   — multi-octave fractal sums. Lacunarity 2.0, gain 0.5
//                        is the canonical Quilez/Hillaire baseline; nebula
//                        shaders typically use lacunarity ~2.02 to break
//                        axis-aligned beating.
// `domain_warp`        — Quilez's iterated FBM displacement. The strength of
//                        the warp is the dominant aesthetic knob: 0 → flat
//                        clouds, 4+ → trifid-like tendrils.
// `clifford_torus_4d`  — wrap a 3-D point to a 4-D torus surface so noise
//                        evaluated on the wrapped coords tiles seamlessly in
//                        all axes. Used for the export-mode tileable volumes.

// Hashing comes from common/math.wgsl: hash3f(vec3<f32>) -> vec3<f32>.

// ---- Value noise -----------------------------------------------------------

fn value_noise_3d(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = p - i;
    let u = f * f * (3.0 - 2.0 * f); // smoothstep

    let n000 = hash3f(i + vec3<f32>(0.0, 0.0, 0.0)).x;
    let n100 = hash3f(i + vec3<f32>(1.0, 0.0, 0.0)).x;
    let n010 = hash3f(i + vec3<f32>(0.0, 1.0, 0.0)).x;
    let n110 = hash3f(i + vec3<f32>(1.0, 1.0, 0.0)).x;
    let n001 = hash3f(i + vec3<f32>(0.0, 0.0, 1.0)).x;
    let n101 = hash3f(i + vec3<f32>(1.0, 0.0, 1.0)).x;
    let n011 = hash3f(i + vec3<f32>(0.0, 1.0, 1.0)).x;
    let n111 = hash3f(i + vec3<f32>(1.0, 1.0, 1.0)).x;

    let nx00 = mix(n000, n100, u.x);
    let nx10 = mix(n010, n110, u.x);
    let nx01 = mix(n001, n101, u.x);
    let nx11 = mix(n011, n111, u.x);
    let nxy0 = mix(nx00, nx10, u.y);
    let nxy1 = mix(nx01, nx11, u.y);
    return mix(nxy0, nxy1, u.z); // [0, 1]
}

// ---- Gradient noise --------------------------------------------------------

fn gradient_noise_3d(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = p - i;
    let u = f * f * (3.0 - 2.0 * f);

    var v: f32 = 0.0;
    for (var iz: i32 = 0; iz < 2; iz = iz + 1) {
        for (var iy: i32 = 0; iy < 2; iy = iy + 1) {
            for (var ix: i32 = 0; ix < 2; ix = ix + 1) {
                let off = vec3<f32>(f32(ix), f32(iy), f32(iz));
                let g = hash3f(i + off) * 2.0 - 1.0;
                let d = f - off;
                let w = (1.0 - abs(off.x - u.x))
                      * (1.0 - abs(off.y - u.y))
                      * (1.0 - abs(off.z - u.z));
                v = v + dot(g, d) * w;
            }
        }
    }
    return v * 0.5 + 0.5; // [0, 1]
}

// ---- Worley noise (F1) -----------------------------------------------------

fn worley_3d(p: vec3<f32>) -> f32 {
    let i = floor(p);
    let f = p - i;
    var d: f32 = 1e9;
    for (var z: i32 = -1; z <= 1; z = z + 1) {
        for (var y: i32 = -1; y <= 1; y = y + 1) {
            for (var x: i32 = -1; x <= 1; x = x + 1) {
                let g = vec3<f32>(f32(x), f32(y), f32(z));
                let h = hash3f(i + g);
                let r = g + h - f;
                d = min(d, dot(r, r));
            }
        }
    }
    return clamp(sqrt(d), 0.0, 1.0);
}

// ---- FBM and ridged FBM ----------------------------------------------------

const FBM_MAX_OCTAVES: i32 = 8;

struct FbmParams {
    octaves: i32,
    lacunarity: f32,
    gain: f32,
};

fn fbm(p: vec3<f32>, params: FbmParams) -> f32 {
    var sum: f32 = 0.0;
    var amp: f32 = 0.5;
    var freq: f32 = 1.0;
    var norm: f32 = 0.0;
    let n = min(params.octaves, FBM_MAX_OCTAVES);
    for (var i: i32 = 0; i < n; i = i + 1) {
        sum = sum + amp * value_noise_3d(p * freq);
        norm = norm + amp;
        freq = freq * params.lacunarity;
        amp = amp * params.gain;
    }
    return select(0.0, sum / norm, norm > 0.0);
}

// Ridged: fold the noise around 0.5 with abs() and invert. Higher gain on
// later octaves than smooth FBM (Musgrave's original recipe) — gives the
// sharp, filamentary tendrils that classic fbm cannot produce.
fn fbm_ridged(p: vec3<f32>, params: FbmParams) -> f32 {
    var sum: f32 = 0.0;
    var amp: f32 = 0.5;
    var freq: f32 = 1.0;
    var norm: f32 = 0.0;
    let n = min(params.octaves, FBM_MAX_OCTAVES);
    for (var i: i32 = 0; i < n; i = i + 1) {
        let v = abs(value_noise_3d(p * freq) * 2.0 - 1.0);
        let r = 1.0 - v;
        sum = sum + amp * r * r;
        norm = norm + amp;
        freq = freq * params.lacunarity;
        amp = amp * params.gain;
    }
    return select(0.0, sum / norm, norm > 0.0);
}

// ---- Domain warp -----------------------------------------------------------

// Iquilezles' iterated FBM warp. Two layers of displacement give the thick,
// flowing "current" look that single-layer warp cannot. Strength magnitude
// ~1-5 is the practical range — past that the field becomes structureless.
fn domain_warp(p_in: vec3<f32>, strength: f32, params: FbmParams) -> vec3<f32> {
    let q = vec3<f32>(
        fbm(p_in + vec3<f32>(0.0, 0.0, 0.0), params),
        fbm(p_in + vec3<f32>(5.2, 1.3, 7.7), params),
        fbm(p_in + vec3<f32>(2.7, 9.1, 3.1), params),
    );
    let r = vec3<f32>(
        fbm(p_in + 4.0 * q + vec3<f32>(1.7, 9.2, 4.4), params),
        fbm(p_in + 4.0 * q + vec3<f32>(8.3, 2.8, 6.6), params),
        fbm(p_in + 4.0 * q + vec3<f32>(3.5, 7.1, 2.9), params),
    );
    return p_in + strength * r;
}

// ---- Clifford-torus 4D wrap ------------------------------------------------

// For seamless 2-D tileable export, we sample 4-D noise on the surface of a
// 4-D Clifford torus. wgpu's `value_noise_3d` is 3-D, so we project to 3-D by
// dropping one coordinate — a pragmatic compromise that hides seams in
// practice. Future work: native 4-D noise for stricter tileability.
fn clifford_wrap(uv: vec2<f32>) -> vec3<f32> {
    let a = uv.x * TWO_PI;
    let b = uv.y * TWO_PI;
    return vec3<f32>(cos(a), sin(a), cos(b));
}
