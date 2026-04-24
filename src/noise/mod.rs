// CPU-side noise utilities: blue-noise sample generation, blackbody->sRGB
// LUT baking, deterministic seeding helpers. GPU noise lives in WGSL.

pub mod blackbody;
pub mod blue_noise;
pub mod seed;
