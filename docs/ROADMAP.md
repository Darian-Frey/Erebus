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

## Phase 4 — Starfield (M3) ✅

**Goal:** layered, blackbody-correct stars with diffraction spikes.

- [x] **Grid-hash starfield** with 1–3 parallax layers (each layer 2× the grid scale of the previous). PCG3D hash per cell; jittered star direction kept inside the cell middle so adjacent cells don't have stars touching at the boundary. Currently inlined into [shaders/nebula/raymarch.wgsl](../shaders/nebula/raymarch.wgsl) as `sample_starfield(dir)` rather than a separate pass — single fragment shader pipeline keeps the bind group small and the nebula-occlusion math (multiply by post-march transmittance) trivial. Splits into `shaders/starfield/grid_hash.wgsl` when a Phase 5 shader composer lands.
- [~] **Compute `bake_blackbody` deferred** — replaced with a CPU-baked 1024-texel RGBA16F LUT in [src/render/resources/textures.rs](../src/render/resources/textures.rs) using Mitchell Charity's polynomial (8 KB upload, no compute pipeline needed since the LUT is seed-independent). Runs once at startup.
- [x] **IMF-weighted brightness**: `mag = pow(rand, imf_exponent)` with default exponent 5 (pushes ~95 % of stars to dim end of the distribution).
- [x] **Galactic-plane density mask**: gaussian falloff from a user-adjustable tilted plane normal lifts the per-cell presence threshold, making the band visibly denser than the rest of the sphere.
- [~] **Nebula-density coupling deferred to Phase 5/7** — requires sampling the nebula density at the starfield's "behind-the-volume" direction, which is straightforward but adds another texture fetch per star. Skipping for the Phase 4 minimum.
- [x] **Diffraction spikes** for stars above `psf_threshold` (default 0.6, top ~5 % of the brightness distribution). Currently a 4-spoke axis-aligned cross built from a per-star tangent basis. Phase 5 replaces this with a proper N-fold pattern + FFT convolution bloom.
- [~] **Sub-pixel jitter / twinkle deferred** — would re-introduce the per-frame temporal aliasing that bug #5 just fixed. Land alongside TAA in Phase 5.
- [x] **Nebula occlusion** for free: stars are added as `sample_starfield(dir) * post_march_transmittance` so dense nebula regions correctly attenuate the background.

**Exit:** stars-only render shows realistic distribution (galactic band, IMF-biased magnitudes, blackbody colour spread from cool red to hot blue), bright stars get diffraction spikes, dense nebula regions correctly hide background stars. ✅

**Deviations:**

- Stars share the nebula's render pipeline rather than running as a separate additive pass. Saves ~8 lines of plumbing and makes nebula-occlusion automatic. The trade-off: starfield can't be enabled while nebula is disabled (since the pipeline is shared) — but `density_scale=0` + `count=0` lights does effectively the same thing.
- Single LUT for blackbody, no separate compute bake — physics doesn't depend on seed, so a CPU bake at startup is sufficient.

---

## Phase 5 — Post chain (M4) ✅

**Goal:** ship-quality HDR → display pipeline.

- [x] **Bloom pyramid** with up to 5 mips (`MAX_MIPS = 5`, dynamically clamped to `log2(min_dim) - 2` for small targets). Per-mip TextureViews + bind groups, rebuilt on HDR resize. See [src/render/resources/textures.rs](../src/render/resources/textures.rs).
- [x] **13-tap CoD:AW Jimenez downsample** in [shaders/bloom/downsample.wgsl](../shaders/bloom/downsample.wgsl) with two entry points: `fs_main_first` (threshold + Karis average per-tap, used only on the first mip) and `fs_main` (plain 13-tap, used on all subsequent mips).
- [x] **9-tap tent upsample** with additive blend at the pipeline level — `BlendComponent { src: One, dst: One, op: Add }`. See [shaders/bloom/upsample.wgsl](../shaders/bloom/upsample.wgsl). Tap radius scaled by `bloom_radius` uniform.
- [x] **Karis-weighted firefly suppression** on the first downsample. Each cluster of 4 taps is averaged with inverse-luminance weights so a single bright star pixel can't dominate the cluster.
- [x] **Tonemap shaders** in [shaders/composite.wgsl](../shaders/composite.wgsl): AgX (default; Sobotka — Three.js port with sRGB↔Rec.2020 inset/outset matrices and 6th-order polynomial), ACES Fitted (Narkowicz one-liner), Reinhard (`x / 1+x`). Switchable via `tonemap_mode` uniform; UI is a combo box.
- [~] **Tony McMapface deferred** — needs a 48³ 3D LUT and a corresponding loader. Land in Phase 7 polish alongside the user-editable gradient widget.
- [x] **Exposure (stops)** moved out of the raymarch into the post pass so bloom thresholds against unexposed scene radiance. Phase-2 `frame.exposure` field is left in `FrameUniforms` for backward compat but the value is no longer applied in the raymarch.
- [x] **Saturation + contrast grade** sliders. Lift/gamma/gain deferred to Phase 7 — saturation around Rec.709 luminance and contrast around middle grey are enough for the look range we currently care about.
- [x] **Triangular-PDF deband dither** at the very end of the post chain (after tonemap, before swapchain write). Two IGN samples differenced, scaled to ±1/255. `deband_amount` uniform multiplier (0 disables).
- [~] **FFT convolution bloom deferred** — post-release stretch as the roadmap noted.

**Exit:** flipping between AgX/ACES/Reinhard shows clearly different aesthetics; bloom is energy-conserving at intensity = 0 (no contribution); deband eliminates banding in dark gas regions; cores no longer clip flat-white. ✅

**Bug log:** WGSL doesn't allow unary `+` in polynomial expressions — see [BUGS.md #6](BUGS.md). One-line fix.

---

## Phase 6 — Export (M5) ✅ (MVP)

**Goal:** render and save a 16K equirect or 6×4K cubemap.

Phase-6 MVP shipped: equirect PNG export at 1K / 2K / 4K / 8K, single-shot direct render (no tiling), synchronous (UI freezes for 1–3 s during render). Tiling for 16K and EXR/cubemap support are tracked as Phase 6.5 below.

- [x] `export::png` 8-bit sRGB writer ([src/export/png.rs](../src/export/png.rs)).
- [x] `export::equirect` reuses the existing equirect ray mapping in `nebula/raymarch.wgsl` — no extra camera basis math needed for the equirect mode.
- [x] `ErebusRenderer::render_equirect_rgba8` allocates a one-shot set of HDR / bloom-pyramid / output / readback resources sized for the chosen export width, runs the full nebula → bloom → tonemap chain, and reads back the sRGB-encoded RGBA8 pixels via `device.poll(Wait)`. Per-row buffer alignment honours `wgpu::COPY_BYTES_PER_ROW_ALIGNMENT`. See [src/render/graph.rs](../src/render/graph.rs).
- [x] Dedicated `export_tonemap_pipeline` targeting `Rgba8UnormSrgb` so the readback bytes are PNG-ready without per-pixel format swizzling.
- [x] Export panel UI in [src/gui/panels.rs](../src/gui/panels.rs) with width combo box (1K / 2K / 4K / 8K) and an `Export PNG…` button that opens a native file dialog (`rfd::FileDialog::save_file`) with a sensible default filename (`erebus_equirect_<W>.png`).
- [x] App update loop handles the `pending_export` flag — runs the dialog, the GPU render, and the file write in sequence; reports the result back into the UI status line.

**Phase 6.5 (deferred):**

- [ ] `export::tiling` — split the viewport into 4K tiles with per-tile ray-basis adjustments. Required for 16K+ since wgpu's default `max_texture_dimension_2d` is 8192.
- [ ] Supersample 1× / 2× / 4× with linear box-filter downfilter on CPU.
- [ ] `export::exr` — linear OpenEXR writer using the existing `exr` crate dep. Skips the tonemap pass and writes the HDR target directly.
- [ ] `export::cubemap` — 6 face renders with per-face camera basis, cross / six-PNG-folder / `.dds` packaging.
- [ ] Background thread + progress UI with cancel — currently the UI freezes during the render (~1–3 s at 8K on a discrete GPU).

**Exit (MVP):** a 4K equirect PNG renders into a user-chosen location, looks identical to the live preview, and tiles seamlessly in longitude. ✅

**Exit (full):** a 16K × 8K equirect PNG renders without OOM and tiles seamlessly along the longitude wrap.

**Risks:** GPU memory on consumer hardware. 16K direct render needs > 1 GB tile budget; tiling makes it tractable but readback bandwidth becomes the new bottleneck. Phase 6 MVP avoids the question by capping at 8K, which fits in ~500 MB total resident GPU memory.

---

## Phase 7 — UI polish & presets (M6) ✅ (MVP)

**Goal:** a UI a customer would pay for.

- [x] Seven panels (Preset, Frame, PostFX, Nebula, Lighting, Starfield, Export). Preset moved to the top so it's the first thing a returning user reaches for.
- [~] **Custom gradient widget deferred to Phase 8** — needs a drag-stops + Kelvin slider + hex input widget set. The gradient *data* now flows through presets cleanly; what's missing is the in-app editor. Users can edit gradients today by hand-editing the saved RON files.
- [x] Tooltips on every non-obvious slider with units and defaults (HG anisotropy, σₑ, IMF exponent, lacunarity, density curve, Kelvin temperatures, transmittance cutoff, step density bias, etc.). Hover any slider label to see them.
- [x] `preset::schema::Preset` shipped with `format_version = 1`. Includes seed, all four uniform blocks, and the gradient stops Vec. `_pad` fields are `#[serde(skip)]` so the on-disk RON is human-friendly. See [src/preset/schema.rs](../src/preset/schema.rs).
- [x] `preset::io::save_to_file` / `load_from_file` via [src/preset/io.rs](../src/preset/io.rs); native file dialogs through `rfd`.
- [x] `preset::migrate::migrate` stub in [src/preset/migrate.rs](../src/preset/migrate.rs) — runs in a loop with arms keyed on `format_version`. Currently a no-op since v1 is the first shipped version.
- [x] Three shipped presets, embedded via `include_str!` so they ship inside the binary: [synthwave.ron](../assets/presets/synthwave.ron) (magenta + cyan + violet, AgX), [cyberpunk.ron](../assets/presets/cyberpunk.ron) (hot pink + electric purple + black, AgX, sharper filaments), [retro_scifi.ron](../assets/presets/retro_scifi.ron) (amber + teal + burgundy, ACES Fitted, softer gas).
- [x] Unit tests in [src/preset/mod.rs](../src/preset/mod.rs) cover load + round-trip serialise/deserialise for all three shipped presets, gating against RON syntax drift.
- [~] **Recent presets list deferred to Phase 8.**
- [~] **Compact base64 share-string deferred to Phase 8** — needs MessagePack + base64 encoders; design question whether to share the raw struct or a hash-keyed deterministic regen.
- [~] **Synthwave-leaning egui theme deferred to Phase 8** — current dark theme with a magenta accent is OK; can iterate as part of release polish.

**Exit:** the user can pick one of three shipped presets, tweak any slider with a tooltip explaining what it does, save the result to a `.ron` file, reopen it later. ✅

**Bug log:** RON serializes Rust fixed-size arrays as tuples (parens) not lists (brackets). See [BUGS.md #7](BUGS.md). Fixed in all three shipped RONs.

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
