# Erebus — User Manual

A procedural volumetric nebula and starfield generator. Compose a scene with sliders, render high-quality equirect or cubemap skyboxes, drop them into Unity / Unreal / Bevy / Godot / Blender.

This manual covers the desktop build (the full product) and notes where the in-browser preview differs.

---

## 1. What Erebus is

Erebus renders a 360° procedural sky as a single equirect image or as a six-face cubemap. The sky contains:

- a **volumetric nebula** (raymarched dust + gas with Beer-Lambert extinction, Cornette-Shanks scattering, multi-channel emission lines).
- in-volume **point lights** illuminating the gas from inside.
- a procedural **starfield** with realistic blackbody colours and diffraction spikes.
- physically grounded **interstellar reddening** (dust attenuates blue more than red, exactly as the real ISM does).

The output is a normal image file you import into your game engine or 3D tool as a skybox / environment map. There is no animated playback at runtime — Erebus is an offline asset generator.

**Two builds:**

- **Desktop (the product).** Full quality, exports up to 8K equirect / 4K cubemap, EXR support, hot-reload of shaders, file dialogs, performance benchmark.
- **Web preview** (`localhost:8000` or the itch.io demo page). Low-fidelity in-browser demo capped at 1K PNG export. Limited by browser WebGPU performance on integrated GPUs. A "Render hero shot" button gives you one full-quality preview frame per click.

---

## 2. Quickstart

Three minutes from launch to saved skybox.

1. **Launch Erebus.**
2. In the **Preset** panel (top-left), click one of the shipped presets: **Synthwave**, **Cyberpunk**, or **Retro Sci-Fi**. The scene loads and renders.
3. In the **Frame** panel, click the **Quality** button (or **Export** for max quality). This bumps march steps and shadow march to slow-but-clean values.
4. Open the **Export** panel (near the bottom of the side panel).
5. Pick **kind = Equirect**, **format = PNG**, **width = 4K**.
6. Click **Export PNG…**. A file dialog appears. Choose where to save it.
7. The render takes a few seconds. The status line shows `saved /path/to/erebus_equirect_4096.png` when done.

Drop that PNG into your engine's skybox slot. Done.

---

## 3. The view modes

Above the central canvas, the **Frame → view** dropdown switches the live preview between two projections. Both render from the same scene data; only the on-screen presentation changes. Export is unaffected.

### 3.1 Flat (equirect)

The skybox shown as a 2:1 unwrap. This is what the exported equirect PNG actually looks like. It's correct for what Erebus outputs but the top and bottom of the image are heavily distorted (every pixel along the top scanline is the +Y pole). Useful for confirming framing and for "I want to see the whole sky at once" inspection.

### 3.2 Skybox

A virtual perspective camera at the centre of the scene, looking out into the sky. Click-drag inside the canvas to rotate; scroll to zoom. The view is what your in-engine player would see if they aimed their camera in that direction. **This is the right preview for design work** — it shows the actual perceptual experience of your skybox.

**Skybox controls (active when view = Skybox):**

| Action               | Result                                |
|----------------------|---------------------------------------|
| Click and drag       | Rotate yaw / pitch (0.25°/pixel)      |
| Mouse wheel          | Adjust FOV (clamped 30°–110°)         |
| `Space`              | Toggle Flat ↔ Skybox                  |
| `R`                  | Reset camera (yaw 0, pitch 0, fov 70°)|
| `←` / `→`            | Nudge yaw by 5°                       |
| `↑` / `↓`            | Nudge pitch by 5°                     |
| `[` / `]`            | FOV ±5°                               |
| **Reset view** button | Same as `R`                          |
| **fov** slider       | Direct FOV control                    |

Pitch is clamped to ±89° (no upside-down). Yaw wraps. Releasing a fast drag leaves a brief inertia decay so a quick flick spins down naturally.

**Performance note.** Skybox dragging is essentially free — the offscreen render is cached and only the cheap composite pass re-runs each frame. You can crank the desktop's preview to **Quality** tier and still drag at 60 fps.

---

## 4. Composing a scene — the panels

The side panel runs top-to-bottom: Preset → Frame → PostFX → Nebula → Lighting → Starfield → Performance → Export. Each section starts collapsed except the most-used ones. Click any header to open it.

Slider tooltips inside the app give the same advice as below; this section is the reference.

### 4.1 Preset

- **name** — Free-text name written into saved RON files.
- **Save…** / **Load…** — Native file dialogs. `Save` writes the current state (all uniforms + gradient stops) to a `.ron` file. `Load` reads one back. Web build only loads the shipped presets (no file dialog in browser).
- **Synthwave / Cyberpunk / Retro Sci-Fi** — Three shipped presets. Click to load.

The orbit-camera position (yaw/pitch/fov), the view mode, the preview-quality tier, and the seed are *not* saved into preset files — those are viewer settings, not scene parameters. Presets are portable across machines.

### 4.2 Frame

- **quality** buttons (`Draft`, `Preview`, `Quality`, `Export`) — Snap the live preview to a known-good performance/quality tier.
  - `Draft` — half-res, 64 march steps, 4 shadow steps. Fast slider iteration on integrated GPUs.
  - `Preview` — full-res, 96 march steps, 4 shadow steps. The default.
  - `Quality` — full-res, 128 march steps, 6 shadow steps. Hero-shot quality at interactive rates on a discrete GPU.
  - `Export` — 256 march steps, 8 shadow steps. Use this before exporting; not playable in real time.
- **view** — Flat / Skybox (see §3).
- **Reset view** — Skybox-only. Restores yaw 0, pitch 0, fov 70°.
- **fov** slider — Skybox-only. 30° to 110°.
- **preview scale** — 0.25 to 1.0 multiplier on the offscreen render's long axis. Lower = faster preview, blurrier. The default 0.5 is a reasonable balance; drop to 0.25 on a slow GPU to keep slider drags responsive.
- **seed** — Integer that varies the entire procedural composition. Click **shuffle** for a new random number; type one in to revisit a previous result. Same seed + same uniforms = bit-identical render across machines.
- **Render hero shot** *(web only)* — Renders one frame at Quality settings (128 march steps, full bloom, full star layers, no preview-scale or pixel cap). The result is then frozen on the canvas — subsequent frames cost almost nothing — until you move a slider or click again. Use this to see what the desktop build would render, without paying the cost every frame.

### 4.3 PostFX

#### Tonemap

- **curve** — `AgX` (default; matches Blender 4.x — preserves saturation, no white-clip), `ACES Fitted` (industry baseline; warmer, more contrast), `Reinhard` (reference comparison only — clips fast, desaturates highlights).
- **exposure (stops)** — Linear EV applied before tonemap. ±1 doubles or halves scene radiance. Most useful range −2 to +2.

#### Bloom

- **intensity** — Multiplier on the bloom contribution. 0 disables bloom entirely (and skips the 9-pass pyramid on the GPU). 0.6 is a typical value; ~1.2 for dreamy glow.
- **threshold** — Luminance threshold for the bright-pass on mip 0. Below threshold = no bloom contribution. Default 1.0; lower for more glow, raise to limit glow to only the brightest cores.
- **radius** — Tent-filter radius applied during the upsample chain. ~1 = sharp, ~3 = soft.

#### Grade

- **saturation** — 1.0 neutral. 0 = greyscale, 2 = punchy.
- **contrast** — 1.0 neutral. ~1.1 = "punchy", 0.7 = washed-out.

#### Other

- **deband amount** — Scales the triangular-PDF dither applied right before the swapchain. 1.0 default; 0 disables. Without this, smooth dark gradients show 8-bit banding.

### 4.4 Nebula

The volumetric raymarcher. Several subsections.

#### Shape

- **density scale** — Spatial frequency multiplier. ~1.0 = large structure, ~3 = fine detail. Don't confuse with march density — this scales the noise input.
- **octaves (density)** — FBM octave count for the density volume. 6 default; 8 maximum (gain ≈ 0.5 means octave 9 contributes <0.2%).
- **lacunarity** — Frequency multiplier per octave. 2.02 default (Duke's anti-grid trick — pure 2.0 produces axis-aligned beating).
- **gain** — Amplitude decay per octave. 0.5 default.
- **ridged blend** — 0 = wispy gas (smooth FBM), 1 = filaments / lightning (ridged FBM), 0.5 = trifid-style mix.

#### Domain warp

- **kind** — `FBM (legacy)` or `Curl (incompressible)`. Curl warp uses a divergence-free vector field (Bridson 2007); produces flowing tendrils and shearing structure where FBM produces cottony bulges. Curl costs ~3× the warp samples but the offscreen render is cached, so only changed frames pay the cost. New scenes default to Curl; shipped presets default to FBM to preserve their original look.
- **warp strength** — Amplitude of the displacement. 0 = flat clouds, 1.5 = trifid tendrils, 4+ = chaos.
- **octaves (warp)** — Octave count for the warp itself. 3 default.

#### March

- **steps** — Raymarch sample count per pixel. Linear cost. 64 = preview, 96 = default, 128 = quality, 256 = export.
- **march length** — World-space length the ray travels. 1.0 default; raise to push the volume further away from the camera.
- **transmittance cutoff** — Early-out threshold. When residual transmittance drops below this on every channel, the ray stops marching. 0.01 saves 30–50% in dense regions.
- **step density bias** — Adaptive step modulator. `dt = base × max(0.25, bias − density)`. Higher = denser regions take smaller steps; halves visible banding for free. 1.5 default.

#### Scattering

- **reddening** — `ISM (R_V=3.1)` (default; physically calibrated interstellar dust — blue ~2× more extinguished than red), `Gray` (wavelength-flat; back-compat with v1 presets), `Custom` (per-channel R/G/B sliders).
- **intensity** *(non-Custom)* — Overall extinction. ~0.3 = wispy haze, 2 = default, ~6 = bright Trifid-style core.
- **σₑ R / G / B** *(Custom)* — Per-channel Beer-Lambert extinction. Custom is the right mode for stylised reddening / cyan dust.
- **albedo** — σ_s / σ_e — fraction of extinguished light that re-scatters. 0.6 default; lower for darker dust lanes.
- **phase** — `HG (legacy)` (Henyey-Greenstein, qualitatively wrong silver lining) or `Cornette-Shanks` (default; same cost, sharper forward peak — the right physics for small particles).
- **anisotropy (g)** — Phase-function eccentricity. 0 = isotropic clouds, 0.6 = forward-scatter dust (default), −0.3 = back-scatter rim.
- **density curve (γ)** — `pow(d, γ)` before the gradient LUT lookup (Legacy emission only). 0.5 (sqrt) lifts wispy tails; 1.0 = linear; 2.0 hides the tails.

#### Density distribution

- **contrast** — Log-normal density remap. 0 = legacy linear clip (back-compat); 1 = subtle bunching; 4 = dense cores ~1.5 EV brighter than thin tendrils (matches observed cold-ISM statistics); 8 = ten decades of dynamic range. Bump nebula march steps when raising past 4.
- **pivot** — Centre point of the remap on the FBM histogram (mean ≈ 0.5). Lower = more void, sharper cores. Only effective when contrast > 0.

#### Emission model

- **density kind** — `Legacy (gradient LUT)` (Phase-5 behaviour; `mix(smooth, ridged) → density` drives a 1D gradient LUT for colour) or `Multichannel (lines)` (separate Hα + [OIII] emission lines + dust extinction; ignores the gradient LUT — colour comes from the line model).
- **palette** *(Multichannel only)* — `NATURAL` (red Hα + teal [OIII] — eye-through-large-telescope) or `HOO` (Hα → red, [OIII] → cyan rim — popular two-line narrowband look).
- **Hα strength** — Multiplier on the Hα-line emission. Hα at 656 nm dominates real emission nebulae; this is the warm pink/red brightness.
- **[OIII] strength** — Multiplier on [OIII]-line emission. [OIII] at 500.7 nm marks the hot inner ionised zones. Keep dim relative to Hα or it overpowers.
- **[OIII] sharpness** — Power applied to the [OIII] field. 1 = same shape as Hα, 3 = sharp inner core (default), 8 = pinpoint hot spots.
- **dust strength** — Multiplier on the dust-extinction field. Dust drives Beer-Lambert extinction (per-channel, paired with the reddening law) so stronger dust both darkens AND reddens.

### 4.5 Lighting

- **active lights** — 0 to 4 in-volume point lights.
- **shadow steps** — Per-light shadow march. 4 = lower bound, 6+ for export quality. Cost is `N_lights × shadow_steps` per main step.
- **ambient emission** — Isotropic self-glow floor. 0 = pure lit-only (Horsehead silhouette); 1+ = Phase-2 self-glow look. Re-uses the gradient LUT colour in Legacy mode.
- **Light 1..4** — Each light has:
  - **position** — World-space x/y/z. Drag with the mouse. Inside the volume (e.g. ±0.5 of origin) gives the canonical "lit nebula core" look; outside (e.g. (0, 0, 2)) is more "external star illuminating dust".
  - **colour** — RGB picker. Match to a stellar temperature for realism; or pick saturated colours for stylised work.
  - **intensity** — Light power. 0 disables the light without removing it from the count.
  - **falloff** — Distance attenuation exponent. 2 = inverse-square (physical); lower = light reaches further.

### 4.6 Starfield

#### Distribution

- **density (grid scale)** — Grid scale of layer 0. Doubles each parallax layer. Higher = more, smaller stars.
- **brightness** — Overall star multiplier. 0 disables the starfield.
- **parallax layers** — 1 to 3. Each successive layer doubles the grid density and reduces magnitude — gives a depth cue.
- **IMF exponent** — `mag = pow(rand, exp)`. 5 (default) = ~95% dim stars (realistic IMF); 1 = uniform brightness.

#### Galactic plane

- **strength** — How much the galactic-plane band concentrates stars. 0 = uniform sphere; 4 = strong Milky-Way band.
- **width** — Gaussian falloff width away from the plane. 0.3 default.
- **plane normal x / y / z** — Vector defining the perpendicular to the plane. Default (0.3, 1.0, 0.2) is a slight tilt; (0, 1, 0) makes the band horizontal.

#### Colour (Kelvin)

- **T min** — Coolest star in the distribution. 2700 K ≈ M-class red dwarfs; 3500 K ≈ orange dwarfs.
- **T max** — Hottest star. 10000 K ≈ A-class white; 30000 K ≈ O/B-class blue giants.

#### PSF / diffraction

- **PSF threshold** — Magnitude above which a star gets a diffraction cross. 0.6 default.
- **PSF intensity** — Spike brightness multiplier.
- **spike length** — Angular extent of each spike (radians).
- **spike count** — 4, 6, or 8 spokes. (Currently visualised as a 4-spoke cross regardless; full N-fold spikes ship in a v1.x update.)

### 4.7 Performance *(desktop only)*

A bench button runs a fixed sequence of test renders (1024 / 2048 at 64 / 128 / 256 march steps) and reports median frame time per config. Use this once to verify your GPU is healthy and to understand cost scaling.

### 4.8 Export

- **kind** — `Equirect (2:1)` or `Cubemap (6 faces)`. Web is equirect-only.
- **format** — `PNG (sRGB tonemapped)` or `EXR (linear HDR)`. Web is PNG-only.
- **width / face size** — 1K / 2K / 4K / 8K (equirect) or 512 / 1K / 2K / 4K (cubemap face). Web is capped at 1K.
- **Export PNG…** / **Export EXR…** / **Export cube faces…** — Triggers the render. Native opens a file dialog. Web triggers a browser download.

Cubemap PNG export writes six files alongside the path you choose: `<name>_px.png`, `_nx`, `_py`, `_ny`, `_pz`, `_nz`. Naming matches the OpenGL/DirectX convention so files drop straight into Unity / Unreal / Bevy / Godot cubemap importers without renaming.

EXR is linear scene-referred radiance (the tonemap is bypassed — `tonemap_mode = 3`). Use this when you need the unclamped HDR data for relighting or post-processing in Blender / Nuke.

---

## 5. Recipes

Practical short answers. Start from a shipped preset, then nudge.

**Dust lanes that redden the stars behind them.** Switch **Scattering → reddening** to ISM. Bump intensity to ~3. Switch **Emission model → density kind** to Multichannel. Raise dust strength to ~2. The dense regions now darken AND warm the background through interstellar extinction.

**Pillars-of-Creation aesthetic.** Multichannel emission, palette = NATURAL. Hα strength ~1.2, [OIII] strength ~0.8, [OIII] sharpness ~5 (sharp ionised cores). Add one or two in-volume point lights at high intensity (10+) inside the densest region. dust strength ~2.

**Pleiades-style blue reflection cloud.** Multichannel emission with Hα strength = 0, [OIII] strength = 0 (no self-emission). Switch reddening to Custom and set σₑ = (0.4, 0.7, 1.0) (Rayleigh-favoured). Albedo = 1.0. Place 3-4 bright lights *outside* the volume (e.g. position (0, 0, 2.5)) so the dust scatters their light. Raise ambient emission to 0 or near-zero — Pleiades has no internal emission.

**Galactic-plane band stretched across the horizon.** Starfield → galactic plane: strength = 3, width = 0.2, plane normal = (0, 1, 0). Bump density to 120. T max to 25000. The horizon now has a clear bright band of stars.

**Spike-y bright stars (telescope look).** Starfield → PSF: threshold down to 0.4 (more stars get spikes), intensity up to 1.0, spike length to ~0.02. The bright stars now have visible diffraction crosses.

**Low-contrast nebula curtain (subtle background).** Frame → quality = Quality. Nebula → Density distribution: contrast = 0, pivot = 0.5 (back to Phase-5 linear clip). Drop ridged blend to 0.2 (smoother). PostFX → exposure = −1.5. Result: a soft volumetric haze without dominant cores.

**Make the nebula look "physical" instead of "stylised".** Domain warp → kind = Curl. Phase = Cornette-Shanks. Scattering → reddening = ISM at intensity ~2. Emission model → density kind = Multichannel, palette = NATURAL. This is what the realism research phase (R1–R3) was for.

**Match an existing reference image.** Iterate visually; physics gets you 80% of the way there but final look is taste. Save the result as a preset (`Save…`) so you can return to it.

---

## 6. Export formats

| Format          | Use case                                                                            |
|-----------------|-------------------------------------------------------------------------------------|
| Equirect PNG    | Universal. Drops into any engine's "panoramic skybox" / "spherical environment" slot. Use 4K for hero scenes, 2K for backgrounds. |
| Equirect EXR    | When you need HDR data — Blender environment lighting, relight passes, post-processing in Nuke. Linear scene-referred. ~10× the file size of PNG. |
| Cubemap PNG     | Older engines or shaders that expect six face textures. Drops into Unity Reflection Probes / Unreal Cubemap import / Bevy / Godot cubemap directly. Each face is square. |
| Cubemap EXR     | Currently desktop-only and ships in a v1.x update.                                  |

**Engine drop-in recipes:**

- **Unity** — Project → Import → drag `erebus_equirect_4096.png`. Inspector: Texture Shape = Cube, Mapping = Latitude-Longitude Layout. Use as a Skybox material.
- **Unreal** — Import as Texture. Right-click → Create LongLat Cubemap (or use the HDRI Backdrop actor for runtime use).
- **Bevy** — Load as `Image`, set `TextureViewDimension::Cube` after slicing — or use the `bevy_cubemap_loader` crate for direct cubemap PNG sets.
- **Godot** — Project Settings → Rendering → Environment → Sky Material = `PanoramaSkyMaterial`. Set the panorama texture.
- **Blender** — Shader editor → Background → Environment Texture, point at the equirect EXR for proper HDR lighting (image-based lighting).

---

## 7. Performance & quality tiers

| Tier     | Use when                                                | Cost     |
|----------|---------------------------------------------------------|----------|
| Draft    | Composing on integrated graphics or while scrubbing many sliders fast. | Cheapest |
| Preview  | Default. Composing on a discrete GPU.                   | Mid      |
| Quality  | Reviewing the final composition before exporting.       | High     |
| Export   | Use this just before clicking Export PNG/EXR.           | Highest  |

The cost scales linearly in **steps** and roughly linearly in **shadow_steps × active_lights**. Doubling either roughly doubles the per-pixel work. The offscreen render is cached between identical frames, so idle preview is essentially free — costs only show during slider drags.

The **preview scale** slider is the cheapest performance lever. Drop to 0.25 for a 4× speedup with minor quality loss; the result still looks good through the skybox preview at narrow FOV.

---

## 8. Web vs native — what's different

| Feature                | Native           | Web                                              |
|------------------------|------------------|--------------------------------------------------|
| Live preview           | Full quality     | Auto-tuned interactive defaults (32 march steps, 1 star layer, no bloom). "Render hero shot" button gives a one-shot Quality render. |
| Export PNG             | 1K / 2K / 4K / 8K equirect, 512 / 1K / 2K / 4K cubemap | 1K equirect only. Quality-tier override. |
| Export EXR             | Yes              | No.                                              |
| Cubemap export         | Yes              | No.                                              |
| Save / Load preset     | Native file dialog | Disabled (browser sandbox).                    |
| Performance benchmark  | Yes              | Hidden (would freeze the tab).                   |
| Hot-reload of shaders  | Yes              | No (shaders are baked into the wasm bundle).     |
| First-frame composition | Default uniforms | Auto-loads Synthwave preset.                    |
| Idle CPU usage         | Continuous repaint | Sleeps when nothing is changing.               |

The browser cap is mostly a GPU watchdog issue — Chrome's WebGPU on integrated graphics will reset the device if a single render runs too long. The desktop binary has no such limit.

---

## 9. Troubleshooting

**Black canvas on web load.** Open DevTools console (F12). If you see `WebGPU adapter not found`, your browser doesn't have WebGPU enabled. Chrome 113+ on Windows / macOS / Linux works out of the box. On Linux Chrome you may need to enable `chrome://flags/#enable-unsafe-webgpu`. Firefox needs 145+. Safari needs 26+ (macOS). The page shows a fallback message in this case.

**Tab goes white during web export.** GPU device lost — usually means the export request was too big for the integrated GPU's watchdog timer. Hard-refresh (`Ctrl+Shift+R`), use 1K only, drop bloom intensity to 0 if needed. The desktop binary has no such limit.

**Preview is slow on integrated graphics.** Drop **preview scale** to 0.25, switch quality tier to **Draft**, and disable bloom (intensity = 0). On web the **Render hero shot** button gives you a one-frame Quality render that costs the slow render only once and then displays at 60 fps.

**Slider feels laggy.** Slider drags are real-time renders. Each new value triggers a fresh raymarch. On a slow GPU drop preview scale; the render gets cheaper. Once you stop dragging, the offscreen freezes and the next slider movement only pays the cost again.

**Stars look like fuzzy blobs at narrow FOV.** Should not happen — stars are drawn screen-space in the composite pass, so they stay crisp at any zoom. If you see this, file a bug with a screenshot and the seed.

**Export PNG looks pixelated / banded compared to the live preview.** Check that you bumped to **Quality** or **Export** tier before exporting. Otherwise you exported with the cheap interactive defaults (32 march steps).

**Loaded RON preset crashes / fails to parse.** Erebus reads RONs at format_version 1 *and* 2 — older v1 preset files load through a back-compat shim. If your preset doesn't load, send the file (it's just text); one of the migration arms may need extending.

**Where are the logs?** On native, run from a terminal — log lines print to stdout. Set the env var `RUST_LOG=info` for default verbosity, `RUST_LOG=debug` for more. On web, open the browser DevTools console.

---

## 10. Determinism

Same seed + same uniforms = bit-identical output across machines and across the desktop ↔ web boundary, on a given Erebus version. This holds as long as you don't switch between major versions or change any uniform between save and reload. Use this to ship preset RON files alongside game assets — collaborators reproduce your composition exactly.

A change to the `format_version` field in a saved preset (currently `2`) signals a schema bump; older presets load through the migration arms in `src/preset/migrate.rs`. The shipped Synthwave / Cyberpunk / Retro Sci-Fi presets are version-locked at v1 and load identically across all releases.

---

## Appendix: physics references

Erebus is calibrated against published literature where it could be. The list below is for users who want to know the engine isn't guessing.

- **Bridson, Houriham & Nordenstam (2007).** *Curl-Noise for Procedural Fluid Flow.* SIGGRAPH 2007. The divergence-free vector field used by **Domain warp → Curl**.
- **Cornette & Shanks (1992).** *Physically reasonable analytic expression for the single-scattering phase function.* Applied Optics 31(16):3152–3160. The phase function used by **Scattering → Cornette-Shanks**.
- **Vazquez-Semadeni (1994).** *Hierarchical structure in nearly pressureless flows…* The Astrophysical Journal 423:681. The log-normal density distribution behind **Density distribution → contrast**.
- **Mathis, Rumpl & Nordsieck (1977).** Used to derive the R_V = 3.1 reddening curve `[0.65, 1.00, 1.45]` exposed via **Scattering → reddening = ISM**.
- **Hubble Palette (SHO mapping).** SII (672 nm) → R, Hα (656 nm) → G, [OIII] (500.7 nm) → B. The popular astrophotography false-colour scheme. NATURAL and HOO palette modes are physical line emissions, not SHO; full SHO ships in a v1.x update.
- **Heckel.** *Real-time dreamy cloudscapes with volumetric raymarching.* The single-scatter pipeline backbone.

For implementation depth, see [docs/SHADER_NOTES.md](SHADER_NOTES.md), [docs/ARCHITECTURE.md](ARCHITECTURE.md), and the per-bug write-ups in [docs/BUGS.md](BUGS.md).

---

*Erebus v1.0. Source code under MIT/Apache-2.0 dual license. Bug reports welcome.*
