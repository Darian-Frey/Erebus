// App shell: eframe lifecycle, top-level state owner, frame loop.

pub mod state;
pub mod config;

use crate::export::{self, ExportFormat, ExportKind, ExportRequest};
use crate::gui;
use crate::preset::{self, PresetAction};
use crate::render::{ErebusRenderer, FrameUniforms};

pub fn run() -> anyhow::Result<()> {
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

struct ErebusApp {
    state: state::State,
    start: std::time::Instant,
    last_frame: std::time::Instant,
}

impl ErebusApp {
    fn new(cc: &eframe::CreationContext<'_>) -> anyhow::Result<Self> {
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

        let now = std::time::Instant::now();
        Ok(Self {
            state: state::State::default(),
            start: now,
            last_frame: now,
        })
    }
}

impl eframe::App for ErebusApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let now = std::time::Instant::now();
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

        // Drain shader-watcher events and surface any compile error to the UI.
        // Also re-upload the gradient LUT if the user just loaded a preset.
        if let Some(wgpu_state) = frame.wgpu_render_state() {
            let mut renderer = wgpu_state.renderer.write();
            if let Some(r) = renderer.callback_resources.get_mut::<ErebusRenderer>() {
                r.poll_hot_reload();
                self.state.last_shader_error = r.last_shader_error.clone();
                if self.state.gradient_dirty {
                    r.update_gradient(&wgpu_state.queue, &self.state.gradient);
                    self.state.gradient_dirty = false;
                }
            }
        }

        gui::render(ctx, &mut self.state);

        // Animated debug shader needs continuous repaint.
        ctx.request_repaint();

        // After the GUI pass: if the user clicked Export, run it now.
        // Synchronous — the UI freezes for the duration of the render +
        // file write. Async export with a progress bar is a Phase 8 polish.
        if let Some(req) = self.state.pending_export.take() {
            let status = self.run_export(req, frame);
            self.state.last_export_status = Some(status);
        }

        if let Some(action) = self.state.pending_preset.take() {
            let status = self.run_preset_action(action);
            self.state.last_preset_status = Some(status);
        }
    }
}

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

        let pixels_result = {
            let mut binding = wgpu_state.renderer.write();
            let renderer = match binding.callback_resources.get_mut::<ErebusRenderer>() {
                Some(r) => r,
                None => return "renderer not available".to_string(),
            };

            let (w, h) = match req.kind {
                ExportKind::Equirect => (req.width, req.width / 2),
            };
            let frame_u = FrameUniforms {
                resolution: [w as f32, h as f32],
                time: self.state.time,
                exposure: self.state.post.exposure,
                seed: self.state.seed,
                frame_index: self.state.frame_index,
                ..Default::default()
            };
            let started = std::time::Instant::now();
            let result = renderer.render_equirect_rgba8(
                &wgpu_state.queue,
                w,
                h,
                frame_u,
                self.state.nebula,
                self.state.lighting,
                self.state.starfield,
                self.state.post,
            );
            log::info!(
                "export render {}×{} done in {:.2}s",
                w,
                h,
                started.elapsed().as_secs_f32()
            );
            result.map(|p| (w, h, p))
        };

        match pixels_result {
            Ok((w, h, pixels)) => match req.format {
                ExportFormat::Png => match export::png::write_rgba8(&path, w, h, &pixels) {
                    Ok(_) => format!("saved {}", path.display()),
                    Err(e) => format!("PNG write failed: {e}"),
                },
            },
            Err(e) => format!("render failed: {e}"),
        }
    }
}

impl ErebusApp {
    fn run_preset_action(&mut self, action: PresetAction) -> String {
        match action {
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

fn prompt_export_path(req: &ExportRequest) -> Option<std::path::PathBuf> {
    let default_name = format!(
        "erebus_{}_{}.{}",
        match req.kind {
            ExportKind::Equirect => "equirect",
        },
        req.width,
        req.format.extension(),
    );
    rfd::FileDialog::new()
        .add_filter(req.format.extension().to_uppercase().as_str(), &[req.format.extension()])
        .set_file_name(&default_name)
        .save_file()
}
