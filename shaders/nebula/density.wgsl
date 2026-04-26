// Density function for the nebula raymarch.
//
// Phase 2: this file is illustrative documentation — the actual density
// function is inlined into nebula/raymarch.wgsl so we don't need a WGSL
// preprocessor yet. When starfield + bloom land in Phase 4-5 we'll add a
// shader-source composer (common/*.wgsl auto-prepended) and split density
// into this file. Until then, edit raymarch.wgsl.
//
// The composition the raymarch uses, from research-driven defaults:
//
//   p_warped = p + warp_strength * fbm_vec3(p, octaves_warp)
//   shape    = mix(fbm(p_warped, octaves_density),
//                  fbm_ridged(p_warped, octaves_density),
//                  ridged_blend)
//   density  = max(0, shape - cutoff) * density_scale
//
// Tuning notes:
//   * 6 octaves density / 3 octaves warp — past 8 the cost grows without
//     visible detail (gain 0.5 means the 9th octave contributes <0.2%).
//   * Lacunarity 2.02 (Duke's anti-grid trick) breaks axis-aligned beating
//     that pure 2.0 produces on cardinal slices.
//   * The `mix(smooth, ridged, 0.5)` blend gives the wispy-gas + filament
//     mix that pure smooth FBM (clouds) and pure ridged (lightning) cannot.
