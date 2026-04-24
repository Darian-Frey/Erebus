# Architecture

A high-level map of how data flows through Erebus, from a slider drag to a pixel on screen and a file on disk.

## Module ownership

```
app/         owns: top-level State, eframe lifecycle, frame loop
  └─ State  owns: preset values, viewport, dirty flags
gui/         consumes: &mut State; produces: UI events
render/      consumes: State (read-only); produces: HDR frame, swapchain blit
  ├─ context        owns: wgpu Device/Queue, surface
  ├─ resources      owns: textures, buffers, samplers, LUTs
  ├─ uniforms       owns: packed POD blocks shared with WGSL
  ├─ passes/        owns: pipelines + per-pass bind groups
  ├─ graph          owns: ordered execution of passes
  └─ hot_reload     owns: WGSL file watcher
export/      consumes: render graph + State; produces: PNG/EXR files
preset/      consumes/produces: serde structs
noise/       consumes: seed; produces: CPU-side LUT bytes
```

State flows in one direction: `gui → State → render`. Render never mutates state. Export reuses the render graph by binding a different output target.

## Frame graph (per preview frame)

```
┌─────────────────────────────────────────────────────────────────┐
│ Pass 0 (precompute, dirty-driven)                                │
│   bake_3d_noise.wgsl     → noise_volume      (128³ RGBA16F)      │
│   bake_blackbody.wgsl    → blackbody_lut     (1024×1 RGBA16F)    │
│   bake_gradient.wgsl     → gradient_lut      (256×1 RGBA16F)     │
└─────────────────────────────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│ Pass 1: nebula raymarch  → hdr_target_a      (RGBA16F)           │
│   reads: noise_volume, gradient_lut, blue_noise                  │
└─────────────────────────────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│ Pass 2: starfield (additive into hdr_target_a)                   │
│   reads: blackbody_lut, hdr_target_a (for nebula-density mask)   │
│   PSF billboards for stars above brightness threshold            │
└─────────────────────────────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│ Pass 3: bloom pyramid                                            │
│   downsample(13-tap) → mip0..mip7  → upsample(tent) ──┐          │
└──────────────────────────────────────────────────────┬┘          │
                             ▼                         │ composite │
┌─────────────────────────────────────────────────────────────────┐
│ Pass 4: tonemap + grade + dither → hdr_target_b → swapchain      │
│   exposure → AgX/ACES/Tony/Reinhard → grade → triangular dither  │
└─────────────────────────────────────────────────────────────────┘
```

Pass 0 only re-runs when its inputs are dirty (seed change, gradient edit, etc.). Passes 1–4 run every frame the preview is live; if all sliders are at rest the app can drop to 5 FPS to save battery.

## Export path

The export job replaces the swapchain target with a `MAP_READ` buffer behind a `RENDER_ATTACHMENT | COPY_SRC` texture, optionally tiles the viewport into 4K chunks with adjusted ray-basis uniforms, and stitches the tiles CPU-side. Supersampling renders each tile at 2× / 4× resolution and box-filters down before stitching. EXR output skips the tonemap pass and writes the linear HDR target directly.

## Determinism

A single `u64` master seed in the preset is fanned out to per-system substreams (nebula, starfield, dust) by `noise::seed`. All randomness is pure functions of substream + cell coordinates, no per-frame mutation. A given preset on a given hardware class produces byte-identical PNG output; `tests/preset_roundtrip.rs` enforces this.

## Hot reload

`render::hot_reload` watches the `shaders/` tree via `notify`. On change it re-reads the file, validates with Naga, replaces the cached `wgpu::ShaderModule`, and rebinds the affected pipeline. Errors surface in the egui frame as a non-blocking toast — the previous-known-good shader keeps rendering until the new one validates.
