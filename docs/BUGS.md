# Bug Log

Running ledger of bugs encountered during development, their root cause, and the fix. Append new entries at the top. Closed entries stay in the file — the value is the historical record, not the open queue.

## Format

Each entry uses this shape:

```markdown
## #N — short title — YYYY-MM-DD — [open|fixed]

**Symptom.** What you observed.
**Root cause.** Why it happened.
**Fix.** What changed (link the commit/PR or the file).
**Lesson.** What to remember next time. Skip if obvious.
```

Number entries monotonically. Don't reuse numbers when bugs are deleted — leave a tombstone (`#N — withdrawn`).

---

## #5 — Per-frame IGN rotation reads as flicker without TAA — 2026-04-26 — fixed

**Symptom.** With Phase 3.5 baked-noise speedup running at ~40 fps, the user reported the nebula was visibly flickering frame-to-frame on a static scene.

**Root cause.** Phase 2's `ign(pixel, frame_index)` rotated the interleaved-gradient noise tile per frame so that residual sampling artifacts would smear into temporal noise. The original assumption was that downstream bloom + tonemap (Phase 5) would smooth the rotation into perceptual mush. Without bloom present, the rotation IS the flicker — every frame's first-sample jitter offset shifts by ~1 step length, so the band of pixels that catches a thin density spike changes visibly each frame. At 40 fps with a static scene the temporal aliasing is unmistakable.

**Fix.** Made `ign()` purely spatial in [shaders/nebula/raymarch.wgsl](../shaders/nebula/raymarch.wgsl) — dropped the `frame_index` argument. The temporal jitter will be re-introduced in Phase 5 alongside proper TAA-style accumulation where bloom hides the residual.

**Lesson.** Don't ship temporal aliasing tricks without the temporal smoothing they were designed against. "It'll get smeared by bloom later" is a valid argument *only after bloom exists* — until then, static jitter is correct.

---

## #4 — `RenderPipelineDescriptor::cache` not in wgpu 0.20 — 2026-04-25 — fixed

**Symptom.** `cargo check` errored: `struct wgpu::RenderPipelineDescriptor<'_> has no field named cache` at both pipeline-creation sites in [src/render/graph.rs](../src/render/graph.rs).

**Root cause.** The `cache: Option<&PipelineCache>` field was added in wgpu 22; we are pinned to wgpu 0.20.

**Fix.** Removed the `cache: None,` lines. Will revisit when we bump wgpu past 22.

**Lesson.** Check the version we're actually pinned to before copying boilerplate from current wgpu docs.

---

## #3 — `Receiver<notify::Event>` is `Send` but not `Sync` — 2026-04-25 — fixed

**Symptom.** `cargo check` errored: `std::sync::mpsc::Receiver<notify::Event> cannot be shared between threads safely` for the renderer stored in `egui_wgpu::CallbackResources`.

**Root cause.** `egui_wgpu::CallbackResources` is `type_map::concurrent::TypeMap`, which requires `Any + Send + Sync`. `mpsc::Receiver` only satisfies `Send`.

**Fix.** Wrapped the receiver in `std::sync::Mutex` inside [src/render/hot_reload.rs](../src/render/hot_reload.rs). `poll()` calls `self.rx.get_mut()` since we already have `&mut self` and don't need a runtime lock.

**Lesson.** Anything that lives inside `CallbackResources` must be `Send + Sync`. `Mutex` is the standard upgrade for `Send`-only channel ends.

---

## #2 — `paint()` lifetime mismatch with `egui_wgpu::CallbackTrait` — 2026-04-25 — fixed

**Symptom.** `cargo check` errored: `method not compatible with trait: lifetime mismatch`. The trait expected `fn paint<'a>(&'a self, .., &mut RenderPass<'a>, &'a CallbackResources)`; our impl had `&mut RenderPass<'static>` and no `'a`.

**Root cause.** Misread the egui-wgpu 0.28 trait signature. The pass lifetime is tied to `&self` and `&CallbackResources`, not `'static` — that's how the trait threads borrows of resources into the pass safely.

**Fix.** Restored the `<'a>` generic on both `NebulaCallback::paint` ([src/gui/mod.rs](../src/gui/mod.rs)) and `ErebusRenderer::composite` ([src/render/graph.rs](../src/render/graph.rs)).

**Lesson.** When a trait uses a generic lifetime, the impl must repeat it verbatim. Don't substitute `'static` "to simplify" — variance flips and the compiler rejects it.

---

## #1 — winit fails to compile under eframe with `default-features = false` — 2026-04-25 — fixed

**Symptom.** First `cargo check` produced 65 errors inside `winit-0.29.15`, starting with `The platform you're compiling for is not supported by winit` and a cascade of `unresolved import self::platform` / `type annotations needed`.

**Root cause.** Disabling eframe's default features stripped the `wayland` and `x11` features it forwards to winit. winit then has no platform backend selected on Linux and refuses to compile.

**Fix.** Added `"wayland", "x11"` to the eframe feature list in [Cargo.toml](../Cargo.toml).

**Lesson.** When trimming default features off a windowing crate, always re-enable the platform backends explicitly. eframe's relevant features are `wayland`, `x11`, `glow`, `wgpu`, `accesskit`, `persistence`.
