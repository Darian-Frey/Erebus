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

## Phase 2 — Nebula MVP (M2)

**Goal:** a recognisable volumetric nebula on screen, parameter-driven.

- [ ] Compute pass `bake_3d_noise` producing a 128³ RGBA16F Perlin–Worley volume.
- [ ] Compute pass `bake_gradient` producing a 256-texel 1D gradient LUT from a hard-coded ramp.
- [ ] `shaders/nebula/raymarch.wgsl`: equirect ray direction, fixed 64 steps, FBM density, gradient lookup, Beer–Lambert accumulation.
- [ ] `shaders/nebula/density.wgsl`: domain-warped FBM + ridged FBM blend.
- [ ] `shaders/common/noise.wgsl`: gradient noise, Worley, FBM, ridged FBM, domain warp, Clifford-torus 4D wrap.
- [ ] `shaders/common/sampling.wgsl`: equirect mapping, Henyey–Greenstein.
- [ ] Blue-noise dither at march start.

**Exit:** screen shows a nebula that visibly responds to seed, density, and gradient changes; no obvious step banding at 96+ steps.

**Risks:** noise tuning is the hardest part of the project. Budget 2–3× expected time for shader iteration; that is where the visual ceiling lives.

---

## Phase 3 — Lighting (M2 cont.)

**Goal:** depth and drama via in-volume light.

- [ ] 1–4 user-placed point lights inside the volume (uniform array).
- [ ] `shaders/nebula/lighting.wgsl`: 8-step shadow march per main sample, secondary Beer–Lambert.
- [ ] HG anisotropy slider exposed (`g ∈ [-0.9, 0.9]`).
- [ ] Optional emissive density falloff for "core glow" effect.

**Exit:** turning lights on noticeably reshapes the nebula's volume; dense regions cast clear shadows.

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
