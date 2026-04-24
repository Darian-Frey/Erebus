// App shell: eframe lifecycle, top-level state owner, frame loop.

pub mod state;
pub mod config;

use crate::gui;
use crate::render::ErebusRenderer;

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

        Ok(Self {
            state: state::State::default(),
            start: std::time::Instant::now(),
        })
    }
}

impl eframe::App for ErebusApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.state.time = self.start.elapsed().as_secs_f32();

        // Drain shader-watcher events and surface any compile error to the UI.
        if let Some(wgpu_state) = frame.wgpu_render_state() {
            let mut renderer = wgpu_state.renderer.write();
            if let Some(r) = renderer.callback_resources.get_mut::<ErebusRenderer>() {
                r.poll_hot_reload();
                self.state.last_shader_error = r.last_shader_error.clone();
            }
        }

        gui::render(ctx, &mut self.state);

        // Animated debug shader needs continuous repaint.
        ctx.request_repaint();
    }
}
