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

## #8 — Starfield Kelvin sliders had no perceptible effect — 2026-04-26 — fixed

**Symptom.** Setting `T_min` and `T_max` to extreme values (both very low expecting red, both very high expecting blue) produced visually identical exports — stars stayed pale-blue/white in both cases. The user noticed and flagged it.

**Root cause.** The starfield shader keyed both `magnitude` and `temperature` off the same hash component `h.y`:

```wgsl
let mag = pow(h.y, starfield.imf_exponent);
let temp = mix(starfield.temperature_min, starfield.temperature_max, h.y);
```

Coupled in this way, bright stars (high `h.y`) always pinned to `T_max` and dim stars (low `h.y`) always pinned to `T_min`. Because the IMF exponent (default 5) biases the field toward dim sub-pixel stars, the only *visible* stars were the ~5 % brightest — which were all locked to `T_max`. Result: `T_min` was effectively ignored, and the field was monochromatic at any single moment.

**Fix.** Sample a second hash with a fixed integer offset for the temperature roll, in [shaders/nebula/raymarch.wgsl](../shaders/nebula/raymarch.wgsl):

```wgsl
let temp_h = hash3i(cell + vec3<i32>(13, 17, 23), frame.seed ^ layer_seed);
let temp = mix(starfield.temperature_min, starfield.temperature_max, temp_h.x);
```

Cost: one extra PCG3D per star (negligible). Visual benefit: red giants and blue dwarfs at all magnitudes; both Kelvin sliders now drive a clearly visible response.

**Lesson.** Whenever two random outcomes are sampled from the same hash channel, ask whether the correlation is intentional (e.g. mass-luminosity for *physically* realistic stars) or accidental (e.g. accidentally fused-domain-hashing). Here a *more* physically realistic version (mass-temperature coupling) was less *visually* useful because the IMF skews the visible population to one end.

---

## #7 — RON serializes Rust fixed-size arrays as tuples, not lists — 2026-04-26 — fixed

**Symptom.** All three shipped preset RONs failed to load with `Expected opening '('` at the position of `lights: [...]`. The unit tests `shipped_presets_load` and `shipped_presets_round_trip` exposed this immediately.

**Root cause.** I'd written the `[PointLight; 4]` array as a JSON-style list `[a, b, c, d]`. RON serializes Rust *fixed-size arrays* as tuples — `(a, b, c, d)` — and only `Vec<T>` as lists. The `gradient: Vec<GradientStop>` field is correctly a list; `lights: [PointLight; 4]` must be a tuple.

**Fix.** Changed the `lights:` field's brackets in [synthwave.ron](../assets/presets/synthwave.ron), [cyberpunk.ron](../assets/presets/cyberpunk.ron), [retro_scifi.ron](../assets/presets/retro_scifi.ron) from `[...]` to `(...)`.

**Lesson.** When hand-writing RON for any Rust struct, sketch a tiny `ron::ser::to_string_pretty(&default_value, ...)` first and copy the format. Don't assume JSON-list syntax is interchangeable with RON tuple syntax — the distinction matters when the Rust type is a fixed-size array vs a `Vec`.

---

## #6 — WGSL rejects unary `+` in expressions — 2026-04-26 — fixed

**Symptom.** `cargo test --test wgsl_validation` failed with `error: expected expression, found '+'` at the AgX polynomial in [shaders/composite.wgsl](../shaders/composite.wgsl):

```wgsl
return + 15.5 * x4 * x2 - 40.14 * x4 * x + ...;
```

**Root cause.** WGSL only supports unary `-`, not unary `+`. The leading `+` was a copy-paste from the GLSL/HLSL Three.js AgX reference, which both accept it.

**Fix.** Dropped the leading `+` and added an explicit `vec3<f32>(0.00232)` in the trailing constant term so naga's vector arithmetic could resolve the type. Same lesson for any future shader port from GLSL: scan for `return + …` and `return -<scalar>` patterns.

**Lesson.** When porting math from GLSL/HLSL to WGSL, run the file through `cargo test --test wgsl_validation` immediately rather than relying on the runtime device validation — naga's error messages point straight at the offending line.

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
