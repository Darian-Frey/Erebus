# Shader Notes

Tuning notes, parameter ranges, and reference links for each shader pass. Living document — extend as the look develops.

## `nebula/raymarch.wgsl`

Shipped Phase 2 defaults (research-driven; see Phase 2 entry in ROADMAP for source confidence flags):

| Parameter | Value | Notes |
| --- | --- | --- |
| Steps | 96 preview / 128 quality / 256 export | Fixed in UI; range 16–256. |
| March length | 1.0 scene unit | Normalised; density scale rescales the noise field instead. |
| `transmittance_cutoff` | 0.01 | Early-out gives 30–50 % saving in dense regions. Disable for export to keep tile timing deterministic. |
| `step_density_bias` | 1.5 | `dt = base * max(0.25, bias - density)` — denser → smaller steps. |
| Phase function | HG single-lobe, default g = 0.6 | Forward-scatter "dust" look. `g = 0` = isotropic clouds. |
| Extinction σₑ | 1.5 (default), 0.1–8.0 range | Multiplied by per-sample density. |
| Albedo | 0.6 | Space Engine's default; roughly Frostbite cloud value. |
| Density curve γ | 0.5 (sqrt) | Lifts wispy tails (the visual signature) without blowing the core. |
| Anti-banding | IGN dither × 1× dt at march entry, frame-indexed offset | Real blue-noise tile lands in Phase 5. |

References: Hillaire SIGGRAPH 2016 (ShaderToy `XlBSRz`); Pegwars *Rendering Nebulae*; Maxime Heckel cloudscapes; Duke `Dusty Nebula 4`.

## `nebula/density.wgsl`

Composition (currently inlined into `raymarch.wgsl`, will split when starfield arrives). **Phase 3.5 update:** the FBM is now baked into a 128³ RGBA16F volume by [shaders/compute/bake_3d_noise.wgsl](../shaders/compute/bake_3d_noise.wgsl); the runtime density function does **4 trilinear texture fetches** (3 warp + 1 main shape), down from ~21 procedural noise evaluations.

```text
seed_off   = derived from frame.seed
p_scaled   = p * density_scale + seed_off
warp       = vec3(R(p_scaled), R(p_scaled + 5.2,1.3,7.7), R(p_scaled + 2.7,9.1,3.1)) * 2 - 1
p_warped   = p_scaled + warp_strength * warp
(smooth, ridged) = sample_noise(p_warped).rg
shape      = mix(smooth, ridged, ridged_blend)
density    = max(shape - 0.45, 0.0) * 1.8
```

The bake covers world-space [0, 8)³ at 128³ samples; the volume sampler uses REPEAT addressing so warp offsets that fall outside one period wrap cleanly. The bake re-runs whenever **seed**, **octaves**, **lacunarity**, or **gain** changes — typically ~5 ms on the dev T1200. **density_scale**, **warp_strength**, and **ridged_blend** stay runtime knobs (no re-bake).

| Parameter | Default | Range | Notes |
| --- | --- | --- | --- |
| Density scale | 1.6 | 0.1–8.0 | Higher → finer detail, fewer big shapes. |
| Octaves (density) | 6 | 1–8 | Past 8 the gain-0.5 contribution is < 0.2 %. |
| Lacunarity | 2.02 | 1.5–2.5 | 2.02 (not 2.0) breaks axis-aligned beating on cardinal slices — Duke trick. |
| Gain | 0.5 | 0.2–0.7 | Universal across sources. |
| Ridged blend | 0.5 | 0–1 | 0 = wispy gas (clouds), 1 = filament/lightning. |
| Warp strength | 1.5 | 0–4 | Lower than IQ's 4.0 because we already have ridged + future curl. |
| Octaves (warp) | 3 | 0–6 | 0 → flat clouds, 3 → trifid-style tendrils. |

The 0.45 cutoff and 1.8 scaler at the end are empirical — they remap typical FBM output [~0.3, 0.85] to roughly [0, 0.7] of useful density. Tune per preset.

## `nebula/lighting.wgsl`

Phase 3: in-volume point lights with shadow marching. Currently inlined into `raymarch.wgsl` as `sample_lights()`; will split when the shader composer lands (Phase 4).

| Parameter | Default | Range | Notes |
| --- | --- | --- | --- |
| `count` | 2 | 0–4 | Number of active lights. Inactive slots are skipped. |
| `shadow_steps` | 6 | 1–12 | Per-light shadow march; midpoint sampling. Early-out at optical depth > 6. |
| `ambient_emission` | 0.25 | 0.0–1.5 | Isotropic self-glow floor. 0 = lit-only ("Horsehead silhouette" mode); 1+ = Phase-2 self-glow look. |
| Light `intensity` | 4.0 (key), 1.5 (fill) | 0–10 | Multiplied with colour. 0 → light skipped. |
| Light `falloff` | 2.0–2.5 | 0–4 | `1 / dist^falloff`. 2 = inverse-square (physical); 0 = no falloff (directional). |

**In-scatter equation per main-march sample:**

```text
for each active light:
    L = normalize(light.pos - p)
    phase = HG(dot(view, L), g)
    shadow_t = exp(-Σ σₑ · density(p + L·k·dt) · dt)   // shadow march toward light
    falloff = 1 / dist^light.falloff
    contribution += light.color * light.intensity * phase * shadow_t * falloff

colour += transmittance * absorbed * (in_scatter * albedo_color * albedo
                                    + albedo_color * ambient_emission)
```

The `albedo_color` from the gradient LUT colours both the self-glow and the per-light scattered radiance, so different parts of the nebula tint the lights differently — the same way real interstellar gas absorbs Hα and re-emits in characteristic emission lines.

**Why 6 shadow steps:** Heckel's measurement is that 4 steps shows visible step banding in shadows; 8 is invisibly clean; 6 is the sweet spot for cost vs quality. Combined with the `optical > 6` early-out we typically only run 2–3 steps in dense regions.

## `starfield/grid_hash.wgsl`

Phase 4 ships starfield inlined into `nebula/raymarch.wgsl` as `sample_starfield(dir)`. Splits into its own file when a shader composer arrives.

| Parameter | Default | Range | Notes |
| --- | --- | --- | --- |
| `density` | 80.0 | 20–200 | Grid scale of layer 0; doubles each layer. Higher = more, smaller stars. |
| `brightness` | 1.0 | 0–4 | Global multiplier on all layers. |
| `layers` | 3 | 1–3 | Number of parallax octaves. |
| `imf_exponent` | 5.0 | 1–8 | `mag = pow(rand, exp)`. 5 → ~95 % dim, 1 → uniform. |
| `galactic_strength` | 1.5 | 0–4 | Lift to per-cell presence threshold inside the band. |
| `galactic_width` | 0.3 | 0.05–1 | Gaussian falloff away from the plane. |
| `galactic_dir` | (0.3, 1.0, 0.2) | unit-ish | Plane normal — tilt the band. |

Per-cell logic: PCG3D hash of `(cell_index, frame.seed ^ layer_seed)` gives `(presence, magnitude, colour_t)`. If `presence < threshold(galactic_band)`, no star. Otherwise the star direction is `(cell + 0.6·rand + 0.2) / scale` normalised — kept inside the cell middle so adjacent stars don't kiss at the boundary.

Star core falloff: `exp(-ang_sq * scale² * 100)` so cells render at roughly constant pixel size as `density` changes.

**Stars get nebula occlusion for free** because they're added to the fragment colour as `sample_starfield(dir) * post_march_transmittance` — the residual transmittance after the volumetric march is exactly the fraction of background light reaching the camera.

## `starfield/psf.wgsl`

| Parameter | Default | Range | Notes |
| --- | --- | --- | --- |
| `psf_threshold` | 0.6 | 0–1 | Stars above this magnitude get diffraction spikes (~5 % of distribution). |
| `psf_intensity` | 0.4 | 0–2 | Spike multiplier on top of core brightness. |
| `spike_count` | 4 | 4–8 | Currently the WGSL builds a 4-spoke axis-aligned cross; the slider is plumbed for Phase 5's N-fold pattern + FFT bloom. |
| `spike_length` | 0.012 | 0.001–0.05 | Angular extent of each spike. |

Phase-4 spike construction: build a tangent basis on the unit sphere from the star direction (avoiding pole degeneracy by switching the up-vector when `|star.y| > 0.95`), project the ray-to-star delta into `(tx, ty)`, and `max(exp(-ay·600) · smoothstep(spike_length, 0, ax), exp(-ax·600) · smoothstep(spike_length, 0, ay))`. Cheap; not physically correct (a real airy disk has the right Bessel-function radial profile + 6-fold spikes from JWST-style hexagonal apertures). Phase 5's FFT convolution bloom replaces this with proper diffraction.

## `starfield/blackbody`

CPU-baked 1024-texel 1D RGBA16F LUT spanning 1000 K → 40000 K, baked once at startup in [src/render/resources/textures.rs](../src/render/resources/textures.rs) using Mitchell Charity's regression polynomial. Sampled in the fragment via linear filtering. The temperature for each star is currently keyed off the same hash as magnitude (`h.y`) so bright stars correlate with hot blue colours — a physically reasonable approximation of the mass-luminosity relationship without computing it explicitly.

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
