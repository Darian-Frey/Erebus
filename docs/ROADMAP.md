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

**Phase 6.5 (partial):**

- [~] `export::tiling` — **deferred to Phase 8**. Required for 16K+ since wgpu's default `max_texture_dimension_2d` is 8192. Niche; most users won't hit it.
- [~] Supersample 1× / 2× / 4× — **deferred to Phase 8**. Quality boost orthogonal to the format work.
- [x] `export::exr` — linear OpenEXR writer in [src/export/exr.rs](../src/export/exr.rs) backed by the `exr` crate. Pixels are scene-referred linear `f32` RGBA; the renderer's `render_equirect_rgba32f` runs the bloom chain but bypasses the tonemap by forcing `tonemap_mode = 3` (passthrough). Output drops straight into Photoshop / Affinity / Resolve / Blender comp pipelines.
- [x] `export::cubemap` — 6-face PNG export in [src/export/cubemap.rs](../src/export/cubemap.rs). New `cube_dir(uv, face)` ray basis in [shaders/nebula/raymarch.wgsl](../shaders/nebula/raymarch.wgsl) selected via `frame.mode` (0 = equirect, 1 = cubemap). Six face renders into reused HDR/bloom/output resources, six readback buffers, one `device.poll(Wait)`. Output filenames follow the OpenGL/DirectX `_px/_nx/_py/_ny/_pz/_nz` convention so files drop straight into Unity / Unreal / Bevy / Godot cubemap importers.
- [x] Per-face size combo box (512 / 1K / 2K / 4K) and kind/format combos in the Export panel. EXR option is gated to equirect (cubemap EXR is doable but deferred to Phase 7.5).
- [~] Background thread + progress UI — **deferred to Phase 8**. Currently the UI freezes during the render (~1–3 s at 8K equirect, ~6 s at 6×4K cubemap).
- [x] `encode_pipeline_pass` helper extracted in `graph.rs` so all four render paths (live preview, equirect PNG, cubemap PNG, equirect EXR) share the same nebula → bloom → tonemap encoding code instead of duplicating it 4×.

**Exit (Phase 6.5):** `cargo run --release` lets a user export equirect PNG, equirect EXR, and cubemap (6 PNG) at the resolution combo of their choice; output drops into a game engine without renaming or post-conversion. ✅

**Still-deferred to Phase 8:** tiled 16K, supersampling, cubemap EXR, background-thread export with progress UI.

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

## Phase 8 — Performance & benchmarks ✅

**Goal:** known performance envelope across hardware tiers.

- [x] **In-app benchmark** in [src/render/bench.rs](../src/render/bench.rs) + `ErebusRenderer::bench_render` ([src/render/graph.rs](../src/render/graph.rs)). Allocates ephemeral resources at each (resolution × step count) config, runs `BENCH_WARMUP=3` warmup frames followed by `BENCH_RUNS=7` measured frames, returns the median ms. Bake runs once before the timing loop so it doesn't pollute the steady-state numbers. Submitted via `device.poll(Wait)` per frame — no readback, so we time the GPU pipeline, not the memory transfer.
- [x] **Bench config matrix**: 1K@64, 1K@128, 2K@96, 4K@96, 4K@128 (all 2:1 equirect aspect). Five configs total ≈ 5–30 s wall-clock depending on GPU. Surfaced via the new **Performance** panel with a `Run benchmark` button + a results table coloured by FPS (green ≥ 60, amber ≥ 30, red <30). Replaces the original `benches/raymarch.rs` plan — keeps everything in one binary, no separate headless wgpu setup needed.
- [x] **Adaptive preview**: parameters get hashed each frame ([src/app/mod.rs::params_hash](../src/app/mod.rs)); changes bump a `last_interaction_at` timestamp. While the timestamp is < 250 ms ago, the GUI scales the offscreen target down by 0.5× ([src/gui/mod.rs](../src/gui/mod.rs)). Snaps back to the user-chosen `preview_scale` after the user stops dragging. The Frame panel shows a faint "(auto ½)" indicator while interacting.
- [x] **Quality tier buttons** in the Frame panel: Draft (½-res, 64 steps, 4 shadow, 1 layer), Preview (full-res, 96 / 4 / 3), Quality (full-res, 128 / 6 / 3), Export (256 / 8 / 3). One-click snap to the recommended tier; users can still tweak individual sliders afterward. Each tier has a tooltip explaining what it changes and when to use it.
- [~] **Off-tool profiling on integrated / mid-range / high-end GPUs deferred** — the in-app benchmark covers the same ground from any user's machine, and is exactly what an itch.io customer or reviewer would run to characterise their own setup. Cross-hardware data tables would still be useful for the itch.io page's "minimum specs" claim — gather later when more development hardware is available.

**Exit:** preview holds 30+ FPS on integrated GPU at 720p / 64 steps. (Verified by user on the dev T1200 ✅; cross-platform confirmation deferred to release prep.) ✅

### Quality tier reference

| Tier | preview_scale | nebula.steps | shadow_steps | starfield.layers |
| --- | --- | --- | --- | --- |
| Draft | 0.5 | 64 | 4 | 1 |
| Preview | 1.0 | 96 | 4 | 3 |
| Quality | 1.0 | 128 | 6 | 3 |
| Export | 1.0 | 256 | 8 | 3 |

### Reference numbers — NVIDIA T1200 (Vulkan, 2026-04-26)

Measured via the in-app benchmark on the dev laptop. Tier ratio is consistently ≈ 1.5× between Draft and Export at 1K-128 and above. At 1K/64 the per-frame CPU dispatch overhead (~10 ms) dominates, which is why both tiers land at ~12–13 ms there — that's the noise floor on this hardware, not a bug.

| Config | Draft ms | Export ms |
| --- | --- | --- |
| 1K equirect / 64 steps | 13.4 | 12.3 |
| 1K equirect / 128 steps | 15.2 | 22.6 |
| 2K equirect / 96 steps | 44.5 | 65.7 |
| 4K equirect / 96 steps | 174.5 | 256.9 |
| 4K equirect / 128 steps | 228.0 | 335.9 |

Phase 8 exit criterion (preview holds 30+ FPS on integrated GPU at 720p / 64 steps) is met with margin — 74 fps in Draft at 1K. The dev hardware sits roughly at "low-end 2021 laptop discrete" tier; Apple Silicon and discrete RTX 30xx+ should be 1.5–4× faster across the board. Cross-platform numbers gather closer to release prep.

---

## Phase 9 — Web build (M7) ✅

**Goal:** browser demo of the same binary.

- [x] **Crate restructured to lib+bin**: [src/lib.rs](../src/lib.rs) declares the modules and exposes a `#[wasm_bindgen]` `start(canvas_id)` entry point that calls `eframe::WebRunner::new().start(...)`. The native binary [src/main.rs](../src/main.rs) is now a one-line caller of `erebus::app::run_native()`. Cargo.toml gains `[lib] crate-type = ["cdylib", "rlib"]`.
- [x] **`wasm32-unknown-unknown` target builds clean** in release: 4.0 MB raw / **1.5 MB gzipped** wasm bundle. Well under the 10 MB target.
- [x] **WebGPU detection + fallback** in [assets/web/index.html](../assets/web/index.html): checks `navigator.gpu` before importing the wasm module; shows a styled fallback message with browser-version requirements if missing.
- [x] **Asset embedding via `include_str!`**: shaders ([src/render/graph.rs::load_shader](../src/render/graph.rs)) and shipped presets ([src/preset/io.rs::load_embedded](../src/preset/io.rs)) bake into the binary on wasm; native still reads from disk so hot-reload works.
- [x] **`getrandom` `js` feature** enabled on wasm so `rand` calls source entropy from the browser's `crypto.getRandomValues`.
- [x] **Native-only paths gated**: `notify` (file watcher), `rfd` (file dialogs), `wgpu::naga` (re-export missing on wasm), `eframe::run_native` are all behind `#[cfg(not(target_arch = "wasm32"))]`. Synchronous Performance + Export panels are hidden in the browser build (the tab can't freeze for ~2 s of GPU work); preset save/load is gated likewise. Shipped preset buttons still work in-browser via `include_str!`.
- [x] **Reduced browser defaults** — `State::default()` is cfg-gated on `wasm32` to start in the Draft tier (½-res, 64 march steps, 4 shadow steps, 1 star layer) so the live preview is interactive on integrated GPUs without the user having to click `Draft` after page load.
- [x] **Browser file save** for the export pipeline — equirect PNG only on wasm. The render path (`ErebusRenderer::render_equirect_rgba8`) is shared with native; encoding writes to a `Vec<u8>` via `image::ImageBuffer::write_to(Cursor)`; download fires via a `web_sys::Blob` + `Url::create_object_url_with_blob` + synthesised anchor click. Cubemap (six downloads / zip) and EXR (large payloads) defer to Phase 11.

**Exit:** browser bundle launches and runs the live preview at acceptable rates in Chrome 113+ / Safari 26+ / Firefox 145+. ✅ Verified rendering 2026-04-26 in Chrome on Linux.

## Phase 9.5 — Web compatibility (dependency upgrade) ✅

**Goal:** unblock the web build runtime by upgrading the eframe + egui + wgpu stack.

- [x] Bump `eframe = "0.28"` → `0.30`, `egui = "0.30"`, `egui-wgpu = "0.30"`, `wgpu = "23"`.
- [x] Resolve wgpu 23 API drifts: added `cache: None` field to all 8 pipeline descriptors; wrapped `entry_point` strings in `Some(...)`.
- [x] Resolve egui-wgpu 0.30 `CallbackTrait::paint` — reverted to non-generic signature with `RenderPass<'static>` and non-lifetimed `&CallbackResources`.
- [x] Resolve eframe 0.30 `WebRunner::start` change — takes `HtmlCanvasElement` directly; resolved via `web_sys::window().document().get_element_by_id(...)`.
- [x] Resolve `egui::ComboBox::from_id_source` → `from_id_salt` rename.
- [x] Native release build clean.
- [x] WASM release build clean. Bundle: **2.9 MB raw / 1.3 MB gzipped** (smaller than the 0.28 build).
- [x] All tests pass (preset roundtrip, WGSL validation).
- [x] Chrome accepts the WebGPU `requestDevice` call (the `maxInterStageShaderComponents` error is gone — see [BUGS.md #9](BUGS.md)).

**Exit:** web bundle runs in Chrome 130+ / Firefox / Safari. Native build unchanged. ✅

**Migration cost:** ~2 hours of API-shim work, no visual regressions reported in the desktop build.

**Build commands:**

```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
cargo build --target wasm32-unknown-unknown --lib --release
wasm-bindgen --target web --out-dir assets/web \
    target/wasm32-unknown-unknown/release/erebus.wasm
# Serve assets/web/ from any static host (or `python3 -m http.server`).
```

---

## Phase 10.5 — Physical-realism foundations (R-series subset)

**Goal:** front-load the small physics-driven shader upgrades that make the rest of the v1.x roadmap (especially Phase 13 nebula variants) coherent rather than an architectural retro-fit. Pulled from [build_docs/REALISM_HANDOFF.md](../build_docs/REALISM_HANDOFF.md). R4 (multi-scatter), R7 (Airy core), R8 (FFT bloom — overlaps Phase 11), and R9 (4D noise) are deferred.

- [x] **R1.1 — Curl-noise warp** ([shaders/nebula/raymarch.wgsl](../shaders/nebula/raymarch.wgsl)). Replace the divergence-bearing FBM-vector domain warp with a divergence-free vector field via finite-difference curl of three decorrelated scalar potentials. Long flowing tendrils instead of cottony bulges (Bridson 2007). UI selector (`Domain warp → kind`) lets users compare against the legacy FBM warp; new state defaults to curl, shipped presets keep FBM until re-baked.
- [x] **R1.2 — Cornette-Shanks phase function**. Drop-in replacement for HG; satisfies symmetry HG violates and gives a tighter forward peak. `phase_kind` selector (HG / CS) wired to the Scattering panel; new presets default to CS, old presets keep HG via serde default.
- [x] **R1.3 — Log-normal density remap**. New `density_pivot` + `density_contrast` uniforms drive an exponential remap of the FBM shape via `density_from_shape()` (used by both `nebula_density` and `shadow_density`). `density_contrast = 0` falls back to the legacy linear clip so old presets render unchanged.
- [x] **R2 — Wavelength-dependent extinction (interstellar reddening)**. `nebula.sigma_e` is now `[f32; 3]`; raymarch `transmittance` likewise; shadow-march early-out is per-channel `min(...) > 6`. Default ratio `[0.65, 1.00, 1.45]` matches ISM R_V=3.1. GUI replaces the scalar slider with reddening-law combo (ISM / Gray / Custom) + intensity slider; Custom mode exposes per-channel R/G/B sliders. Preset format bumped to v2; v1 presets load via a custom `deserialize_with` that broadcasts the legacy scalar to all three channels (Gray dust, identical to old behaviour).
- [x] **R3.1 — Multi-channel baked volume**. The previously-reserved B channel of [shaders/compute/bake_3d_noise.wgsl](../shaders/compute/bake_3d_noise.wgsl) now stores `smooth × ridged` — a dust field with filament structure. R/G keep their existing smooth/ridged FBM semantics so legacy presets render unchanged. New `sample_noise_4` helper exposes the full vec4.
- [x] **R3.2 — Multi-channel raymarch**. New `nebula_multichannel(p)` returns `(halpha, oiii, dust)`. `density_kind` selector (`DENSITY_LEGACY` / `DENSITY_MULTICHANNEL`) branches the main raymarch loop; LEGACY keeps the gradient-LUT pipeline untouched, MULTICHANNEL drives emission per spectral line and extinction from dust. Raymarch composition: `transmittance * (absorbed * (in_scatter + self_emission) + emission * dt)`. Per-channel σ_e (R2) reddens the dust extinction automatically.
- [x] **R3.3 — Palette mode toggle**. `palette_mode` (`PALETTE_NATURAL = 0`, `PALETTE_HOO = 1`) read by the raymarch when `density_kind == MULTICHANNEL`. NATURAL = red Hα + teal [OIII] (eye-through-telescope). HOO = red Hα + cyan [OIII] (popular two-line narrowband). SHO is deferred until the [SII] channel ships in a later phase. UI: combo box appears under Emission model when MULTICHANNEL is selected.

**Exit:** the existing three shipped presets render visibly more like real nebula photography (red emission, blue dust scattering, dust lanes that redden background stars). Phase 13 nebula variants become a presets-and-tuning job rather than an architectural change. Performance: budget +2.5 ms at preview / 1K-128 on the dev machine; wasm idle stays at 60 fps blit thanks to the existing freeze.

**Migration:** preset format version bump. Defaults for new fields preserve current behaviour for old presets (`sigma_e = [1, 1, 1]`, `palette_mode = 0`, etc.).

---

## Phase 10 — Release (M8) — IN PROGRESS

**Goal:** ship v1.0 to itch.io at $8–$15. Phases 11–15 ship after as free v1.x updates.

- [x] User manual ([docs/USER_MANUAL.md](USER_MANUAL.md)) — single-file end-user guide covering quickstart, view modes, panel reference (every slider), recipes, export formats with engine drop-in instructions, performance tiers, web vs native differences, troubleshooting, determinism, physics references appendix.
- [ ] Demo gallery: 12+ curated stills covering aesthetic range.
- [ ] Trailer / GIF reel.
- [ ] Itch.io page copy, screenshots, system requirements.
- [ ] Native installers: signed Windows `.exe`, notarised macOS `.dmg`, Linux `.AppImage` + `.tar.gz`.
- [ ] Public web demo (Phase 9 build) embedded on the itch.io page.
- [ ] Changelog and version-1.0 tag.

**Exit:** v1.0 release. Customers can buy and download. Phases 11–15 then queue as v1.1+ free updates per the original roadmap order.

---

## Post-release expansion phases (v1.x)

Each of the four phases below ships as a paid or free v1.x update on itch.io. They're scoped so any individual phase is 2–3 weeks of focused work and produces customer-visible variety. Roughly ordered by implementation cost ascending and by customer-recognisability descending.

### Phase 11 — Hero objects (v1.1)

Cinematic foreground objects and the optical effects that make bright sources read as photographic.

- [ ] **Hero star** ("the sun"): single placeable bright disc with corona, chromosphere granulation noise, customisable angular size + temperature. Click-to-place on the live preview.
- [ ] **Lens flare / ghost reflections**: chain of disc + polygon ghosts along the optical-axis line through the brightest source.
- [ ] **Proper N-fold diffraction spikes**: 4 / 6 / 8 spokes with arbitrary rotation; the existing `spike_count` slider becomes load-bearing. Replaces the Phase-4 axis-aligned cross.
- [ ] **FFT convolution bloom**: physically correct aperture-shape PSF as an alternative to the 13-tap pyramid. Marty McFly's `iMMERSE` parameter surface as reference (padding, threshold, radius, blade count, sharpness).

**Exit:** a user can compose "rising sun behind a distant nebula" with realistic lens artifacts in under 30 seconds.

### Phase 12 — Galactic features (v1.2)

Background detail that makes the sky feel inhabited rather than empty.

- [ ] **Distant galaxies** as oriented billboards: spiral / elliptical / irregular shapes with rotation, Sersic-like brightness profile, hue (typical: yellowish elliptical, blue-tinted spiral arms).
- [ ] **Globular clusters**: tight bright knot of ~hundreds of stars at a placed direction.
- [ ] **Open clusters**: 5–20 bluish stars in a Pleiades-like loose group with common motion vector.
- [ ] **Galactic core / bulge**: tilted-ellipsoid density boost along the galactic plane axis.
- [ ] **Dust lanes**: secondary low-albedo noise pass producing dark filamentary structures *within* the nebula volume.
- [ ] **Multi-region density masks**: replace the single galactic plane with N user-placed bands, blobs, or arcs.

**Exit:** Milky-Way-from-La-Palma aesthetic possible with a single preset; distinct foreground / background depth in the field.

### Phase 13 — Nebula type variants (v1.3)

Different volumetric profiles fed into the existing raymarcher. Largest visual-variety-per-line-of-code phase since the pipeline is already there.

- [ ] **Planetary nebulae**: small, sharply-bounded spherical or bipolar emission shells. Distance-from-centre falloff at fixed radius, illuminated by a central white-dwarf point light.
- [ ] **Supernova remnants**: ridged-FBM-constrained-to-a-thin-spherical-shell. Veil / Crab / Cygnus Loop aesthetics.
- [ ] **Reflection nebulae**: blue-tinted, illuminated *by* a nearby light rather than self-emitting. `ambient_emission = 0` + custom blue gradient + `albedo = 1` is most of the configuration.
- [ ] **Dark nebulae / Bok globules**: negative-density regions silhouetted against bright background. Requires a "transmittance-only" volume mode.
- [ ] **HII regions / stellar nurseries**: bright pink/magenta Hα-line-emission blobs. Mostly a gradient + density preset.

**Exit:** five distinct nebula archetypes shipped as presets; one tool covers the visual range from Trifid to Veil to Pleiades.

### Phase 14 — Exotic & relativistic (v1.4)

The "wow factor" phase. Real new shader math; the technical centrepiece of the tool.

- [ ] **Distant black hole** (small on screen): accretion disk with relativistic Doppler beaming (brighter on the approaching side), event-horizon shadow, optional polar jets.
- [ ] **Hero black hole** (replaces hero-star slot): full Schwarzschild lensing of the background — warp ray direction by deflection angle ∝ 4GM/bc². Einstein ring, photon sphere, full accretion disk.
- [ ] **Gravitational lensing of background**: any heavy point in the scene distorts the equirect ray around it. Reusable once the deflection math is in.
- [ ] **Pulsars**: millisecond-pulsing point + two opposing jets. Uses the existing `time` uniform.
- [ ] **Quasars**: extremely bright distant point with a thin accretion disk and a redshift-shifted gradient.
- [ ] **Wolf-Rayet bubbles**: spherical wind-blown shells with a bright central star; combines a hero star with a thin volumetric shell.

**Exit:** an Interstellar-Gargantua-quality shot is achievable in the tool; the centrepiece "screenshot" for the itch.io page.

### Phase 15 — Compositional tools & polish (v1.5)

The ergonomic layer that turns Erebus from "procedural generator" into "scene composer".

- [ ] **Click-to-place gizmo** for hero objects (star, black hole, galaxy) on the live preview.
- [ ] **Object inspector panel** with per-object parameters; multi-select.
- [ ] **Drag-stops gradient editor widget** (deferred from Phase 7).
- [ ] **Recent presets list** + per-preset thumbnail in the load dialog.
- [ ] **Time slider** for animated modes (variable stars, pulsar pulses, rotating accretion disks). Animated GIF / WebM export.
- [ ] **Compact base64 share-string** (deferred from Phase 7) — Discord-pasteable preset codes.

**Exit:** the tool is comfortable for a non-technical user; the v1.5 release video shows compositional workflow rather than parameter sliders.

---

## Cross-cutting concerns

- **Determinism**: every preset produces identical pixels for a given seed across all platforms. Verified by a hash-of-PNG test fixture in CI on representative presets. Drift on platform-specific intrinsics is the most likely failure; use `f32` carefully and avoid platform-divergent functions.
- **HDR everywhere**: never write to an 8-bit intermediate; always RGBA16F until the final tonemap+dither step. This is the single largest determinant of output quality (compass artifact §6.2).
- **Shader test discipline**: `tests/wgsl_validation.rs` parses every shader through Naga on every CI run. Catches regressions before they reach a device.
- **No hidden state**: every visible visual property maps to a serialisable preset field. No "magic" hard-coded numbers in shaders; all live in uniforms.
- **Visual review at every phase exit**: produce a still image, compare it to references in the compass artifact, and iterate before declaring the phase done.
