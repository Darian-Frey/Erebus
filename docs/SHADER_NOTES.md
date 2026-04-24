# Shader Notes

Tuning notes, parameter ranges, and reference links for each shader pass. Living document — extend as the look develops.

## `nebula/raymarch.wgsl`

- **Step count**: 64 = preview, 128 = quality, 256 = export. Dense regions need more; consider density-adaptive stepping (step by `1/density`) once base look is settled.
- **March length** in scene units: 1.0 typical; the noise volume tiles seamlessly because we sample by direction.
- **Phase function**: Henyey–Greenstein with `g ∈ [-0.9, 0.9]`. `g = 0` is isotropic; `g = 0.6` reads as forward-scattering dust.
- **Beer–Lambert step**: `transmittance *= exp(-σₑ * dt)`. Keep σₑ user-facing as "extinction".
- Reference: Hillaire SIGGRAPH 2016, ShaderToy `XlBSRz`.

## `nebula/density.wgsl`

- Compose: `ridged_fbm(p) * (1 - worley(p * scale))` for filaments + cellular voids.
- Domain warp: displace `p` by `vec3(fbm(p), fbm(p + 17), fbm(p + 43))` × warp strength.
- Curl noise contribution: small additive offset; too much and the swirl becomes noise.
- Reference: Quilez domain warping (`iquilezles.org/articles/warp`).

## `nebula/lighting.wgsl`

- 8 shadow steps is plenty; the inner volume self-shadows from the main march already.
- Light radius is artistic — physically point lights, but we let users place 1–4 lights with falloff exponent 1.0–4.0.

## `starfield/grid_hash.wgsl`

- 3 parallax layers with relative scale 1×, 0.5×, 0.25× and brightness multiplier 1.0, 0.5, 0.25.
- Hash function: PCG3D — small, fast, no visible repetition at our scales.
- IMF biasing: `temperature = mix(2700, 30000, pow(rand, 3.0))`.

## `starfield/psf.wgsl`

- Threshold for PSF billboard: top ~5% brightest stars. Below threshold, single-pixel emission is fine because bloom catches them.
- Spike count user-exposed (4 / 6 / 8). 6-spoke matches JWST aesthetic.
- Reference: tiffnix.com/star-rendering.

## `bloom/`

- 13-tap downsample is the CoD:AW Jimenez kernel. Karis average only on the top mip.
- 8 mips at 4K = 6×6 pixel coarsest mip; further reduction adds nothing.
- Tent upsample radius 1.0; intensity scaled to maintain energy conservation.

## `tonemap/`

- AgX: default. Uses the Sobotka 1D LUT approximation; Bevy ships an open-source WGSL port we can adapt.
- ACES: Narkowicz one-liner. Useful as the "industry standard" comparison option.
- Tony McMapface: 3D LUT. Loads 48×48×48 RGBA16F texture at startup.
- Reinhard: comparison only. Do not use as default.
- Deband dither: uniform triangular noise on a per-channel basis, ±0.5/255 amplitude.
