# Erebus

A procedural volumetric nebula and starfield generator. The modern, HDR, real-time successor to Spacescape — standalone, cross-platform, engine-agnostic.

Erebus produces high-resolution skyboxes, equirectangular latlongs, and 2D tileable backgrounds suitable for game engines (Unity / Unreal / Godot / Bevy), film comp work, wallpapers, and printable stills. Output is HDR-correct end-to-end and exportable to 8-bit PNG or linear OpenEXR up to 16K.

## Features

- **Volumetric raymarching** of a 3D Perlin–Worley noise field with domain warping and curl-noise swirl, producing filamentary tendrils, dust lanes, and depth-shadowed cores that pure 2D-layered tools cannot match.
- **Physically grounded scattering**: Beer–Lambert transmittance and a Henyey–Greenstein phase function with adjustable anisotropy. Optional in-volume point lights with shadow marching.
- **Blackbody starfield** with per-star Kelvin temperature, IMF-weighted brightness distribution, optional galactic-plane density mask, and diffraction-spike PSF billboards for cinematic glare.
- **HDR pipeline** in linear RGBA16F throughout. Energy-conserving bloom pyramid, optional FFT convolution bloom for real diffraction kernels.
- **Tone mapping**: AgX (default), ACES Fitted, Tony McMapface, Reinhard. Exposure (stops) is the top-level grading control. Triangular-PDF deband dither prevents banding in dark gradients.
- **Output modes**: equirectangular latlong (seamless in longitude), 6-face cubemap, 2D tileable (4D Clifford-torus). PNG and EXR. Tiled offscreen rendering up to 16K × 8K with optional 2× / 4× supersampling.
- **Preset system**: deterministic seeds, RON/JSON serialization, format-versioned for forward compatibility. A 200-character preset string fully reproduces a scene.
- **Cross-platform**: native Windows / macOS / Linux from one binary. WebGPU/WASM build planned from the same WGSL shaders.

## Stack

- **Rust** + **`wgpu`** (Vulkan / Metal / DX12 / WebGPU) + **WGSL** shaders
- **`eframe` / `egui`** for the control surface
- **`rfd`** for native file dialogs (also works under WASM)
- **`serde`** + **`ron`** for preset I/O, **`image`** + **`exr`** for output

## Build and run

```bash
cargo run --release
```

Native debug builds enable `opt-level = 1` for the workspace and `opt-level = 3` for dependencies, which keeps shader iteration responsive without hour-long initial compiles.

### Web build (planned)

```bash
cargo build --release --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir dist target/wasm32-unknown-unknown/release/erebus.wasm
```

## Repository layout

```
Erebus/
├── src/
│   ├── main.rs             # Entry point.
│   ├── app/                # eframe shell, top-level state, config.
│   ├── gui/                # egui panels, custom widgets, theme.
│   ├── render/             # wgpu context, frame graph, uniforms, hot-reload.
│   │   ├── passes/         # precompute, nebula, starfield, bloom, tonemap.
│   │   └── resources/      # textures, buffers, samplers.
│   ├── export/             # tiling, PNG, EXR, cubemap, equirect.
│   ├── preset/             # schema, I/O, format-version migrations.
│   └── noise/              # CPU helpers: blackbody LUT, blue noise, seeding.
├── shaders/
│   ├── common/             # uniforms, math, noise, sampling helpers.
│   ├── nebula/             # raymarch, density, in-volume lighting.
│   ├── starfield/          # grid-hash field, PSF billboards.
│   ├── bloom/              # downsample, upsample, composite.
│   ├── tonemap/            # AgX, ACES, Tony McMapface, Reinhard, dither.
│   ├── compute/            # 3D noise / blackbody / gradient bake.
│   └── fullscreen.wgsl     # Reused fullscreen-triangle vertex shader.
├── assets/
│   ├── presets/            # Shipped RON presets (synthwave, cyberpunk, …).
│   ├── luts/               # Optional baked LUTs.
│   └── fonts/              # UI fonts (if/when added).
├── docs/
│   ├── ROADMAP.md          # Phase-by-phase development plan.
│   ├── ARCHITECTURE.md     # Render graph and data flow.
│   └── SHADER_NOTES.md     # Tuning notes for each pass.
├── examples/               # Headless export, CLI reference.
├── tests/                  # Preset round-trip, WGSL Naga validation.
├── benches/                # GPU timing harness.
├── .github/workflows/      # CI: fmt, clippy, test, wasm-check.
├── Cargo.toml
├── rust-toolchain.toml
├── rustfmt.toml
└── README.md
```

## Roadmap

See [docs/ROADMAP.md](docs/ROADMAP.md) for the full phase plan. Top-level milestones:

1. **M1 — Foundation**: window, wgpu init, fullscreen pass, hot-reload.
2. **M2 — Nebula MVP**: 3D noise bake, raymarch, gradient LUT.
3. **M3 — Starfield**: grid-hash field, blackbody color, PSF billboards.
4. **M4 — Post chain**: HDR bloom, tone mapping, deband.
5. **M5 — Export**: equirect, cubemap, PNG/EXR, tiled supersample.
6. **M6 — UX & presets**: panel polish, gradient editor, preset I/O.
7. **M7 — Web build**: WASM target, browser file APIs.
8. **M8 — Release**: benchmarks, demo gallery, itch.io packaging.

## License

Dual-licensed under either of:

- MIT — see [LICENSE-MIT](LICENSE-MIT)
- Apache 2.0 — see [LICENSE-APACHE](LICENSE-APACHE)

at your option.

## Acknowledgements

Erebus stands on a generation of public shader research: Sébastien Hillaire (Frostbite), Iñigo Quilez (domain warping, noise derivatives), Krzysztof Narkowicz (ACES Fitted), Troy Sobotka (AgX), Tiffany Cover (PSF/diffraction stars), Duke (Shadertoy nebulae), Alex Peterson (Spacescape), and the demoscene tradition. See [docs/ROADMAP.md](docs/ROADMAP.md) and the in-app About panel for full citations.
