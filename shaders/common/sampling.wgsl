// Sampling helpers for the nebula pass.
//
// `henyey_greenstein` — single-lobe HG phase function. g ∈ (-1, 1).
//                       g > 0 forward-scatter; g = 0 isotropic; g < 0 back.
// `ign_dither`        — Jorge Jimenez's interleaved-gradient noise.
//                       Cheap GPU-side blue-noise substitute used to jitter
//                       the raymarch start position without a texture.

const HG_EPS: f32 = 1e-3;

fn henyey_greenstein(cos_theta: f32, g: f32) -> f32 {
    let g2 = g * g;
    let denom = 1.0 + g2 - 2.0 * g * cos_theta;
    return (1.0 - g2) / (4.0 * 3.14159265 * pow(max(denom, HG_EPS), 1.5));
}

// Interleaved gradient noise (Jorge Jimenez, SIGGRAPH 2014). Returns [0,1).
// Properties close to a 64×64 blue-noise tile and free of axis-aligned bands.
fn ign_dither(pixel: vec2<f32>) -> f32 {
    return fract(52.9829189 * fract(0.06711056 * pixel.x + 0.00583715 * pixel.y));
}

// Per-frame variant — adds a hashed integer offset so the noise field rotates
// across frames, turning residual spatial banding into temporal sparkle that
// bloom and the human eye smear away.
fn ign_dither_temporal(pixel: vec2<f32>, frame: u32) -> f32 {
    let f = f32(frame & 1023u);
    return fract(52.9829189 * fract(0.06711056 * (pixel.x + f * 0.754877)
                                  + 0.00583715 * (pixel.y + f * 0.569841)));
}
