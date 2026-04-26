// App shell: eframe lifecycle, top-level state owner, frame loop.

pub mod state;
pub mod config;

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

use web_time::Instant;

#[cfg(not(target_arch = "wasm32"))]
use crate::export::{self, ExportFormat, ExportKind, ExportRequest};
#[cfg(target_arch = "wasm32")]
use crate::export::{ExportFormat, ExportKind, ExportRequest};
use crate::gui;
use crate::preset::{self, PresetAction};
#[cfg(not(target_arch = "wasm32"))]
use crate::render::bench::{BenchResult, BENCH_CONFIGS, BENCH_RUNS, BENCH_WARMUP};
use crate::render::ErebusRenderer;
use crate::render::FrameUniforms;

/// Run the native eframe shell. WASM target uses a different entry point
/// (see [`crate::start`]) that wires the same `ErebusApp` into a canvas.
#[cfg(not(target_arch = "wasm32"))]
pub fn run_native() -> anyhow::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title(config::WINDOW_TITLE)
            .with_inner_size([config::INITIAL_WIDTH, config::INITIAL_HEIGHT])
            .with_min_inner_size([config::MIN_WIDTH, config::MIN_HEIGHT]),
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };

    eframe::run_native(
        config::WINDOW_TITLE,
        options,
        Box::new(|cc| Ok(Box::new(ErebusApp::new(cc)?))),
    )
    .map_err(|e| anyhow::anyhow!("eframe: {e}"))
}

pub struct ErebusApp {
    state: state::State,
    start: Instant,
    last_frame: Instant,
}

impl ErebusApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> anyhow::Result<Self> {
        let wgpu_state = cc
            .wgpu_render_state
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("eframe was not started with the wgpu backend"))?;

        crate::gui::theme::install(&cc.egui_ctx);

        let renderer = ErebusRenderer::new(wgpu_state)?;
        wgpu_state
            .renderer
            .write()
            .callback_resources
            .insert(renderer);

        let now = Instant::now();
        Ok(Self {
            state: state::State::default(),
            start: now,
            last_frame: now,
        })
    }
}

impl eframe::App for ErebusApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let now = Instant::now();
        let dt_ms = now.duration_since(self.last_frame).as_secs_f32() * 1000.0;
        self.last_frame = now;
        self.state.time = self.start.elapsed().as_secs_f32();
        self.state.frame_index = self.state.frame_index.wrapping_add(1);
        // Exponential moving average — smooths the noisy per-frame jitter.
        let a = 0.1;
        self.state.frame_ms_ema = self.state.frame_ms_ema * (1.0 - a) + dt_ms * a;
        self.state.fps_ema = if self.state.frame_ms_ema > 0.0 {
            1000.0 / self.state.frame_ms_ema
        } else {
            0.0
        };

        // Adaptive preview: hash the visual parameters and bump the
        // last-interaction timestamp whenever the hash changes. While the
        // user is interacting (timestamp < 250 ms ago), the GUI scales the
        // offscreen target down to keep the preview responsive.
        let h = params_hash(&self.state);
        if h != self.state.last_param_hash {
            self.state.last_param_hash = h;
            self.state.last_interaction_at = Instant::now();
            // Any slider change leaves hero-shot mode — the next frame should
            // render at the cheap interactive defaults again so the user can
            // keep iterating without the multi-second hero-shot cost.
            self.state.hero_shot = false;
        }
        self.state.interacting =
            self.state.last_interaction_at.elapsed() < Duration::from_millis(250);

        // Drain shader-watcher events and surface any compile error to the UI.
        // Also re-upload the gradient LUT if the user just loaded a preset.
        if let Some(wgpu_state) = frame.wgpu_render_state() {
            let mut renderer = wgpu_state.renderer.write();
            if let Some(r) = renderer.callback_resources.get_mut::<ErebusRenderer>() {
                if r.poll_hot_reload() {
                    self.state.last_rendered_hash = None;
                }
                self.state.last_shader_error = r.last_shader_error.clone();
                if self.state.gradient_dirty {
                    r.update_gradient(&wgpu_state.queue, &self.state.gradient);
                    self.state.gradient_dirty = false;
                    // Gradient is sampled inside the raymarch shader. The
                    // cached HDR target was rendered against the *old* LUT,
                    // so force a re-render now that the LUT has changed.
                    self.state.last_rendered_hash = None;
                }
            }
        }

        gui::render(ctx, &mut self.state);

        // Animated debug shader needs continuous repaint.
        ctx.request_repaint();

        // Native export: synchronous pop-up file dialog + render + write.
        #[cfg(not(target_arch = "wasm32"))]
        if let Some(req) = self.state.pending_export.take() {
            let status = self.run_export(req, frame);
            self.state.last_export_status = Some(status);
        }
        // Wasm export: split across multiple frames. Frame N submits the GPU
        // work and stashes a `PendingExport`; subsequent frames poll the
        // device and check the map-async receiver. Without this split the JS
        // event loop is blocked, so the readback callback never fires.
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(req) = self.state.pending_export.take() {
                let status = self.start_export_wasm(req, frame);
                self.state.last_export_status = Some(status);
            }
            self.poll_export_wasm(frame);
        }

        // Native-only: bench. Reads back GPU timestamps after a long warmup;
        // not a thing browsers should do.
        #[cfg(not(target_arch = "wasm32"))]
        if self.state.pending_bench {
            self.state.pending_bench = false;
            self.state.bench_running = true;
            self.run_bench(frame);
            self.state.bench_running = false;
        }

        if let Some(action) = self.state.pending_preset.take() {
            let status = self.run_preset_action(action);
            self.state.last_preset_status = Some(status);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl ErebusApp {
    fn run_export(&mut self, req: ExportRequest, frame: &mut eframe::Frame) -> String {
        let path = match req.path.clone() {
            Some(p) => p,
            None => match prompt_export_path(&req) {
                Some(p) => p,
                None => return "cancelled".to_string(),
            },
        };

        let wgpu_state = match frame.wgpu_render_state() {
            Some(s) => s,
            None => return "no wgpu render state".to_string(),
        };

        let started = Instant::now();
        let mut binding = wgpu_state.renderer.write();
        let renderer = match binding.callback_resources.get_mut::<ErebusRenderer>() {
            Some(r) => r,
            None => return "renderer not available".to_string(),
        };

        let frame_u = FrameUniforms {
            time: self.state.time,
            exposure: self.state.post.exposure,
            seed: self.state.seed,
            frame_index: self.state.frame_index,
            ..Default::default()
        };

        let result = match (req.kind, req.format) {
            (ExportKind::Equirect, ExportFormat::Png) => {
                let (w, h) = (req.width, req.width / 2);
                let pixels = renderer.render_equirect_rgba8(
                    &wgpu_state.queue,
                    w,
                    h,
                    frame_u,
                    self.state.nebula,
                    self.state.lighting,
                    self.state.starfield,
                    self.state.post,
                );
                pixels.and_then(|p| {
                    export::png::write_rgba8(&path, w, h, &p)?;
                    Ok(format!("saved {}", path.display()))
                })
            }
            (ExportKind::Equirect, ExportFormat::Exr) => {
                let (w, h) = (req.width, req.width / 2);
                let pixels = renderer.render_equirect_rgba32f(
                    &wgpu_state.queue,
                    w,
                    h,
                    frame_u,
                    self.state.nebula,
                    self.state.lighting,
                    self.state.starfield,
                    self.state.post,
                );
                pixels.and_then(|p| {
                    export::exr::write_rgba32f(&path, w, h, &p)?;
                    Ok(format!("saved {}", path.display()))
                })
            }
            (ExportKind::Cubemap, ExportFormat::Png) => {
                let face_size = req.width;
                let faces = renderer.render_cubemap_rgba8(
                    &wgpu_state.queue,
                    face_size,
                    frame_u,
                    self.state.nebula,
                    self.state.lighting,
                    self.state.starfield,
                    self.state.post,
                );
                faces.and_then(|f| {
                    let written = export::cubemap::write_six(&path, face_size, &f)?;
                    Ok(format!("saved {} faces under {}", written.len(), path.display()))
                })
            }
            (ExportKind::Cubemap, ExportFormat::Exr) => {
                Err(anyhow::anyhow!("Cubemap EXR not yet implemented (Phase 7.5)"))
            }
        };

        log::info!("export done in {:.2}s", started.elapsed().as_secs_f32());

        match result {
            Ok(msg) => msg,
            Err(e) => format!("export failed: {e}"),
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl ErebusApp {
    /// Wasm export step 1: validate, snapshot uniforms, submit GPU work,
    /// stash the `PendingExport` in `state.pending_export_job`. Returns the
    /// initial status string (e.g. `rendering 2048×1024…`). Subsequent frames
    /// drive the readback via `poll_export_wasm`.
    fn start_export_wasm(
        &mut self,
        req: ExportRequest,
        frame: &mut eframe::Frame,
    ) -> String {
        if self.state.pending_export_job.is_some() {
            return "export already in flight".to_string();
        }
        if !matches!(req.kind, ExportKind::Equirect) || !matches!(req.format, ExportFormat::Png) {
            return "web export currently supports equirect PNG only".to_string();
        }

        let wgpu_state = match frame.wgpu_render_state() {
            Some(s) => s,
            None => return "no wgpu render state".to_string(),
        };

        let mut binding = wgpu_state.renderer.write();
        let renderer = match binding.callback_resources.get_mut::<ErebusRenderer>() {
            Some(r) => r,
            None => return "renderer not available".to_string(),
        };

        let frame_u = FrameUniforms {
            time: self.state.time,
            exposure: self.state.post.exposure,
            seed: self.state.seed,
            frame_index: self.state.frame_index,
            ..Default::default()
        };

        let (w, h) = (req.width, req.width / 2);
        log::info!("export: submitting GPU work for {w}×{h}");
        match renderer.submit_equirect_export(
            &wgpu_state.queue,
            w,
            h,
            frame_u,
            self.state.nebula,
            self.state.lighting,
            self.state.starfield,
            self.state.post,
        ) {
            Ok(job) => {
                self.state.pending_export_job = Some(job);
                format!("rendering {w}×{h}…")
            }
            Err(e) => format!("export failed: {e}"),
        }
    }

    /// Wasm export step 2: each frame, drive `device.poll(Poll)` so the
    /// browser-side readback callback can fire, then non-blocking-check the
    /// rx. When pixels arrive, encode PNG + trigger download.
    fn poll_export_wasm(&mut self, frame: &mut eframe::Frame) {
        if self.state.pending_export_job.is_none() {
            return;
        }
        let Some(wgpu_state) = frame.wgpu_render_state() else {
            return;
        };
        let mut binding = wgpu_state.renderer.write();
        let Some(renderer) = binding.callback_resources.get_mut::<ErebusRenderer>() else {
            return;
        };
        renderer.poll_export_progress();

        // Borrow the job by reference for the try_finish call so we can take
        // ownership only on success/failure.
        let outcome = {
            let job = self.state.pending_export_job.as_ref().unwrap();
            renderer.try_finish_export(job)
        };
        match outcome {
            Ok(None) => {} // still pending — keep polling
            Ok(Some(pixels)) => {
                let job = self.state.pending_export_job.take().unwrap();
                let (w, h) = (job.width, job.height);
                log::info!("export: GPU readback complete ({} bytes)", pixels.len());
                let status = match crate::export::png::encode_rgba8(w, h, &pixels) {
                    Ok(png) => {
                        let filename = format!("erebus_equirect_{w}.png");
                        log::info!("export: PNG {} bytes, triggering download", png.len());
                        match crate::export::web::download_bytes(&png, "image/png", &filename) {
                            Ok(()) => format!(
                                "downloaded {filename} ({:.1} MB)",
                                png.len() as f32 / 1.0e6
                            ),
                            Err(e) => format!("download failed: {e}"),
                        }
                    }
                    Err(e) => format!("PNG encode failed: {e}"),
                };
                self.state.last_export_status = Some(status);
            }
            Err(e) => {
                self.state.pending_export_job = None;
                self.state.last_export_status = Some(format!("export failed: {e}"));
            }
        }
    }
}

impl ErebusApp {
    fn run_preset_action(&mut self, action: PresetAction) -> String {
        match action {
            #[cfg(not(target_arch = "wasm32"))]
            PresetAction::SaveToFile => {
                let preset = preset::Preset {
                    format_version: preset::schema::CURRENT_VERSION,
                    name: self.state.preset_name.clone(),
                    seed: self.state.seed,
                    nebula: self.state.nebula,
                    lighting: self.state.lighting,
                    starfield: self.state.starfield,
                    post: self.state.post,
                    gradient: self.state.gradient.clone(),
                };
                let default_name = format!(
                    "{}.ron",
                    self.state.preset_name.replace(|c: char| !c.is_alphanumeric(), "_")
                );
                match rfd::FileDialog::new()
                    .add_filter("RON preset", &["ron"])
                    .set_file_name(&default_name)
                    .save_file()
                {
                    Some(path) => match preset::io::save_to_file(&path, &preset) {
                        Ok(_) => format!("saved {}", path.display()),
                        Err(e) => format!("save failed: {e}"),
                    },
                    None => "cancelled".to_string(),
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            PresetAction::LoadFromFile => match rfd::FileDialog::new()
                .add_filter("RON preset", &["ron"])
                .pick_file()
            {
                Some(path) => match preset::io::load_from_file(&path) {
                    Ok(p) => {
                        self.apply_preset(p);
                        format!("loaded {}", path.display())
                    }
                    Err(e) => format!("load failed: {e}"),
                },
                None => "cancelled".to_string(),
            },
            #[cfg(target_arch = "wasm32")]
            PresetAction::SaveToFile | PresetAction::LoadFromFile => {
                "file save/load is desktop-only".to_string()
            }
            PresetAction::LoadShipped(which) => match which.load() {
                Ok(p) => {
                    self.apply_preset(p);
                    format!("loaded {}", which.label())
                }
                Err(e) => format!("load failed: {e}"),
            },
        }
    }

    fn apply_preset(&mut self, p: preset::Preset) {
        self.state.preset_name = p.name;
        self.state.seed = p.seed;
        self.state.nebula = p.nebula;
        self.state.lighting = p.lighting;
        self.state.starfield = p.starfield;
        self.state.post = p.post;
        self.state.gradient = p.gradient;
        self.state.gradient_dirty = true;
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl ErebusApp {
    fn run_bench(&mut self, frame: &mut eframe::Frame) {
        let Some(wgpu_state) = frame.wgpu_render_state() else {
            log::error!("bench: wgpu_render_state unavailable");
            return;
        };
        let mut binding = wgpu_state.renderer.write();
        let Some(renderer) = binding.callback_resources.get_mut::<ErebusRenderer>() else {
            log::error!("bench: ErebusRenderer not found in callback_resources");
            return;
        };

        let mut results = Vec::with_capacity(BENCH_CONFIGS.len());
        for &(label, w, h, steps) in BENCH_CONFIGS {
            // Build per-config nebula uniforms with the requested step count.
            let mut nebula = self.state.nebula;
            nebula.steps = steps;

            let frame_u = FrameUniforms {
                time: self.state.time,
                exposure: self.state.post.exposure,
                seed: self.state.seed,
                frame_index: self.state.frame_index,
                ..Default::default()
            };

            match renderer.bench_render(
                &wgpu_state.queue,
                w,
                h,
                BENCH_WARMUP,
                BENCH_RUNS,
                frame_u,
                nebula,
                self.state.lighting,
                self.state.starfield,
                self.state.post,
            ) {
                Ok(ms) => {
                    log::info!("bench {label}: {ms:.2} ms");
                    results.push(BenchResult {
                        label: label.to_string(),
                        width: w,
                        height: h,
                        steps,
                        ms_median: ms,
                    });
                }
                Err(e) => {
                    log::error!("bench {label} failed: {e}");
                }
            }
        }

        self.state.bench_results = results;
    }
}

/// Hash the parameters that the user can drive from the GUI. Used to detect
/// "is the user mid-interaction" for the adaptive-preview auto-downscale.
fn params_hash(state: &crate::app::state::State) -> u64 {
    let mut h = DefaultHasher::new();
    bytemuck::bytes_of(&state.nebula).hash(&mut h);
    bytemuck::bytes_of(&state.lighting).hash(&mut h);
    bytemuck::bytes_of(&state.starfield).hash(&mut h);
    bytemuck::bytes_of(&state.post).hash(&mut h);
    state.seed.hash(&mut h);
    state.preview_scale.to_bits().hash(&mut h);
    (state.view_mode as u32).hash(&mut h);
    h.finish()
}

#[cfg(not(target_arch = "wasm32"))]
fn prompt_export_path(req: &ExportRequest) -> Option<std::path::PathBuf> {
    let kind_label = match req.kind {
        ExportKind::Equirect => "equirect",
        ExportKind::Cubemap => "cubemap",
    };
    // For cubemap PNG we save 6 files derived from the chosen base name; the
    // dialog still asks for a single .png path so the user can see where the
    // files will land.
    let default_name = format!(
        "erebus_{}_{}.{}",
        kind_label,
        req.width,
        req.format.extension(),
    );
    rfd::FileDialog::new()
        .add_filter(
            req.format.extension().to_uppercase().as_str(),
            &[req.format.extension()],
        )
        .set_file_name(&default_name)
        .save_file()
}
