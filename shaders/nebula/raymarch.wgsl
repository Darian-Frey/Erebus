// Volumetric raymarch fragment shader. Per-pixel: ray dir from UV, march N
// steps, accumulate emissive*transmittance via Beer-Lambert + HG scattering,
// in-volume light shadow march, blue-noise dither, write HDR RGBA16F.
