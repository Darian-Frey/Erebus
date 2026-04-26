# Erebus Development Roadmap

A phased plan from empty repo to itch.io release. Each phase has a concrete deliverable, an exit criterion, and a list of open risks. Phases overlap where work is parallelizable.

The ordering is deliberate: get pixels on screen before polishing UI, get HDR right before adding bloom, get bloom right before exposing tone-map choice. Visual quality is bounded by shader authoring (see compass artifact §6), so every phase ends with a visual-review pass, not just a green test run.

---

## Phase 0 — Foundation (week 0)

**Goal:** clean repo, building binary, CI green.

- [x] Repository skeleton: `src/`, `shaders/`, `assets/`, `docs/`, `tests/`, `benches/`, `examples/`.
- [x] `Cargo.toml` with `wgpu`, `eframe`, `egui`, `bytemuck`, `glam`, `serde`, `ron`, `image`, `exr`, `rfd`.
- [x] Dual MIT / Apache-2.0 license.
- [x] `.gitignore`, `rustfmt.toml`, `rust-toolchain.toml`.
- [x] GitHub Actions CI: fmt, clippy, test, wasm-check.
- [x] `cargo run` opens an `eframe` window — done as part of Phase 1 below.

**Exit:** `cargo build --release` succeeds on Linux/macOS/Windows; CI green.

---

## Phase 1 — Render plumbing (M1) ✅

**Goal:** a fullscreen WGSL shader rendering into HDR RGBA16F, hot-reloadable.

- [x] `render::context` exposes the shader-source root; eframe owns device/queue/adapter init.
- [x] `render::resources::textures::HdrTarget` allocates the HDR RGBA16F target and resizes on viewport change.
- [x] `shaders/fullscreen.wgsl` (oversized triangle) + Phase 1 placeholder fragment in `shaders/nebula/raymarch.wgsl` (UV+time gradient).
- [x] `render::hot_reload::ShaderWatcher` (notify, 150 ms debounce). Naga front-end validates before pipeline rebuild; errors surface in a bottom panel without crashing the app.
- [x] `render::uniforms::FrameUniforms` (resolution, time, exposure, seed) with C-layout matched WGSL.
- [x] `tests/wgsl_validation.rs` parses every shader through Naga in CI.
- [x] Preview shows the gradient via egui paint callback; resizing works (HDR target rebuilt on size change).

**Exit:** edit a WGSL file → preview updates within ~150 ms without app restart. ✅

**Risks:** wgpu/eframe surface sharing — pinned to wgpu 0.20 / eframe 0.28 / egui 0.28.

---

## Phase 2 — Nebula MVP (M2) ✅

**Goal:** a recognisable volumetric nebula on screen, parameter-driven.

- [x] Compute pass `bake_3d_noise` producing a 128³ RGBA16F volume — pulled forward from the Phase-6 deferral when Phase-3 shadow marching exposed the per-pixel cost ceiling of procedural noise (full-res preview dropped to ~6 fps). Bake stores 6-octave smooth FBM in R, ridged FBM in G; runtime samples 4× per main density (3 warp + 1 main) and 1× per shadow step. Re-bake triggers on seed/octaves/lacunarity/gain change; everything else stays runtime. See [shaders/compute/bake_3d_noise.wgsl](../shaders/compute/bake_3d_noise.wgsl).
- [~] **Deferred.** Compute pass `bake_gradient`. The 256-texel LUT is currently CPU-baked once at startup ([src/render/resources/textures.rs](../src/render/resources/textures.rs)); a compute path lands when the user-editable gradient widget arrives in Phase 7.
- [x] [shaders/nebula/raymarch.wgsl](../shaders/nebula/raymarch.wgsl): equirect ray direction, configurable 16–256 steps, FBM density, sqrt-mapped gradient LUT lookup, Beer–Lambert + HG single-scatter accumulation, IGN dither at entry, density-adaptive step length, transmittance early-out.
- [x] [shaders/nebula/density.wgsl](../shaders/nebula/density.wgsl): currently inlined into `raymarch.wgsl` with a comment pointer; split into its own module when starfield (Phase 4) introduces a shader-source composer.
- [x] [shaders/common/noise.wgsl](../shaders/common/noise.wgsl): value/gradient noise, Worley, FBM, ridged FBM, domain warp, Clifford-torus wrap. Available as a reference module; `raymarch.wgsl` inlines the parts it uses.
- [x] [shaders/common/sampling.wgsl](../shaders/common/sampling.wgsl): Henyey–Greenstein, IGN dither (with temporal variant). [shaders/common/math.wgsl](../shaders/common/math.wgsl) covers PCG3D hash + equirect mappings.
- [x] Blue-noise stand-in via Jorge Jimenez's interleaved-gradient noise, jitter amplitude = 1× step length, frame-indexed offset for temporal de-banding. Real blue-noise texture deferred until Phase 5 needs it for bloom.

**Exit:** screen shows a nebula that visibly responds to seed, density, gradient, warp, anisotropy and exposure; no obvious step banding at 96+ steps. ✅

**Defaults baked in** (research-driven, see [docs/SHADER_NOTES.md](SHADER_NOTES.md)): 6/3 octaves, lacunarity 2.02, gain 0.5, ridged blend 0.5, warp strength 1.5, 96 steps preview, σₑ 2.0, albedo 0.6, HG g 0.6, sqrt density curve, transmittance cutoff 0.01.

**Risks materialised:** nothing severe yet. Procedural noise instead of baked is the one notable deviation; revisit if 4 K preview drops below 30 FPS on integrated GPUs (Phase 8 benchmark).

---

## Phase 3 — Lighting (M2 cont.) ✅

**Goal:** depth and drama via in-volume light.

- [x] 1–4 user-placed point lights inside the volume (`array<PointLight, 4>` in [src/render/uniforms.rs](../src/render/uniforms.rs)). UI exposes `count` 0–4, per-light position/colour/intensity/falloff, ambient-emission floor.
- [x] Per-main-march-sample shadow march toward each active light: midpoint sampling, configurable shadow_steps (1–12, default 6 per Heckel), early-out when shadow optical depth > 6 (transmittance < 0.0025).
- [x] HG anisotropy slider already shipped in Phase 2 (`g ∈ [-0.9, 0.9]`); now actually drives per-light scattering instead of the Phase-2 fixed key direction.
- [x] Emissive density falloff via `ambient_emission` uniform (0 = lights only, 1.5 = bright self-glow). Combined with the gradient LUT this acts as a wavelength-dependent self-emission floor — the gradient colours both the self-glow and the per-light albedo tint.
- [~] Standalone `nebula/lighting.wgsl` deferred — `sample_lights()` is currently inlined into [shaders/nebula/raymarch.wgsl](../shaders/nebula/raymarch.wgsl) for the same reason as `density.wgsl`; splits when the Phase-4 starfield introduces a shader-source composer.

**Exit:** turning lights on noticeably reshapes the volume — bright cores at light positions, dark dust lanes on the far side from each light, HG slider visibly redirects highlights. ✅

**Performance note:** at default settings (96 main steps × 2 lights × 6 shadow steps) we evaluate `nebula_density` ~1100 times per pixel per frame, ~1 G evals/s at 1080p/60. Ran fine on the dev T1200 with no measurable regression vs Phase 2; if 4 K preview slows down on integrated GPUs in Phase 8 benchmarks, the cheap lever is reducing shadow_steps to 4 and disabling the warp inside the shadow density function.

---

## Phase 4 — Starfield (M3)

**Goal:** layered, blackbody-correct stars with diffraction spikes.

- [ ] `shaders/starfield/grid_hash.wgsl`: 3-level grid hash with parallax depth offsets.
- [ ] Compute pass `bake_blackbody` producing a 1024-texel blackbody LUT (1000K–40000K).
- [ ] IMF-weighted brightness sampler (`pow(rand, 3.0)` baseline).
- [ ] Galactic-plane density mask (tilted band, FBM-modulated).
- [ ] Optional density coupling: bright stars cluster inside nebula gas (Spacescape technique).
- [ ] `shaders/starfield/psf.wgsl`: PSF billboard for stars above brightness threshold; airy disk + N-fold spikes; size cropped by inverse-square.
- [ ] Sub-pixel jitter for "twinkle" / re-AA.

**Exit:** a stars-only render at 4K survives a side-by-side comparison with a real Hubble background — colour distribution and density gradient look right.

---

## Phase 5 — Post chain (M4)

**Goal:** ship-quality HDR → display pipeline.

- [ ] Bloom pyramid: 6–8-mip downsample (13-tap CoD:AW) + tent upsample composite.
- [ ] Karis average on the brightest mip to suppress firefly stars.
- [ ] Tone-map shaders: AgX (default), ACES Fitted, Tony McMapface, Reinhard.
- [ ] Exposure (stops) slider as the primary brightness control.
- [ ] Optional grade: lift / gamma / gain or simple sat/contrast.
- [ ] Triangular-PDF deband dither at the end of the chain.
- [ ] **Deferred:** FFT convolution bloom for cinematic spikes (post-release stretch).

**Exit:** flipping between tone-maps shows clearly different aesthetics, none clip; bloom is energy-conserving (intensity 0 ≡ no bloom).

---

## Phase 6 — Export (M5)

**Goal:** render and save a 16K equirect or 6×4K cubemap.

- [ ] `export::tiling` splits the export viewport into 4K tiles with adjusted ray basis per tile.
- [ ] Supersample at 1×, 2×, 4× (linear downfilter on CPU).
- [ ] `export::png` 8-bit sRGB writer.
- [ ] `export::exr` linear OpenEXR writer (16- or 32-bit float).
- [ ] `export::equirect` and `export::cubemap` per-mode camera basis.
- [ ] Optional cube-cross or six-PNG-folder packaging; optional `.dds`.
- [ ] Progress UI with cancel.

**Exit:** a 16K × 8K equirect PNG renders without OOM and tiles seamlessly along the longitude wrap.

**Risks:** GPU memory on consumer hardware. 16K direct render needs > 1 GB tile budget; tiling makes it tractable but readback bandwidth is the new bottleneck.

---

## Phase 7 — UI polish & presets (M6)

**Goal:** a UI a customer would pay for.

- [ ] Six panels: Nebula, Starfield, Lighting, PostFX, Export, Presets.
- [ ] Custom gradient widget: drag-stops, Kelvin slider, hex input, copy/paste.
- [ ] Per-slider tooltips with units (Kelvin, stops, anisotropy).
- [ ] `preset::schema` finalised; `format_version` = 1.
- [ ] `preset::io` save/load via `rfd`; recent presets list.
- [ ] `preset::migrate` stub for future versions.
- [ ] Three shipped presets: synthwave, cyberpunk, retro_scifi.
- [ ] Compact 200-char preset string (base64 of MessagePack-encoded values) for sharing.
- [ ] Synthwave-leaning egui theme.

**Exit:** non-technical user can load a preset, tweak two sliders, and export a 4K PNG — without reading docs.

---

## Phase 8 — Performance & benchmarks

**Goal:** known performance envelope across hardware tiers.

- [ ] `benches/raymarch.rs` reports ms/frame at 1080p / 4K for 64 / 128 / 256 steps.
- [ ] Profile on integrated (Intel Iris / Apple M-series), mid-range (RTX 3060 / RX 6700), and high-end (RTX 4080).
- [ ] Adaptive preview: drop resolution while sliders move, restore on release.
- [ ] Document quality presets: Draft (64 steps, half-res), Preview (128 steps, full-res), Export (256+ steps, supersampled).

**Exit:** preview holds 30+ FPS on integrated GPU at 720p / 64 steps.

---

## Phase 9 — Web build (M7)

**Goal:** browser demo of the same binary.

- [ ] `wasm32-unknown-unknown` target builds clean (already gated in CI).
- [ ] WebGPU detection with a graceful "browser unsupported" fallback page.
- [ ] File save via browser download API (`rfd` WASM backend).
- [ ] Reduced default resolution and step count for browser.
- [ ] Asset embedding (`include_bytes!`) so the WASM bundle is self-contained.
- [ ] Compile-size audit: target <10 MB gzipped.

**Exit:** itch.io HTML upload of the WASM bundle runs the full preview at acceptable FPS in Chrome 113+ / Safari 26 / Firefox 145.

---

## Phase 10 — Release (M8)

**Goal:** ship to itch.io at $8–$15.

- [ ] Demo gallery: 12+ curated stills covering aesthetic range.
- [ ] Trailer / GIF reel.
- [ ] Itch.io page copy, screenshots, system requirements.
- [ ] Native installers: signed Windows `.exe`, notarised macOS `.dmg`, Linux `.AppImage` + `.tar.gz`.
- [ ] Public web demo (Phase 9 build) embedded on the itch.io page.
- [ ] Changelog and version-1.0 tag.
- [ ] Post-release roadmap: animated nebulae (4D noise time slice), procedural galaxies, ray-marched dust silhouettes against bright stars.

**Exit:** v1.0 release. Customers can buy and download.

---

## Cross-cutting concerns

- **Determinism**: every preset produces identical pixels for a given seed across all platforms. Verified by a hash-of-PNG test fixture in CI on representative presets. Drift on platform-specific intrinsics is the most likely failure; use `f32` carefully and avoid platform-divergent functions.
- **HDR everywhere**: never write to an 8-bit intermediate; always RGBA16F until the final tonemap+dither step. This is the single largest determinant of output quality (compass artifact §6.2).
- **Shader test discipline**: `tests/wgsl_validation.rs` parses every shader through Naga on every CI run. Catches regressions before they reach a device.
- **No hidden state**: every visible visual property maps to a serialisable preset field. No "magic" hard-coded numbers in shaders; all live in uniforms.
- **Visual review at every phase exit**: produce a still image, compare it to references in the compass artifact, and iterate before declaring the phase done.
